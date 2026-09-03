use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::types::{BranchError, DeleteError, GitError, MergeError, RestoreError};

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
/// `commit_name`/`commit_email` come from the caller (resolved from the
/// `mcgit-db` `settings` table, defaulting to `mcgit`/`mcgit@localhost` —
/// this crate itself stays free of any DB dependency, see `Cargo.toml`).
fn ensure_identity(world_dir: &Path, commit_name: &str, commit_email: &str) -> Result<(), GitError> {
    run(world_dir, &["config", "--local", "user.name", commit_name])?;
    run(world_dir, &["config", "--local", "user.email", commit_email])?;
    Ok(())
}

/// Saves a snapshot of `world_dir`: stages every change and commits it with
/// `message`. Returns `NothingToCommit` instead of an error when there's
/// nothing staged, since that's a normal outcome, not a failure.
pub fn commit(
    world_dir: &Path,
    message: &str,
    commit_name: &str,
    commit_email: &str,
) -> Result<CommitOutcome, GitError> {
    ensure_identity(world_dir, commit_name, commit_email)?;
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
pub fn restore(
    world_dir: &Path,
    commit_hash: &str,
    commit_name: &str,
    commit_email: &str,
) -> Result<RestoreOutcome, RestoreError> {
    if is_currently_open(world_dir)? {
        return Err(RestoreError::WorldCurrentlyOpen);
    }

    let backup = commit(world_dir, "Backup before restoring", commit_name, commit_email)?;
    run(world_dir, &["checkout", commit_hash, "--", "."])?;

    let short = &commit_hash[..commit_hash.len().min(7)];
    let restore = commit(world_dir, &format!("Restored to {short}"), commit_name, commit_email)?;

    Ok(RestoreOutcome { backup, restore })
}

fn trimmed_stdout(output: std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A commit's full tree state and metadata, as needed to recreate it
/// unchanged under a different parent.
struct Descendant {
    tree: String,
    author_date: String,
    committer_date: String,
    message: String,
}

/// Creates a new commit that reuses `tree` exactly (same file content as
/// whatever commit it was copied from) under `parent` (or no parent at
/// all, for a new root), preserving the original author/committer dates so
/// a surviving snapshot's date in the timeline never shifts just because
/// an unrelated one was deleted.
fn commit_tree_with_dates(
    world_dir: &Path,
    tree: &str,
    parent: Option<&str>,
    message: &str,
    author_date: &str,
    committer_date: &str,
) -> Result<String, GitError> {
    let mut args: Vec<&str> = vec!["commit-tree", tree];
    if let Some(parent) = parent {
        args.push("-p");
        args.push(parent);
    }
    args.push("-m");
    args.push(message);

    let output = Command::new("git")
        .args(&args)
        .current_dir(world_dir)
        .env("GIT_AUTHOR_DATE", author_date)
        .env("GIT_COMMITTER_DATE", committer_date)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    Ok(trimmed_stdout(output))
}

/// Deletes `commit_hash` from `world_dir`'s history — the only truly
/// destructive operation in the Git Engine. Never uses `rebase`/`merge`:
/// every surviving commit after the deleted one is rebuilt with
/// `git commit-tree`, reusing its exact original tree (a commit is a full
/// snapshot, not a diff), so there is nothing for Git to reconcile and no
/// conflict is possible even for binary world files. If the deleted commit
/// was the current tip, the world's files are reset to match the new tip.
pub fn delete_snapshot(world_dir: &Path, commit_hash: &str) -> Result<(), DeleteError> {
    if is_currently_open(world_dir)? {
        return Err(DeleteError::WorldCurrentlyOpen);
    }

    let head = trimmed_stdout(run(world_dir, &["rev-parse", "HEAD"])?);
    let parent = trimmed_stdout(run(world_dir, &["log", "--format=%P", "-1", commit_hash])?);

    let range = format!("{commit_hash}..HEAD");
    let descendants_output = run(
        world_dir,
        &["log", "--reverse", "--format=%T\x1f%aI\x1f%cI\x1f%s", &range],
    )?;
    let descendants: Vec<Descendant> = String::from_utf8_lossy(&descendants_output.stdout)
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(4, '\x1f');
            Some(Descendant {
                tree: fields.next()?.to_string(),
                author_date: fields.next()?.to_string(),
                committer_date: fields.next()?.to_string(),
                message: fields.next()?.to_string(),
            })
        })
        .collect();

    let mut new_parent = if parent.is_empty() { None } else { Some(parent) };
    for d in &descendants {
        let tip = commit_tree_with_dates(
            world_dir,
            &d.tree,
            new_parent.as_deref(),
            &d.message,
            &d.author_date,
            &d.committer_date,
        )?;
        new_parent = Some(tip);
    }

    let branch = trimmed_stdout(run(world_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?);
    let ref_name = format!("refs/heads/{branch}");
    match &new_parent {
        Some(tip) => {
            run(world_dir, &["update-ref", &ref_name, tip])?;
        }
        None => {
            run(world_dir, &["update-ref", "-d", &ref_name])?;
        }
    }

    // The tree at HEAD only actually changes if the deleted commit was the
    // tip itself — every other case leaves HEAD's own tree untouched, so
    // the working directory already matches reality.
    if commit_hash == head && new_parent.is_some() {
        run(world_dir, &["reset", "--hard"])?;
    }

    Ok(())
}

/// The name of the branch `world_dir`'s HEAD currently points to. Works even
/// on a repository with no commits yet, since it just reads HEAD's symbolic
/// ref name, not anything from the commit graph.
pub fn current_branch(world_dir: &Path) -> Result<String, GitError> {
    Ok(trimmed_stdout(run(world_dir, &["branch", "--show-current"])?))
}

/// Every branch that exists in `world_dir`'s repository.
pub fn list_branches(world_dir: &Path) -> Result<Vec<String>, GitError> {
    let output = run(world_dir, &["branch", "--format=%(refname:short)"])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect())
}

/// Creates `name` as a new branch and switches to it in one step, at
/// whatever commit `world_dir` is currently on. No safety checkpoint and no
/// open-world check are needed here: the new branch starts out pointing at
/// the exact same commit as the current one, so no file on disk changes —
/// unlike `switch_branch`, which moves to a branch that can have different
/// content. Git validates `name` itself; an invalid name surfaces as
/// `GitError::CommandFailed` with Git's own message.
pub fn create_branch(world_dir: &Path, name: &str) -> Result<(), GitError> {
    run(world_dir, &["checkout", "-b", name])?;
    Ok(())
}

/// Outcome of switching branches: whether pending changes on the branch
/// being left had to be checkpointed first, and which branch is now current.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitchOutcome {
    pub checkpoint: CommitOutcome,
    pub branch: String,
}

