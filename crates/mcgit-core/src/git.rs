use std::path::Path;
use std::process::Command;

use crate::types::{GitError, RestoreError};

/// Runs `git <args>` in `world_dir`, mapping a failed spawn or a non-zero
/// exit code to `GitError`. Every function in this module that shells out
/// to `git` goes through here instead of repeating the same
/// spawn/check-status/read-stderr dance.
fn run(world_dir: &Path, args: &[&str]) -> Result<std::process::Output, GitError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(world_dir)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    Ok(output)
}

/// Runs `git init` in `world_dir`. Idempotent: `git init` on an
/// already-initialized repository is a safe no-op (Git's own guarantee),
/// so callers don't need to check `is_repository` first.
pub fn init(world_dir: &Path) -> Result<(), GitError> {
    run(world_dir, &["init"])?;
    exclude_session_lock(world_dir)?;
    Ok(())
}

/// Makes sure `session.lock` is never versioned: it's a transient file
/// Minecraft (and `is_currently_open`) uses to detect an open world, not
/// part of the world's actual content, so it shouldn't show up as noise in
/// every snapshot the player takes while playing. Written to
/// `.git/info/exclude` rather than a tracked `.gitignore` — Git's own
/// mechanism for local-only ignore rules, so it never needs a commit of
/// its own and a freshly versioned world still looks untouched until the
/// player saves their own first snapshot.
fn exclude_session_lock(world_dir: &Path) -> Result<(), GitError> {
    let path = world_dir.join(".git").join("info").join("exclude");
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    if existing.lines().any(|line| line == "session.lock") {
        return Ok(());
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str("session.lock\n");
    std::fs::write(&path, updated)?;
    Ok(())
}

/// Whether `world_dir` is already the root of a Git repository.
pub fn is_repository(world_dir: &Path) -> bool {
    world_dir.join(".git").is_dir()
}

/// Outcome of trying to save a snapshot. `NothingToCommit` is not an error —
/// it just means the world hasn't changed since the last snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitOutcome {
    Created(String),
    NothingToCommit,
}

/// Sets a `--local` Git identity for `world_dir`'s repository, so commits
/// work even on a machine that has never configured a global `user.name`/
/// `user.email`. `--local` always wins over `--global`/`--system`, so this
/// never touches the player's own identity if they happen to have one.
fn ensure_identity(world_dir: &Path) -> Result<(), GitError> {
    run(world_dir, &["config", "--local", "user.name", "mcgit"])?;
    run(world_dir, &["config", "--local", "user.email", "mcgit@localhost"])?;
    Ok(())
}

/// Saves a snapshot of `world_dir`: stages every change and commits it with
/// `message`. Returns `NothingToCommit` instead of an error when there's
/// nothing staged, since that's a normal outcome, not a failure.
pub fn commit(world_dir: &Path, message: &str) -> Result<CommitOutcome, GitError> {
    ensure_identity(world_dir)?;
    run(world_dir, &["add", "-A"])?;

    let status = run(world_dir, &["status", "--porcelain"])?;
    if status.stdout.is_empty() {
        return Ok(CommitOutcome::NothingToCommit);
    }

    run(world_dir, &["commit", "-m", message])?;
    let rev = run(world_dir, &["rev-parse", "HEAD"])?;
    Ok(CommitOutcome::Created(
        String::from_utf8_lossy(&rev.stdout).trim().to_string(),
    ))
}

/// One saved snapshot, as read from `git log`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub hash: String,
    pub date: String,
    pub message: String,
}

/// Lists every snapshot saved for `world_dir`, most recent first. A world
/// that was never versioned, or that was versioned but never had a
/// snapshot saved yet, both return an empty list — neither case is an
/// error, they're just "no history yet".
pub fn log(world_dir: &Path) -> Result<Vec<Snapshot>, GitError> {
    if !is_repository(world_dir) {
        return Ok(Vec::new());
    }

    let output = match run(world_dir, &["log", "--pretty=format:%H\x1f%aI\x1f%s"]) {
        Ok(output) => output,
        Err(GitError::CommandFailed(stderr)) if stderr.contains("does not have any commits yet") => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(e),
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\x1f');
            let hash = fields.next()?.to_string();
            let date = fields.next()?.to_string();
            let message = fields.next()?.to_string();
            Some(Snapshot { hash, date, message })
        })
        .collect())
}

