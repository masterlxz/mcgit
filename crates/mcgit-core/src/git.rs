use std::path::Path;
use std::process::Command;

use crate::types::GitError;

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
}