/// Switches `world_dir` to branch `name`. Refuses if the world looks
/// currently open in Minecraft, same guard as `restore`/`delete_snapshot` —
/// switching branches can change file content on disk, just like those do.
/// Before switching, always saves whatever's pending on the current branch
/// as a checkpoint, so a switch never fails because of "local changes would
/// be overwritten" and never silently carries unrelated work onto the
/// target branch.
pub fn switch_branch(
    world_dir: &Path,
    name: &str,
    commit_name: &str,
    commit_email: &str,
) -> Result<SwitchOutcome, BranchError> {
    if is_currently_open(world_dir)? {
        return Err(BranchError::WorldCurrentlyOpen);
    }

    let checkpoint = commit(
        world_dir,
        "Checkpoint before switching branches",
        commit_name,
        commit_email,
    )?;
    run(world_dir, &["checkout", name])?;

    Ok(SwitchOutcome {
        checkpoint,
        branch: name.to_string(),
    })
}

/// Whether a changed file was added, modified, or deleted between two refs.
/// Rename detection is deliberately not enabled (no `-M`, no `diff.renames`
/// config), so a renamed file shows up as a `Deleted` + `Added` pair instead
/// of its own variant — simpler to reason about than reconstructing renames,
/// and good enough for a file-level summary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeStatus {
    Added,
    Modified,
    Deleted,
}

/// One file that differs between two refs, with its size in bytes on each
/// side (`None` when the file doesn't exist on that side, i.e. it was added
/// or deleted). No attempt is made to diff *content* — most world files
/// (`.mca`, `level.dat`) are binary, so a line-level diff wouldn't mean
/// anything; a real content-aware diff is Fase 4's job, which actually
/// interprets the format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileChange {
    pub path: String,
    pub status: ChangeStatus,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
}

/// The size in bytes of `path` as it exists at `git_ref`, via
/// `git cat-file -s <git_ref>:<path>` — one small, single-purpose command
/// rather than parsing `git diff --stat`'s human-formatted "Bin X -> Y
/// bytes" text.
fn blob_size(world_dir: &Path, git_ref: &str, path: &str) -> Result<u64, GitError> {
    let output = run(world_dir, &["cat-file", "-s", &format!("{git_ref}:{path}")])?;
    trimmed_stdout(output)
        .parse()
        .map_err(|_| GitError::CommandFailed(format!("unexpected `git cat-file -s` output for {path}")))
}

/// The raw bytes of `path` as it exists at `git_ref`, via
/// `git cat-file -p <git_ref>:<path>` — unlike `blob_size`, this is used
/// when the actual binary content is needed (e.g. to parse a region file),
/// so no text conversion is applied to the output.
fn blob_contents(world_dir: &Path, git_ref: &str, path: &str) -> Result<Vec<u8>, GitError> {
    let output = Command::new("git")
        .args(["cat-file", "-p", &format!("{git_ref}:{path}")])
        .current_dir(world_dir)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }
    Ok(output.stdout)
}

/// Diffs the chunks of a region file (`path`, e.g. `"region/r.0.0.mca"`)
/// between `from` and `to` — which 16×16 chunks were added, removed, or
/// changed, at their absolute world coordinates. See `mcgit_world` for what
/// "changed" means here (byte-level, not content-aware yet).
pub fn diff_region_chunks(
    world_dir: &Path,
    from: &str,
    to: &str,
    path: &str,
) -> Result<Vec<mcgit_world::ChunkDiff>, GitError> {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let (region_x, region_z) = mcgit_world::parse_region_coords(filename)
        .ok_or_else(|| GitError::CommandFailed(format!("not a region file path: {path}")))?;

    let from_bytes = blob_contents(world_dir, from, path)?;
    let to_bytes = blob_contents(world_dir, to, path)?;

    mcgit_world::diff_region_chunks(&from_bytes, &to_bytes, region_x, region_z)
        .map_err(|e| GitError::CommandFailed(e.to_string()))
}