/// Whether `world_dir` looks like it's currently loaded in a Minecraft
/// client. Mirrors what the game itself does: while a world is open, it
/// holds an exclusive OS-level lock on `session.lock` inside the world
/// folder. If the file doesn't exist yet, the world was never opened, so
/// it can't be locked.
fn is_currently_open(world_dir: &Path) -> std::io::Result<bool> {
    let lock_path = world_dir.join("session.lock");
    if !lock_path.is_file() {
        return Ok(false);
    }

    let file = std::fs::OpenOptions::new().write(true).open(&lock_path)?;
    match file.try_lock() {
        Ok(()) => {
            file.unlock()?;
            Ok(false)
        }
        Err(std::fs::TryLockError::WouldBlock) => Ok(true),
        Err(std::fs::TryLockError::Error(e)) => Err(e),
    }
}

/// Outcome of a restore: whether the pre-restore safety checkpoint actually
/// created a commit, and whether bringing the files back to `commit_hash`
/// resulted in a new commit (it won't if the world was already in that
/// exact state).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub backup: CommitOutcome,
    pub restore: CommitOutcome,
}

/// Restores `world_dir` to the state it had at `commit_hash`. Never
/// destructive: it never rewrites or discards history. Instead it (1)
/// refuses if the world looks currently open in Minecraft, (2) saves
/// whatever's pending right now as a backup snapshot, (3) brings the files
/// back to the old state, and (4) records that as a new snapshot on top —
/// so restoring is itself always undoable by restoring again.
pub fn restore(world_dir: &Path, commit_hash: &str) -> Result<RestoreOutcome, RestoreError> {
    if is_currently_open(world_dir)? {
        return Err(RestoreError::WorldCurrentlyOpen);
    }

    let backup = commit(world_dir, "Backup before restoring")?;
    run(world_dir, &["checkout", commit_hash, "--", "."])?;

    let short = &commit_hash[..commit_hash.len().min(7)];
    let restore = commit(world_dir, &format!("Restored to {short}"))?;

    Ok(RestoreOutcome { backup, restore })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_git_dir() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-init-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        init(&world_dir).unwrap();

        let has_git_dir = world_dir.join(".git").is_dir();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(has_git_dir);
    }

    #[test]
    fn is_repository_reflects_init_state() {
        let world_dir =
            std::env::temp_dir().join(format!("mcgit-core-test-is-repository-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        let before = is_repository(&world_dir);
        init(&world_dir).unwrap();
        let after = is_repository(&world_dir);

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(!before);
        assert!(after);
    }

    #[test]
    fn init_twice_does_not_error() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-init-twice-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        init(&world_dir).unwrap();
        let second = init(&world_dir);

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(second.is_ok());
    }

    #[test]
    fn init_ignores_session_lock() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-init-gitignore-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        init(&world_dir).unwrap(); // idempotent: no duplicate lines

        let exclude = std::fs::read_to_string(world_dir.join(".git").join("info").join("exclude")).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        std::fs::write(world_dir.join("session.lock"), b"").unwrap();
        let outcome = commit(&world_dir, "First snapshot").unwrap();
        let tracked = Command::new("git")
            .args(["ls-files"])
            .current_dir(&world_dir)
            .output()
            .unwrap();
        let tracked_files = String::from_utf8_lossy(&tracked.stdout).into_owned();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(exclude.matches("session.lock").count(), 1);
        assert!(!tracked_files.contains("session.lock"));
        assert!(tracked_files.contains("level.dat"));
        assert!(matches!(outcome, CommitOutcome::Created(_)));
    }

    #[test]
    fn commit_with_changes_returns_created_hash() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-commit-created-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"fake world data").unwrap();

        let outcome = commit(&world_dir, "First snapshot").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        match outcome {
            CommitOutcome::Created(hash) => assert_eq!(hash.len(), 40),
            CommitOutcome::NothingToCommit => panic!("expected a commit to be created"),
        }
    }

    #[test]
    fn commit_again_with_no_changes_returns_nothing_to_commit() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-commit-repeat-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"fake world data").unwrap();
        commit(&world_dir, "First snapshot").unwrap();

        let second = commit(&world_dir, "Second snapshot").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(second, CommitOutcome::NothingToCommit);
    }

    #[test]
    fn commit_on_fresh_empty_repo_returns_nothing_to_commit() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-commit-empty-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();

        let outcome = commit(&world_dir, "Nothing here yet").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(outcome, CommitOutcome::NothingToCommit);
    }

    #[test]
    fn log_on_never_initialized_world_returns_empty() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-log-never-init-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        let history = log(&world_dir).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn log_on_repo_with_no_commits_returns_empty() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-log-no-commits-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();

        let history = log(&world_dir).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(history.is_empty());
    }

    #[test]
    fn log_returns_one_snapshot_with_full_hash() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-log-one-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"fake world data").unwrap();
        commit(&world_dir, "First snapshot").unwrap();

        let history = log(&world_dir).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].hash.len(), 40);
        assert_eq!(history[0].message, "First snapshot");
    }

    #[test]
    fn log_returns_snapshots_most_recent_first() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-log-two-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        commit(&world_dir, "First snapshot").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v2").unwrap();
        commit(&world_dir, "Second snapshot").unwrap();

        let history = log(&world_dir).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].message, "Second snapshot");
        assert_eq!(history[1].message, "First snapshot");
    }

    #[test]
    fn restore_brings_back_old_content_and_creates_a_new_commit() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-restore-basic-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        let first = commit(&world_dir, "First snapshot").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v2").unwrap();
        commit(&world_dir, "Second snapshot").unwrap();

        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };
        let outcome = restore(&world_dir, &first_hash).unwrap();

        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(content, b"v1");
        assert_eq!(outcome.backup, CommitOutcome::NothingToCommit);
        match outcome.restore {
            CommitOutcome::Created(_) => {}
            CommitOutcome::NothingToCommit => panic!("expected a restore commit"),
        }
    }

    #[test]
    fn restore_backs_up_pending_changes_first() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-restore-backup-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        let first = commit(&world_dir, "First snapshot").unwrap();
        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };
        // Pending, never-saved change.
        std::fs::write(world_dir.join("level.dat"), b"uncommitted").unwrap();

        let outcome = restore(&world_dir, &first_hash).unwrap();

        let history = log(&world_dir).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();
        match outcome.backup {
            CommitOutcome::Created(_) => {}
            CommitOutcome::NothingToCommit => panic!("expected the pending change to be backed up"),
        }
        assert_eq!(history.len(), 3); // first snapshot, backup, restore
        assert!(history[0].message.starts_with("Restored to "));
        assert_eq!(history[1].message, "Backup before restoring");
        assert_eq!(history[2].message, "First snapshot");
    }

    #[test]
    fn restore_to_current_state_creates_no_new_commits() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-restore-noop-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        let first = commit(&world_dir, "First snapshot").unwrap();
        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        let outcome = restore(&world_dir, &first_hash).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(outcome.backup, CommitOutcome::NothingToCommit);
        assert_eq!(outcome.restore, CommitOutcome::NothingToCommit);
    }

    #[test]
    fn restore_fails_with_invalid_hash() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-restore-bad-hash-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        commit(&world_dir, "First snapshot").unwrap();

        let result = restore(&world_dir, "0000000000000000000000000000000000000");

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(RestoreError::Git(_))));
    }

    #[test]
    fn restore_refuses_when_world_is_locked() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-restore-locked-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        let first = commit(&world_dir, "First snapshot").unwrap();
        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        // Simulate Minecraft having the world open, same as the real game does.
        let lock_file = std::fs::File::create(world_dir.join("session.lock")).unwrap();
        lock_file.lock().unwrap();

        let result = restore(&world_dir, &first_hash);
        let history_len = log(&world_dir).unwrap().len();

        drop(lock_file);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(RestoreError::WorldCurrentlyOpen)));
        assert_eq!(history_len, 1, "restore must not touch history when the world is open");
    }
}