/// Diffs one chunk's blocks between `from` and `to` — which blocks (by
/// absolute world position) actually changed, and to/from what. `path` is
/// the region file the chunk lives in (e.g. `"region/r.0.0.mca"`);
/// `chunk_x`/`chunk_z` are the chunk's *absolute* coordinates, the same ones
/// `diff_region_chunks` reports for a chunk marked `Changed`.
pub fn diff_chunk_blocks(
    world_dir: &Path,
    from: &str,
    to: &str,
    path: &str,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<Vec<mcgit_world::BlockDiff>, GitError> {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let (region_x, region_z) = mcgit_world::parse_region_coords(filename)
        .ok_or_else(|| GitError::CommandFailed(format!("not a region file path: {path}")))?;
    let local_x = (chunk_x - region_x * 32) as usize;
    let local_z = (chunk_z - region_z * 32) as usize;

    let from_chunk = read_chunk_nbt(world_dir, from, path, local_x, local_z)?;
    let to_chunk = read_chunk_nbt(world_dir, to, path, local_x, local_z)?;

    mcgit_world::diff_chunk_blocks(&from_chunk, &to_chunk, chunk_x, chunk_z)
        .map_err(|e| GitError::CommandFailed(e.to_string()))
}

/// Diffs one chunk's entities (mobs, dropped items, ...) between `from` and
/// `to`, by `UUID` — which ones appeared and which disappeared. `path` is
/// the region file the chunk lives in, but here it's an `entities/` file
/// (e.g. `"entities/r.0.0.mca"`), not `region/` — a different folder with a
/// different per-chunk NBT shape (see `mcgit_world::diff_chunk_entities`),
/// though the same `r.<x>.<z>.mca` naming and 32×32 layout, so
/// `read_chunk_nbt` works unchanged.
pub fn diff_chunk_entities(
    world_dir: &Path,
    from: &str,
    to: &str,
    path: &str,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<Vec<mcgit_world::EntityDiff>, GitError> {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let (region_x, region_z) = mcgit_world::parse_region_coords(filename)
        .ok_or_else(|| GitError::CommandFailed(format!("not a region file path: {path}")))?;
    let local_x = (chunk_x - region_x * 32) as usize;
    let local_z = (chunk_z - region_z * 32) as usize;

    let from_chunk = read_chunk_nbt(world_dir, from, path, local_x, local_z)?;
    let to_chunk = read_chunk_nbt(world_dir, to, path, local_x, local_z)?;

    mcgit_world::diff_chunk_entities(&from_chunk, &to_chunk).map_err(|e| GitError::CommandFailed(e.to_string()))
}

/// Diffs one chunk's generated structures between `from` and `to`, by
/// structure id — which types started or stopped being recorded as
/// starting there. `path` is a `region/` file, same folder `diff_chunk_blocks`
/// reads, since a chunk's `structures.starts` lives alongside its blocks.
pub fn diff_chunk_structures(
    world_dir: &Path,
    from: &str,
    to: &str,
    path: &str,
    chunk_x: i32,
    chunk_z: i32,
) -> Result<Vec<mcgit_world::StructureDiff>, GitError> {
    let filename = path.rsplit('/').next().unwrap_or(path);
    let (region_x, region_z) = mcgit_world::parse_region_coords(filename)
        .ok_or_else(|| GitError::CommandFailed(format!("not a region file path: {path}")))?;
    let local_x = (chunk_x - region_x * 32) as usize;
    let local_z = (chunk_z - region_z * 32) as usize;

    let from_chunk = read_chunk_nbt(world_dir, from, path, local_x, local_z)?;
    let to_chunk = read_chunk_nbt(world_dir, to, path, local_x, local_z)?;

    mcgit_world::diff_chunk_structures(&from_chunk, &to_chunk).map_err(|e| GitError::CommandFailed(e.to_string()))
}

/// Fetches `path` (a region file) as it exists at `git_ref` and pulls out
/// the raw NBT bytes of one chunk (local `0..32` coordinates within that
/// region) from it.
fn read_chunk_nbt(
    world_dir: &Path,
    git_ref: &str,
    path: &str,
    local_x: usize,
    local_z: usize,
) -> Result<Vec<u8>, GitError> {
    let region_bytes = blob_contents(world_dir, git_ref, path)?;
    let mut region = fastanvil::Region::from_stream(std::io::Cursor::new(region_bytes))
        .map_err(|e| GitError::CommandFailed(e.to_string()))?;
    region
        .read_chunk(local_x, local_z)
        .map_err(|e| GitError::CommandFailed(e.to_string()))?
        .ok_or_else(|| GitError::CommandFailed(format!("chunk ({local_x},{local_z}) not found in {path}@{git_ref}")))
}

/// Lists every file under `prefix` as it exists at `git_ref` (e.g. `"region/"`
/// -> every `.mca` file, `entities/`, `poi/`, ...), via
/// `git ls-tree -r --name-only <ref> -- <prefix>`.
fn list_files(world_dir: &Path, git_ref: &str, prefix: &str) -> Result<Vec<String>, GitError> {
    let output = run(world_dir, &["ls-tree", "-r", "--name-only", git_ref, "--", prefix])?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// Aggregates block counts across every region file in `region/`, for the
/// world as it existed at `git_ref` — a single snapshot's totals, not a diff
/// between two. Sorted most common block first (ties broken by name, for a
/// stable order). Only the `region/` folder — not `DIM-1/`/`DIM1/`
/// (Nether/End) or `entities/`/`poi/` — same scope as `diff_region_chunks`.
pub fn world_block_stats(world_dir: &Path, git_ref: &str) -> Result<Vec<(String, u64)>, GitError> {
    let paths = list_files(world_dir, git_ref, "region/")?;
    aggregate_region_stats(world_dir, git_ref, paths, |bytes| {
        mcgit_world::count_region_blocks(bytes).map_err(|e| GitError::CommandFailed(e.to_string()))
    })
}

/// Aggregates structure counts (by type, e.g. `"minecraft:trial_chambers"`)
/// across every region file in `region/`, for the world as it existed at
/// `git_ref` — same folder `world_block_stats` reads, since a chunk's
/// generated-structure data lives alongside its blocks. Each structure
/// instance counts once (see `count_chunk_structures`), sorted most common
/// first.
pub fn world_structure_stats(world_dir: &Path, git_ref: &str) -> Result<Vec<(String, u64)>, GitError> {
    let paths = list_files(world_dir, git_ref, "region/")?;
    aggregate_region_stats(world_dir, git_ref, paths, |bytes| {
        mcgit_world::count_region_structures(bytes).map_err(|e| GitError::CommandFailed(e.to_string()))
    })
}

/// Aggregates entity counts (by `id`, e.g. `"minecraft:sheep"`) across every
/// region file in `entities/`, for the world as it existed at `git_ref` —
/// mobs and dropped items live in their own folder, separate from `region/`
/// (see `count_chunk_entities`). Sorted most common first.
pub fn world_entity_stats(world_dir: &Path, git_ref: &str) -> Result<Vec<(String, u64)>, GitError> {
    let paths = list_files(world_dir, git_ref, "entities/")?;
    aggregate_region_stats(world_dir, git_ref, paths, |bytes| {
        mcgit_world::count_region_entities(bytes).map_err(|e| GitError::CommandFailed(e.to_string()))
    })
}

/// Shared plumbing behind `world_block_stats`/`world_structure_stats`/
/// `world_entity_stats`: fetch each file's blob contents, hand it to
/// `count_one_region`, and sum the resulting per-name counts across all of
/// them — sorted most common first (ties broken by name, for a stable
/// order).
fn aggregate_region_stats(
    world_dir: &Path,
    git_ref: &str,
    paths: Vec<String>,
    count_one_region: impl Fn(&[u8]) -> Result<HashMap<String, u64>, GitError>,
) -> Result<Vec<(String, u64)>, GitError> {
    let mut totals: HashMap<String, u64> = HashMap::new();
    for path in paths {
        let bytes = blob_contents(world_dir, git_ref, &path)?;
        let counts = count_one_region(&bytes)?;
        for (name, count) in counts {
            *totals.entry(name).or_insert(0) += count;
        }
    }

    let mut stats: Vec<(String, u64)> = totals.into_iter().collect();
    stats.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    Ok(stats)
}

/// Lists every file that differs between `from` and `to`, with its size on
/// each side. File-level only — see `FileChange` for why there's no content
/// diff here.
pub fn diff_branches(world_dir: &Path, from: &str, to: &str) -> Result<Vec<FileChange>, GitError> {
    let output = run(world_dir, &["diff", "--name-status", from, to])?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut changes = Vec::new();
    for line in stdout.lines() {
        let mut fields = line.splitn(2, '\t');
        let (Some(code), Some(path)) = (fields.next(), fields.next()) else {
            continue;
        };
        let status = match code {
            "A" => ChangeStatus::Added,
            "M" => ChangeStatus::Modified,
            "D" => ChangeStatus::Deleted,
            _ => continue,
        };

        let old_size = match status {
            ChangeStatus::Added => None,
            _ => Some(blob_size(world_dir, from, path)?),
        };
        let new_size = match status {
            ChangeStatus::Deleted => None,
            _ => Some(blob_size(world_dir, to, path)?),
        };

        changes.push(FileChange {
            path: path.to_string(),
            status,
            old_size,
            new_size,
        });
    }

    Ok(changes)
}

/// Whether `world_dir` currently has a merge in progress (started but not
/// yet finished or aborted).
fn is_merge_in_progress(world_dir: &Path) -> bool {
    world_dir.join(".git").join("MERGE_HEAD").is_file()
}

/// Previews merging `to` into `from` without touching the working tree or
/// the index at all — `git merge-tree --write-tree` is a pure read-only
/// operation. Returns the paths that would conflict; an empty list means
/// the merge would be clean. Doesn't go through `run()`: a non-zero exit
/// here just means "would conflict", not a failure.
pub fn preview_merge(world_dir: &Path, from: &str, to: &str) -> Result<Vec<String>, GitError> {
    let output = Command::new("git")
        .args(["merge-tree", "--write-tree", from, to])
        .current_dir(world_dir)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.trim().is_empty() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    // First line is the resulting tree hash (only meaningful on a clean
    // merge, unused here). Conflicting entries look like
    // `<mode> <oid> <stage>\t<path>`; the trailing human-readable
    // "Auto-merging"/"CONFLICT" lines don't match that shape and are
    // skipped.
    let mut paths: Vec<String> = Vec::new();
    for line in stdout.lines().skip(1) {
        let mut fields = line.splitn(2, '\t');
        let (Some(prefix), Some(path)) = (fields.next(), fields.next()) else {
            continue;
        };
        if prefix.split(' ').count() != 3 {
            continue;
        }
        if !paths.iter().any(|p| p == path) {
            paths.push(path.to_string());
        }
    }
    Ok(paths)
}

/// Which side of a merge conflict is missing a version of a file, as
/// opposed to both sides having genuinely different content.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    BothModified,
    DeletedByUs,
    DeletedByThem,
}

/// One file with an unresolved conflict during an in-progress merge.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictedFile {
    pub path: String,
    pub kind: ConflictKind,
}

/// Lists every unresolved conflict in `world_dir`, via `git status
/// --porcelain=v1`'s XY status codes for unmerged paths.
pub fn list_merge_conflicts(world_dir: &Path) -> Result<Vec<ConflictedFile>, GitError> {
    let output = run(world_dir, &["status", "--porcelain=v1"])?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    Ok(stdout
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }
            let kind = match &line[0..2] {
                "UD" => ConflictKind::DeletedByThem,
                "DU" => ConflictKind::DeletedByUs,
                "UU" | "AA" | "AU" | "UA" => ConflictKind::BothModified,
                _ => return None,
            };
            Some(ConflictedFile {
                path: line[3..].to_string(),
                kind,
            })
        })
        .collect())
}

/// Outcome of attempting to merge another branch into the current one:
/// either a clean merge commit, or the set of files that need resolving
/// before the merge can be finished.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MergeOutcome {
    Merged(String),
    ConflictsPending(Vec<ConflictedFile>),
}

/// Merges `other` into `world_dir`'s current branch. Refuses if the world
/// looks currently open in Minecraft (a merge can rewrite files on disk,
/// same risk class as `restore`/`switch_branch`), or if a previous merge is
/// still unresolved — Git itself refuses a second `merge` in that state
/// with a confusing generic error ("Exiting because of an unresolved
/// conflict"), caught here with a clearer one instead.
pub fn merge_branch(
    world_dir: &Path,
    other: &str,
    commit_name: &str,
    commit_email: &str,
) -> Result<MergeOutcome, MergeError> {
    if is_currently_open(world_dir)? {
        return Err(MergeError::WorldCurrentlyOpen);
    }
    if is_merge_in_progress(world_dir) {
        return Err(MergeError::AlreadyInProgress);
    }

    ensure_identity(world_dir, commit_name, commit_email)?;
    let output = Command::new("git")
        .args(["merge", other, "-m", &format!("Merge branch '{other}'")])
        .current_dir(world_dir)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    if output.status.success() {
        let rev = run(world_dir, &["rev-parse", "HEAD"])?;
        return Ok(MergeOutcome::Merged(trimmed_stdout(rev)));
    }

    // A real conflict leaves MERGE_HEAD behind; anything else (e.g. a bad
    // ref name) doesn't, and is a genuine error worth surfacing as such.
    if is_merge_in_progress(world_dir) {
        return Ok(MergeOutcome::ConflictsPending(list_merge_conflicts(world_dir)?));
    }
    Err(MergeError::Git(GitError::CommandFailed(
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )))
}

/// Which side of a conflict to keep when resolving it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Ours,
    Theirs,
}

/// Resolves one conflicted file during an in-progress merge by keeping
/// `keep`'s version. For a content conflict this is a plain `git checkout
/// --ours`/`--theirs`; for a modify/delete conflict where `keep` is the
/// side that deleted the file, there's no version to check out (Git
/// refuses with "does not have our/their version") — resolving there means
/// accepting the deletion via `git rm` instead.
pub fn resolve_conflict(world_dir: &Path, path: &str, keep: Side) -> Result<(), MergeError> {
    if is_currently_open(world_dir)? {
        return Err(MergeError::WorldCurrentlyOpen);
    }

    let flag = match keep {
        Side::Ours => "--ours",
        Side::Theirs => "--theirs",
    };
    let output = Command::new("git")
        .args(["checkout", flag, "--", path])
        .current_dir(world_dir)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    if output.status.success() {
        run(world_dir, &["add", "--", path])?;
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("does not have our version") || stderr.contains("does not have their version") {
            run(world_dir, &["rm", "--", path])?;
        } else {
            return Err(MergeError::Git(GitError::CommandFailed(stderr.into_owned())));
        }
    }

    Ok(())
}

/// Finishes an in-progress merge once every conflict has been resolved
/// (staged via `resolve_conflict`). Doesn't `add -A` — only what was
/// explicitly resolved should go into the merge commit.
pub fn finish_merge(
    world_dir: &Path,
    message: &str,
    commit_name: &str,
    commit_email: &str,
) -> Result<String, GitError> {
    ensure_identity(world_dir, commit_name, commit_email)?;
    run(world_dir, &["commit", "-m", message])?;
    let rev = run(world_dir, &["rev-parse", "HEAD"])?;
    Ok(trimmed_stdout(rev))
}

/// Aborts an in-progress merge, restoring the world exactly to how it was
/// before the merge started.
pub fn abort_merge(world_dir: &Path) -> Result<(), GitError> {
    run(world_dir, &["merge", "--abort"])?;
    Ok(())
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
        let outcome = commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();
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

        let outcome = commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();

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
        commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();

        let second = commit(&world_dir, "Second snapshot", "Tester", "tester@example.com").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(second, CommitOutcome::NothingToCommit);
    }

    #[test]
    fn commit_on_fresh_empty_repo_returns_nothing_to_commit() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-commit-empty-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();

        let outcome = commit(&world_dir, "Nothing here yet", "Tester", "tester@example.com").unwrap();

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
        commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();

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
        commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v2").unwrap();
        commit(&world_dir, "Second snapshot", "Tester", "tester@example.com").unwrap();

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
        let first = commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v2").unwrap();
        commit(&world_dir, "Second snapshot", "Tester", "tester@example.com").unwrap();

        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };
        let outcome = restore(&world_dir, &first_hash, "Tester", "tester@example.com").unwrap();

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
        let first = commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();
        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };
        // Pending, never-saved change.
        std::fs::write(world_dir.join("level.dat"), b"uncommitted").unwrap();

        let outcome = restore(&world_dir, &first_hash, "Tester", "tester@example.com").unwrap();

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
        let first = commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();
        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        let outcome = restore(&world_dir, &first_hash, "Tester", "tester@example.com").unwrap();

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
        commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();

        let result = restore(&world_dir, "0000000000000000000000000000000000000", "Tester", "tester@example.com");

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(RestoreError::Git(_))));
    }

    #[test]
    fn restore_refuses_when_world_is_locked() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-restore-locked-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"v1").unwrap();
        let first = commit(&world_dir, "First snapshot", "Tester", "tester@example.com").unwrap();
        let first_hash = match first {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        // Simulate Minecraft having the world open, same as the real game does.
        let lock_file = std::fs::File::create(world_dir.join("session.lock")).unwrap();
        lock_file.lock().unwrap();

        let result = restore(&world_dir, &first_hash, "Tester", "tester@example.com");
        let history_len = log(&world_dir).unwrap().len();

        drop(lock_file);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(RestoreError::WorldCurrentlyOpen)));
        assert_eq!(history_len, 1, "restore must not touch history when the world is open");
    }

    #[test]
    fn delete_middle_commit_preserves_descendant_content_and_date() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-delete-middle-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"b").unwrap();
        let b = match commit(&world_dir, "B", "Tester", "tester@example.com").unwrap() {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };
        std::fs::write(world_dir.join("level.dat"), b"c").unwrap();
        commit(&world_dir, "C", "Tester", "tester@example.com").unwrap();

        let before = log(&world_dir).unwrap();
        let c_date_before = before[0].date.clone();

        delete_snapshot(&world_dir, &b).unwrap();

        let after = log(&world_dir).unwrap();
        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();

        assert_eq!(after.len(), 2);
        assert_eq!(after[0].message, "C");
        assert_eq!(after[0].date, c_date_before, "surviving commit's date must not shift");
        assert_eq!(after[1].message, "A");
        assert_eq!(content, b"c", "working directory must stay untouched");
    }

    #[test]
    fn delete_tip_resets_world_files() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-delete-tip-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"b").unwrap();
        let b = match commit(&world_dir, "B", "Tester", "tester@example.com").unwrap() {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        delete_snapshot(&world_dir, &b).unwrap();

        let history = log(&world_dir).unwrap();
        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].message, "A");
        assert_eq!(content, b"a", "deleting the tip must reset files to the new tip's content");
    }

    #[test]
    fn delete_root_with_descendants_creates_new_parentless_root() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-delete-root-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        let a = match commit(&world_dir, "A", "Tester", "tester@example.com").unwrap() {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };
        std::fs::write(world_dir.join("level.dat"), b"b").unwrap();
        commit(&world_dir, "B", "Tester", "tester@example.com").unwrap();

        delete_snapshot(&world_dir, &a).unwrap();

        let history = log(&world_dir).unwrap();
        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        let parents = run(&world_dir, &["log", "--format=%P", "-1", &history[0].hash]).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].message, "B");
        assert_eq!(content, b"b");
        assert!(String::from_utf8_lossy(&parents.stdout).trim().is_empty(), "new root must have no parent");
    }

    #[test]
    fn delete_only_commit_leaves_repo_with_empty_history() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-delete-only-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"only").unwrap();
        let only = match commit(&world_dir, "Only", "Tester", "tester@example.com").unwrap() {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        delete_snapshot(&world_dir, &only).unwrap();

        let history = log(&world_dir).unwrap();
        let still_a_repo = is_repository(&world_dir);
        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();

        assert!(history.is_empty());
        assert!(still_a_repo);
        assert_eq!(content, b"only", "files on disk are untouched by deleting the only commit");
    }

    #[test]
    fn delete_fails_with_invalid_hash() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-delete-bad-hash-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();

        let result = delete_snapshot(&world_dir, "0000000000000000000000000000000000000");

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(DeleteError::Git(_))));
    }

    #[test]
    fn delete_refuses_when_world_is_locked() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-delete-locked-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        let a = match commit(&world_dir, "A", "Tester", "tester@example.com").unwrap() {
            CommitOutcome::Created(hash) => hash,
            CommitOutcome::NothingToCommit => panic!("expected a commit"),
        };

        let lock_file = std::fs::File::create(world_dir.join("session.lock")).unwrap();
        lock_file.lock().unwrap();

        let result = delete_snapshot(&world_dir, &a);
        let history_len = log(&world_dir).unwrap().len();

        drop(lock_file);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(DeleteError::WorldCurrentlyOpen)));
        assert_eq!(history_len, 1, "delete must not touch history when the world is open");
    }

    #[test]
    fn create_branch_switches_to_new_branch() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-branch-create-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let original = current_branch(&world_dir).unwrap();

        create_branch(&world_dir, "experiment").unwrap();

        let now = current_branch(&world_dir).unwrap();
        let branches = list_branches(&world_dir).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(now, "experiment");
        assert!(branches.contains(&original));
        assert!(branches.contains(&"experiment".to_string()));
    }

    #[test]
    fn create_branch_works_on_repo_with_no_commits() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-branch-create-empty-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();

        create_branch(&world_dir, "experiment").unwrap();

        let now = current_branch(&world_dir).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(now, "experiment");
    }

    #[test]
    fn switch_branch_creates_checkpoint_when_pending_changes() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-branch-switch-checkpoint-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"main-content").unwrap();
        commit(&world_dir, "Main content", "Tester", "tester@example.com").unwrap();
        let original = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"experiment-content").unwrap();
        commit(&world_dir, "Experiment content", "Tester", "tester@example.com").unwrap();
        // Pending, never-saved change on the branch we're about to leave.
        std::fs::write(world_dir.join("level.dat"), b"uncommitted").unwrap();

        let outcome = switch_branch(&world_dir, &original, "Tester", "tester@example.com").unwrap();

        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        let history = log(&world_dir).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(outcome.branch, original);
        match outcome.checkpoint {
            CommitOutcome::Created(_) => {}
            CommitOutcome::NothingToCommit => panic!("expected the pending change to be checkpointed"),
        }
        assert_eq!(content, b"main-content", "must reflect the target branch's content");
        assert_eq!(history[0].message, "Main content");
    }

    #[test]
    fn switch_branch_no_checkpoint_when_clean() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-branch-switch-clean-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let original = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let outcome = switch_branch(&world_dir, &original, "Tester", "tester@example.com").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(outcome.checkpoint, CommitOutcome::NothingToCommit);
    }

    #[test]
    fn switch_branch_refuses_when_world_is_locked() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-branch-switch-locked-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let original = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let lock_file = std::fs::File::create(world_dir.join("session.lock")).unwrap();
        lock_file.lock().unwrap();

        let result = switch_branch(&world_dir, &original, "Tester", "tester@example.com");
        let history_len = log(&world_dir).unwrap().len();

        drop(lock_file);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(BranchError::WorldCurrentlyOpen)));
        assert_eq!(history_len, 1, "switch must not touch history when the world is open");
    }

    #[test]
    fn list_branches_reflects_all_created_branches() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-branch-list-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let original = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment-1").unwrap();
        switch_branch(&world_dir, &original, "Tester", "tester@example.com").unwrap();
        create_branch(&world_dir, "experiment-2").unwrap();

        let branches = list_branches(&world_dir).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(branches.len(), 3);
        assert!(branches.contains(&original));
        assert!(branches.contains(&"experiment-1".to_string()));
        assert!(branches.contains(&"experiment-2".to_string()));
    }

    #[test]
    fn diff_branches_reports_added_file() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-diff-added-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("new_region.mca"), b"new file contents").unwrap();
        commit(&world_dir, "Add region", "Tester", "tester@example.com").unwrap();

        let changes = diff_branches(&world_dir, &main, "experiment").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "new_region.mca");
        assert_eq!(changes[0].status, ChangeStatus::Added);
        assert_eq!(changes[0].old_size, None);
        assert_eq!(changes[0].new_size, Some(17));
    }

    #[test]
    fn diff_branches_reports_modified_file() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-diff-modified-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"aaa").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"aaaaaaa").unwrap();
        commit(&world_dir, "Grow level.dat", "Tester", "tester@example.com").unwrap();

        let changes = diff_branches(&world_dir, &main, "experiment").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "level.dat");
        assert_eq!(changes[0].status, ChangeStatus::Modified);
        assert_eq!(changes[0].old_size, Some(3));
        assert_eq!(changes[0].new_size, Some(7));
    }

    #[test]
    fn diff_branches_reports_deleted_file() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-diff-deleted-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        std::fs::write(world_dir.join("old_region.mca"), b"gone soon").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::remove_file(world_dir.join("old_region.mca")).unwrap();
        commit(&world_dir, "Remove region", "Tester", "tester@example.com").unwrap();

        let changes = diff_branches(&world_dir, &main, "experiment").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "old_region.mca");
        assert_eq!(changes[0].status, ChangeStatus::Deleted);
        assert_eq!(changes[0].old_size, Some(9));
        assert_eq!(changes[0].new_size, None);
    }

    #[test]
    fn diff_branches_between_identical_branches_returns_empty() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-diff-empty-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"a").unwrap();
        commit(&world_dir, "A", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let changes = diff_branches(&world_dir, &main, "experiment").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(changes.is_empty());
    }

    #[test]
    fn preview_merge_reports_no_conflicts_for_non_overlapping_files() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-preview-clean-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("r.0.0.mca"), b"base").unwrap();
        std::fs::write(world_dir.join("r.1.0.mca"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("r.1.0.mca"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("r.0.0.mca"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();

        let conflicts = preview_merge(&world_dir, &main, "experiment").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(conflicts.is_empty());
    }

    #[test]
    fn preview_merge_reports_conflicting_file() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-preview-conflict-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();

        let conflicts = preview_merge(&world_dir, &main, "experiment").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(conflicts, vec!["level.dat".to_string()]);
    }

    #[test]
    fn merge_branch_succeeds_cleanly_when_no_conflict() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-merge-clean-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("r.0.0.mca"), b"base").unwrap();
        std::fs::write(world_dir.join("r.1.0.mca"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("r.1.0.mca"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("r.0.0.mca"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();

        let outcome = merge_branch(&world_dir, "experiment", "Tester", "tester@example.com").unwrap();

        let r00 = std::fs::read(world_dir.join("r.0.0.mca")).unwrap();
        let r10 = std::fs::read(world_dir.join("r.1.0.mca")).unwrap();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(outcome, MergeOutcome::Merged(_)));
        assert_eq!(r00, b"changed-on-main");
        assert_eq!(r10, b"changed-on-experiment");
    }

    #[test]
    fn merge_branch_returns_conflicts_pending_on_content_conflict() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-merge-conflict-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();

        let outcome = merge_branch(&world_dir, "experiment", "Tester", "tester@example.com").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        match outcome {
            MergeOutcome::ConflictsPending(conflicts) => {
                assert_eq!(conflicts.len(), 1);
                assert_eq!(conflicts[0].path, "level.dat");
                assert_eq!(conflicts[0].kind, ConflictKind::BothModified);
            }
            MergeOutcome::Merged(_) => panic!("expected a conflict"),
        }
    }

    #[test]
    fn merge_branch_refuses_when_already_in_progress() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-merge-already-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();
        merge_branch(&world_dir, "experiment", "Tester", "tester@example.com").unwrap(); // leaves a conflict in progress

        let result = merge_branch(&world_dir, "experiment", "Tester", "tester@example.com");

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(MergeError::AlreadyInProgress)));
    }

    #[test]
    fn merge_branch_refuses_when_world_is_locked() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-merge-locked-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();

        let lock_file = std::fs::File::create(world_dir.join("session.lock")).unwrap();
        lock_file.lock().unwrap();

        let result = merge_branch(&world_dir, "experiment", "Tester", "tester@example.com");

        drop(lock_file);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(matches!(result, Err(MergeError::WorldCurrentlyOpen)));
    }

    #[test]
    fn resolve_conflict_keep_ours_then_finish_merge() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-resolve-ours-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();
        merge_branch(&world_dir, "experiment", "Tester", "tester@example.com").unwrap();

        resolve_conflict(&world_dir, "level.dat", Side::Ours).unwrap();
        let merge_hash = finish_merge(&world_dir, "Merge branch 'experiment'", "Tester", "tester@example.com").unwrap();

        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        let parents = run(&world_dir, &["log", "--format=%P", "-1", &merge_hash]).unwrap();
        let still_in_progress = is_merge_in_progress(&world_dir);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(content, b"changed-on-main");
        assert_eq!(
            String::from_utf8_lossy(&parents.stdout).trim().split(' ').count(),
            2,
            "a finished merge must have two parents"
        );
        assert!(!still_in_progress);
    }

    #[test]
    fn resolve_conflict_on_modify_delete_keep_theirs_deletes_file() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-resolve-delete-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        std::fs::write(world_dir.join("r.0.0.mca"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::remove_file(world_dir.join("r.0.0.mca")).unwrap();
        commit(&world_dir, "Experiment deletes region", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("r.0.0.mca"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main modifies region", "Tester", "tester@example.com").unwrap();
        let outcome = merge_branch(&world_dir, "experiment", "Tester", "tester@example.com").unwrap();
        let kind = match outcome {
            MergeOutcome::ConflictsPending(conflicts) => conflicts[0].kind,
            MergeOutcome::Merged(_) => panic!("expected a conflict"),
        };
        assert_eq!(kind, ConflictKind::DeletedByThem);

        // "theirs" (experiment) deleted the file — keeping theirs means accepting the deletion.
        resolve_conflict(&world_dir, "r.0.0.mca", Side::Theirs).unwrap();
        finish_merge(&world_dir, "Merge branch 'experiment'", "Tester", "tester@example.com").unwrap();

        let exists = world_dir.join("r.0.0.mca").exists();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(!exists, "keeping the side that deleted the file must remove it");
    }

    #[test]
    fn abort_merge_restores_pre_merge_state() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-merge-abort-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        init(&world_dir).unwrap();
        std::fs::write(world_dir.join("level.dat"), b"base").unwrap();
        commit(&world_dir, "Base", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-experiment").unwrap();
        commit(&world_dir, "Experiment change", "Tester", "tester@example.com").unwrap();
        switch_branch(&world_dir, &main, "Tester", "tester@example.com").unwrap();
        std::fs::write(world_dir.join("level.dat"), b"changed-on-main").unwrap();
        commit(&world_dir, "Main change", "Tester", "tester@example.com").unwrap();
        merge_branch(&world_dir, "experiment", "Tester", "tester@example.com").unwrap();

        abort_merge(&world_dir).unwrap();

        let content = std::fs::read(world_dir.join("level.dat")).unwrap();
        let still_in_progress = is_merge_in_progress(&world_dir);
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(content, b"changed-on-main", "must match the pre-merge state exactly");
        assert!(!still_in_progress);
    }

    /// Builds a valid, minimal region file (via `fastanvil::Region::create`)
    /// with the given (local x, local z, marker) chunks written into it.
    fn build_region_bytes(chunks: &[(usize, usize, i64)]) -> Vec<u8> {
        use std::collections::HashMap;
        use std::io::Cursor;

        let mut region = fastanvil::Region::create(Cursor::new(Vec::new())).unwrap();
        for &(x, z, marker) in chunks {
            let mut map = HashMap::new();
            map.insert("InhabitedTime".to_string(), fastnbt::Value::Long(marker));
            let bytes = fastnbt::to_bytes(&fastnbt::Value::Compound(map)).unwrap();
            region.write_chunk(x, z, &bytes).unwrap();
        }
        region.into_inner().unwrap().into_inner()
    }

    #[test]
    fn diff_region_chunks_reports_only_the_chunk_that_actually_changed() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-region-diff-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("region")).unwrap();
        init(&world_dir).unwrap();

        let base = build_region_bytes(&[(0, 0, 1), (5, 5, 1)]);
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &base).unwrap();
        commit(&world_dir, "Base region", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let changed = build_region_bytes(&[(0, 0, 2), (5, 5, 1)]);
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &changed).unwrap();
        commit(&world_dir, "Change chunk (0,0)", "Tester", "tester@example.com").unwrap();

        let diffs = diff_region_chunks(&world_dir, &main, "experiment", "region/r.0.0.mca").unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].chunk_x, 0);
        assert_eq!(diffs[0].chunk_z, 0);
        assert_eq!(diffs[0].status, mcgit_world::ChunkStatus::Changed);
    }

    /// Builds a region with one chunk containing a single section (`Y=0`)
    /// whose every block is `stone_index` into `palette` — mirrors the
    /// single-entry-palette shape (no `data` array) when `palette.len() ==
    /// 1`, and a real bit-packed one otherwise.
    fn build_region_with_chunk_block(local_x: usize, local_z: usize, palette: &[&str], block_index: usize) -> Vec<u8> {
        use std::collections::HashMap;
        use std::io::Cursor;

        let palette_values: Vec<fastnbt::Value> = palette
            .iter()
            .map(|name| {
                let mut entry = HashMap::new();
                entry.insert("Name".to_string(), fastnbt::Value::String(name.to_string()));
                fastnbt::Value::Compound(entry)
            })
            .collect();

        let mut block_states = HashMap::new();
        block_states.insert("palette".to_string(), fastnbt::Value::List(palette_values));
        if palette.len() > 1 {
            // Every one of the 4096 positions points at `block_index`,
            // 4 bits each (fits any palette used in these tests) packed
            // 16-to-a-long, matching the real non-straddling scheme.
            let mut long: i64 = 0;
            for i in 0..16 {
                long |= (block_index as i64) << (i * 4);
            }
            let data = vec![long; 256];
            block_states.insert(
                "data".to_string(),
                fastnbt::Value::LongArray(fastnbt::LongArray::new(data)),
            );
        }

        let mut section = HashMap::new();
        section.insert("Y".to_string(), fastnbt::Value::Byte(0));
        section.insert("block_states".to_string(), fastnbt::Value::Compound(block_states));

        let mut chunk = HashMap::new();
        chunk.insert("sections".to_string(), fastnbt::Value::List(vec![fastnbt::Value::Compound(section)]));
        let chunk_bytes = fastnbt::to_bytes(&fastnbt::Value::Compound(chunk)).unwrap();

        let mut region = fastanvil::Region::create(Cursor::new(Vec::new())).unwrap();
        region.write_chunk(local_x, local_z, &chunk_bytes).unwrap();
        region.into_inner().unwrap().into_inner()
    }

    #[test]
    fn diff_chunk_blocks_reports_the_block_that_changed_in_a_real_git_history() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-chunk-block-diff-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("region")).unwrap();
        init(&world_dir).unwrap();

        let base = build_region_with_chunk_block(0, 0, &["minecraft:stone", "minecraft:air"], 0);
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &base).unwrap();
        commit(&world_dir, "Base region", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let changed = build_region_with_chunk_block(0, 0, &["minecraft:stone", "minecraft:air"], 1);
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &changed).unwrap();
        commit(&world_dir, "Dig out chunk (0,0)", "Tester", "tester@example.com").unwrap();

        let diffs = diff_chunk_blocks(&world_dir, &main, "experiment", "region/r.0.0.mca", 0, 0).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(diffs.len(), 4096, "every block in the section was flipped from stone to air");
        assert!(diffs.iter().all(|d| d.from == "minecraft:stone" && d.to == "minecraft:air"));
        assert!(diffs.iter().all(|d| (0..16).contains(&d.x) && (0..16).contains(&d.z) && (0..16).contains(&d.y)));
    }

    #[test]
    fn world_block_stats_sums_across_region_files_sorted_most_common_first() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-block-stats-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("region")).unwrap();
        init(&world_dir).unwrap();

        // r.0.0.mca: one chunk, all stone (single-entry palette, no `data`).
        let region_a = build_region_with_chunk_block(0, 0, &["minecraft:stone"], 0);
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &region_a).unwrap();
        // r.1.0.mca: one chunk, all dirt — a second region file, to confirm
        // totals are summed across files, not just within one.
        let region_b = build_region_with_chunk_block(0, 0, &["minecraft:dirt"], 0);
        std::fs::write(world_dir.join("region").join("r.1.0.mca"), &region_b).unwrap();
        commit(&world_dir, "Two regions", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();

        let stats = world_block_stats(&world_dir, &main).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(stats, vec![("minecraft:dirt".to_string(), 4096), ("minecraft:stone".to_string(), 4096)]);
    }

    /// Builds a region with one chunk that has a `structures.starts` entry
    /// for `structure_id` — enough to exercise `world_structure_stats`
    /// without a real village/trial chamber's full generation data.
    fn build_region_with_structure_start(local_x: usize, local_z: usize, structure_id: &str) -> Vec<u8> {
        use std::collections::HashMap;
        use std::io::Cursor;

        let mut starts = HashMap::new();
        starts.insert(structure_id.to_string(), fastnbt::Value::Compound(HashMap::new()));
        let mut structures = HashMap::new();
        structures.insert("starts".to_string(), fastnbt::Value::Compound(starts));

        let mut chunk = HashMap::new();
        chunk.insert("structures".to_string(), fastnbt::Value::Compound(structures));
        let bytes = fastnbt::to_bytes(&fastnbt::Value::Compound(chunk)).unwrap();

        let mut region = fastanvil::Region::create(Cursor::new(Vec::new())).unwrap();
        region.write_chunk(local_x, local_z, &bytes).unwrap();
        region.into_inner().unwrap().into_inner()
    }

    #[test]
    fn world_structure_stats_sums_across_region_files_sorted_most_common_first() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-structure-stats-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("region")).unwrap();
        init(&world_dir).unwrap();

        let region_a = build_region_with_structure_start(0, 0, "minecraft:village_plains");
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &region_a).unwrap();
        // A second village start, in a second region file — confirms totals
        // are summed across files, and that a single instance in each
        // outranks a lone trial-chambers start below it.
        let region_b = build_region_with_structure_start(0, 0, "minecraft:village_plains");
        std::fs::write(world_dir.join("region").join("r.1.0.mca"), &region_b).unwrap();
        let region_c = build_region_with_structure_start(0, 0, "minecraft:trial_chambers");
        std::fs::write(world_dir.join("region").join("r.2.0.mca"), &region_c).unwrap();
        commit(&world_dir, "Three regions", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();

        let stats = world_structure_stats(&world_dir, &main).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(
            stats,
            vec![("minecraft:village_plains".to_string(), 2), ("minecraft:trial_chambers".to_string(), 1)]
        );
    }

    /// Builds an `entities/`-shaped region (root `Entities` list, not
    /// `sections`) with one chunk holding one entity of `id`.
    fn build_region_with_entity(local_x: usize, local_z: usize, id: &str) -> Vec<u8> {
        use std::collections::HashMap;
        use std::io::Cursor;

        let mut entity = HashMap::new();
        entity.insert("id".to_string(), fastnbt::Value::String(id.to_string()));

        let mut chunk = HashMap::new();
        chunk.insert("Entities".to_string(), fastnbt::Value::List(vec![fastnbt::Value::Compound(entity)]));
        let bytes = fastnbt::to_bytes(&fastnbt::Value::Compound(chunk)).unwrap();

        let mut region = fastanvil::Region::create(Cursor::new(Vec::new())).unwrap();
        region.write_chunk(local_x, local_z, &bytes).unwrap();
        region.into_inner().unwrap().into_inner()
    }

    #[test]
    fn world_entity_stats_sums_across_region_files_sorted_most_common_first() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-entity-stats-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("entities")).unwrap();
        init(&world_dir).unwrap();

        let region_a = build_region_with_entity(0, 0, "minecraft:sheep");
        std::fs::write(world_dir.join("entities").join("r.0.0.mca"), &region_a).unwrap();
        let region_b = build_region_with_entity(0, 0, "minecraft:sheep");
        std::fs::write(world_dir.join("entities").join("r.1.0.mca"), &region_b).unwrap();
        let region_c = build_region_with_entity(0, 0, "minecraft:cow");
        std::fs::write(world_dir.join("entities").join("r.2.0.mca"), &region_c).unwrap();
        commit(&world_dir, "Three entity regions", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();

        let stats = world_entity_stats(&world_dir, &main).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(stats, vec![("minecraft:sheep".to_string(), 2), ("minecraft:cow".to_string(), 1)]);
    }

    #[test]
    fn diff_chunk_structures_reports_the_structure_that_changed_in_a_real_git_history() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-chunk-structure-diff-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("region")).unwrap();
        init(&world_dir).unwrap();

        let base = build_region_with_structure_start(0, 0, "minecraft:mineshaft");
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &base).unwrap();
        commit(&world_dir, "Base region", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let changed = build_region_with_structure_start(0, 0, "minecraft:village_plains");
        std::fs::write(world_dir.join("region").join("r.0.0.mca"), &changed).unwrap();
        commit(&world_dir, "Different structure starts here now", "Tester", "tester@example.com").unwrap();

        let diffs = diff_chunk_structures(&world_dir, &main, "experiment", "region/r.0.0.mca", 0, 0).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(diffs.len(), 2);
        assert!(diffs
            .iter()
            .any(|d| d.id == "minecraft:mineshaft" && d.presence == mcgit_world::Presence::Removed));
        assert!(diffs
            .iter()
            .any(|d| d.id == "minecraft:village_plains" && d.presence == mcgit_world::Presence::Added));
    }

    /// Builds an `entities/`-shaped region with one chunk holding one entity
    /// of `id`/`uuid` — unlike `build_region_with_entity`, includes `UUID`
    /// (required for `diff_chunk_entities`'s identity, see `entity_identity`).
    fn build_region_with_entity_uuid(local_x: usize, local_z: usize, id: &str, uuid: [i32; 4]) -> Vec<u8> {
        use std::collections::HashMap;
        use std::io::Cursor;

        let mut entity = HashMap::new();
        entity.insert("id".to_string(), fastnbt::Value::String(id.to_string()));
        entity.insert("UUID".to_string(), fastnbt::Value::IntArray(fastnbt::IntArray::new(uuid.to_vec())));

        let mut chunk = HashMap::new();
        chunk.insert("Entities".to_string(), fastnbt::Value::List(vec![fastnbt::Value::Compound(entity)]));
        let bytes = fastnbt::to_bytes(&fastnbt::Value::Compound(chunk)).unwrap();

        let mut region = fastanvil::Region::create(Cursor::new(Vec::new())).unwrap();
        region.write_chunk(local_x, local_z, &bytes).unwrap();
        region.into_inner().unwrap().into_inner()
    }

    #[test]
    fn diff_chunk_entities_reports_the_entity_that_changed_in_a_real_git_history() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-chunk-entity-diff-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();
        std::fs::create_dir_all(world_dir.join("entities")).unwrap();
        init(&world_dir).unwrap();

        let base = build_region_with_entity_uuid(0, 0, "minecraft:sheep", [1, 2, 3, 4]);
        std::fs::write(world_dir.join("entities").join("r.0.0.mca"), &base).unwrap();
        commit(&world_dir, "Base region", "Tester", "tester@example.com").unwrap();
        let main = current_branch(&world_dir).unwrap();
        create_branch(&world_dir, "experiment").unwrap();

        let changed = build_region_with_entity_uuid(0, 0, "minecraft:cow", [5, 6, 7, 8]);
        std::fs::write(world_dir.join("entities").join("r.0.0.mca"), &changed).unwrap();
        commit(&world_dir, "Sheep left, cow arrived", "Tester", "tester@example.com").unwrap();

        let diffs = diff_chunk_entities(&world_dir, &main, "experiment", "entities/r.0.0.mca", 0, 0).unwrap();

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert_eq!(diffs.len(), 2);
        assert!(diffs
            .iter()
            .any(|d| d.id == "minecraft:sheep" && d.presence == mcgit_world::Presence::Removed));
        assert!(diffs
            .iter()
            .any(|d| d.id == "minecraft:cow" && d.presence == mcgit_world::Presence::Added));
    }
}
