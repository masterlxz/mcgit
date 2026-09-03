use std::collections::HashMap;

use serde::Serialize;
use tauri::State;

use mcgit_db::world as db_world;
use mcgit_instance::{scaffold, worlds};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct WorldDto {
    pub folder_name: String,
    pub git_enabled: bool,
}

/// Lists the worlds (`saves/*` subdirectories containing a `level.dat`)
/// inside an instance. The filesystem decides which worlds exist; the
/// `worlds` table only supplies `git_enabled` for the ones that have it set
/// (defaulting to `false` for a world never versioned before).
#[tauri::command]
pub async fn list_worlds(
    instance_id: i64,
    state: State<'_, AppState>,
) -> Result<Vec<WorldDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let fs_worlds = tauri::async_runtime::spawn_blocking(move || {
        let root = scaffold::instance_root(&instances_dir, instance_id);
        worlds::list_worlds(&root)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let db_rows = db_world::list_by_instance(&state.db, instance_id)
        .await
        .map_err(|e| e.to_string())?;
    let enabled_by_folder: HashMap<String, bool> =
        db_rows.into_iter().map(|row| (row.folder_name, row.git_enabled)).collect();

    Ok(fs_worlds
        .into_iter()
        .map(|world| WorldDto {
            git_enabled: enabled_by_folder.get(&world.folder_name).copied().unwrap_or(false),
            folder_name: world.folder_name,
        })
        .collect())
}

/// Runs `git init` in the world's folder, then records `git_enabled = true`
/// for it. `git init` on an already-initialized repo is a safe no-op, so
/// this is safe to call again on a world that already has versioning on.
#[tauri::command]
pub async fn enable_world_versioning(
    instance_id: i64,
    folder_name: String,
    state: State<'_, AppState>,
) -> Result<WorldDto, String> {
    let instances_dir = state.instances_dir.clone();
    let folder_name_for_fs = folder_name.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name_for_fs);
        mcgit_core::git::init(&world_dir)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let row = db_world::set_git_enabled(&state.db, instance_id, &folder_name, true)
        .await
        .map_err(|e| e.to_string())?;
    Ok(WorldDto {
        folder_name: row.folder_name,
        git_enabled: row.git_enabled,
    })
}

/// Turns versioning off for a world. Does NOT touch the `.git` directory or
/// any history on disk — only hides the versioning UI for this world going
/// forward. Re-enabling later just flips the flag back.
#[tauri::command]
pub async fn disable_world_versioning(
    instance_id: i64,
    folder_name: String,
    state: State<'_, AppState>,
) -> Result<WorldDto, String> {
    let row = db_world::set_git_enabled(&state.db, instance_id, &folder_name, false)
        .await
        .map_err(|e| e.to_string())?;
    Ok(WorldDto {
        folder_name: row.folder_name,
        git_enabled: row.git_enabled,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotResultDto {
    pub created: bool,
    pub commit_hash: Option<String>,
}

/// Saves a snapshot of the world's folder (`git add -A` + `git commit`).
/// `created = false` means nothing had changed since the last snapshot —
/// not a failure, just nothing to do.
#[tauri::command]
pub async fn create_world_snapshot(
    instance_id: i64,
    folder_name: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<SnapshotResultDto, String> {
    let (commit_name, commit_email) = mcgit_db::settings::get_commit_identity(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let instances_dir = state.instances_dir.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::commit(&world_dir, &message, &commit_name, &commit_email)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(match outcome {
        mcgit_core::git::CommitOutcome::Created(hash) => SnapshotResultDto {
            created: true,
            commit_hash: Some(hash),
        },
        mcgit_core::git::CommitOutcome::NothingToCommit => SnapshotResultDto {
            created: false,
            commit_hash: None,
        },
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct SnapshotDto {
    pub hash: String,
    pub date: String,
    pub message: String,
}

/// Lists every snapshot saved for a world, most recent first. A world
/// that was never versioned, or was versioned but never had a snapshot
/// saved yet, both return an empty list — not an error.
#[tauri::command]
pub async fn list_world_history(
    instance_id: i64,
    folder_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<SnapshotDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let history = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::log(&world_dir)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(history
        .into_iter()
        .map(|snapshot| SnapshotDto {
            hash: snapshot.hash,
            date: snapshot.date,
            message: snapshot.message,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct RestoreDto {
    pub backup_created: bool,
    pub restored: bool,
}

/// Restores a world to an older snapshot. Never destructive: a backup
/// snapshot of the current state is always taken first, and bringing the
/// files back is itself recorded as a new snapshot, never a history rewrite.
/// Refuses if the world looks currently open in Minecraft.
#[tauri::command]
pub async fn restore_world_version(
    instance_id: i64,
    folder_name: String,
    commit_hash: String,
    state: State<'_, AppState>,
) -> Result<RestoreDto, String> {
    let (commit_name, commit_email) = mcgit_db::settings::get_commit_identity(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let instances_dir = state.instances_dir.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::restore(&world_dir, &commit_hash, &commit_name, &commit_email)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(RestoreDto {
        backup_created: matches!(outcome.backup, mcgit_core::git::CommitOutcome::Created(_)),
        restored: matches!(outcome.restore, mcgit_core::git::CommitOutcome::Created(_)),
    })
}

/// Deletes a snapshot from a world's history. The only truly destructive
/// world command — never silent, the UI always confirms before calling this.
#[tauri::command]
pub async fn delete_world_snapshot(
    instance_id: i64,
    folder_name: String,
    commit_hash: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::delete_snapshot(&world_dir, &commit_hash)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct BranchDto {
    pub name: String,
    pub is_current: bool,
}

fn branch_dtos(world_dir: &std::path::Path) -> Result<Vec<BranchDto>, mcgit_core::types::GitError> {
    let current = mcgit_core::git::current_branch(world_dir)?;
    let names = mcgit_core::git::list_branches(world_dir)?;
    Ok(names
        .into_iter()
        .map(|name| {
            let is_current = name == current;
            BranchDto { name, is_current }
        })
        .collect())
}

/// Lists every branch for a world, marking which one is current. Nothing is
/// cached in the database — the current branch is always derived live from
/// Git, same as history.
#[tauri::command]
pub async fn list_world_branches(
    instance_id: i64,
    folder_name: String,
    state: State<'_, AppState>,
) -> Result<Vec<BranchDto>, String> {
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        branch_dtos(&world_dir)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// Creates a new branch at the world's current commit and switches to it.
/// Returns the updated branch list so the caller doesn't need a second
/// round trip.
#[tauri::command]
pub async fn create_world_branch(
    instance_id: i64,
    folder_name: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<Vec<BranchDto>, String> {
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::create_branch(&world_dir, &name)?;
        branch_dtos(&world_dir)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct SwitchDto {
    pub checkpoint_created: bool,
    pub branch: String,
}

/// Switches a world to a different, already-existing branch. Refuses if the
/// world looks currently open in Minecraft. Any pending change on the
/// branch being left is checkpointed automatically first, so this never
/// fails from "local changes would be overwritten" and never loses work.
#[tauri::command]
pub async fn switch_world_branch(
    instance_id: i64,
    folder_name: String,
    name: String,
    state: State<'_, AppState>,
) -> Result<SwitchDto, String> {
    let (commit_name, commit_email) = mcgit_db::settings::get_commit_identity(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let instances_dir = state.instances_dir.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::switch_branch(&world_dir, &name, &commit_name, &commit_email)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(SwitchDto {
        checkpoint_created: matches!(outcome.checkpoint, mcgit_core::git::CommitOutcome::Created(_)),
        branch: outcome.branch,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct FileChangeDto {
    pub path: String,
    pub status: String,
    pub old_size: Option<u64>,
    pub new_size: Option<u64>,
}

impl From<mcgit_core::git::FileChange> for FileChangeDto {
    fn from(change: mcgit_core::git::FileChange) -> Self {
        let status = match change.status {
            mcgit_core::git::ChangeStatus::Added => "added",
            mcgit_core::git::ChangeStatus::Modified => "modified",
            mcgit_core::git::ChangeStatus::Deleted => "deleted",
        };
        FileChangeDto {
            path: change.path,
            status: status.to_string(),
            old_size: change.old_size,
            new_size: change.new_size,
        }
    }
}

/// Compares the world's current branch against `other_branch`, file by
/// file. No content diff — see `mcgit_core::git::FileChange`.
#[tauri::command]
pub async fn diff_world_branches(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    state: State<'_, AppState>,
) -> Result<Vec<FileChangeDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let changes = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        let current = mcgit_core::git::current_branch(&world_dir)?;
        mcgit_core::git::diff_branches(&world_dir, &current, &other_branch)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(changes.into_iter().map(FileChangeDto::from).collect())
}

/// Previews merging `other_branch` into the world's current branch without
/// touching anything — returns the paths that would conflict (empty means
/// a clean merge).
#[tauri::command]
pub async fn preview_world_merge(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    state: State<'_, AppState>,
) -> Result<Vec<String>, String> {
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        let current = mcgit_core::git::current_branch(&world_dir)?;
        mcgit_core::git::preview_merge(&world_dir, &current, &other_branch)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct ConflictedFileDto {
    pub path: String,
    pub kind: String,
}

impl From<mcgit_core::git::ConflictedFile> for ConflictedFileDto {
    fn from(conflict: mcgit_core::git::ConflictedFile) -> Self {
        let kind = match conflict.kind {
            mcgit_core::git::ConflictKind::BothModified => "both_modified",
            mcgit_core::git::ConflictKind::DeletedByUs => "deleted_by_us",
            mcgit_core::git::ConflictKind::DeletedByThem => "deleted_by_them",
        };
        ConflictedFileDto {
            path: conflict.path,
            kind: kind.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind")]
pub enum MergeOutcomeDto {
    Merged { commit_hash: String },
    ConflictsPending { conflicts: Vec<ConflictedFileDto> },
}

/// Merges `other_branch` into the world's current branch. Refuses if the
/// world looks currently open in Minecraft, or if a merge is already in
/// progress. A clean merge returns `Merged`; a conflicting one leaves the
/// merge in progress (never silently discards anything) and returns the
/// files that need resolving.
#[tauri::command]
pub async fn merge_world_branch(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    state: State<'_, AppState>,
) -> Result<MergeOutcomeDto, String> {
    let (commit_name, commit_email) = mcgit_db::settings::get_commit_identity(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let instances_dir = state.instances_dir.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::merge_branch(&world_dir, &other_branch, &commit_name, &commit_email)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(match outcome {
        mcgit_core::git::MergeOutcome::Merged(commit_hash) => MergeOutcomeDto::Merged { commit_hash },
        mcgit_core::git::MergeOutcome::ConflictsPending(conflicts) => MergeOutcomeDto::ConflictsPending {
            conflicts: conflicts.into_iter().map(ConflictedFileDto::from).collect(),
        },
    })
}

/// Resolves one conflicted file during an in-progress merge by keeping
/// either this branch's version (`"ours"`) or the other branch's
/// (`"theirs"`).
#[tauri::command]
pub async fn resolve_world_merge_conflict(
    instance_id: i64,
    folder_name: String,
    path: String,
    keep: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let side = match keep.as_str() {
        "ours" => mcgit_core::git::Side::Ours,
        "theirs" => mcgit_core::git::Side::Theirs,
        other => return Err(format!("unknown side to keep: {other}")),
    };
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::resolve_conflict(&world_dir, &path, side)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// Finishes an in-progress merge once every conflict has been resolved.
/// Returns the new merge commit's hash.
#[tauri::command]
pub async fn finish_world_merge(
    instance_id: i64,
    folder_name: String,
    message: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let (commit_name, commit_email) = mcgit_db::settings::get_commit_identity(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::finish_merge(&world_dir, &message, &commit_name, &commit_email)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

/// Aborts an in-progress merge, restoring the world exactly to how it was
/// before the merge started.
#[tauri::command]
pub async fn abort_world_merge(
    instance_id: i64,
    folder_name: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let instances_dir = state.instances_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::abort_merge(&world_dir)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkDiffDto {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub status: String,
}

impl From<mcgit_world::ChunkDiff> for ChunkDiffDto {
    fn from(diff: mcgit_world::ChunkDiff) -> Self {
        let status = match diff.status {
            mcgit_world::ChunkStatus::Added => "added",
            mcgit_world::ChunkStatus::Removed => "removed",
            mcgit_world::ChunkStatus::Changed => "changed",
        };
        ChunkDiffDto {
            chunk_x: diff.chunk_x,
            chunk_z: diff.chunk_z,
            status: status.to_string(),
        }
    }
}

/// Diffs the chunks of a region file between the world's current branch and
/// `other_branch` — which 16×16 chunks were added, removed, or changed, at
/// their absolute world coordinates. Byte-level only, no content diff.
#[tauri::command]
pub async fn diff_world_region_chunks(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    path: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChunkDiffDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let diffs = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        let current = mcgit_core::git::current_branch(&world_dir)?;
        mcgit_core::git::diff_region_chunks(&world_dir, &current, &other_branch, &path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(diffs.into_iter().map(ChunkDiffDto::from).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockDiffDto {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub from: String,
    pub to: String,
}

impl From<mcgit_world::BlockDiff> for BlockDiffDto {
    fn from(diff: mcgit_world::BlockDiff) -> Self {
        BlockDiffDto {
            x: diff.x,
            y: diff.y,
            z: diff.z,
            from: diff.from,
            to: diff.to,
        }
    }
}

/// Diffs one chunk's blocks (by absolute world position) between the
/// world's current branch and `other_branch` — decodes each side's
/// `block_states` (palette + bit-packed indices) and reports exactly which
/// positions differ, and to/from what block. `chunk_x`/`chunk_z` are the
/// chunk's absolute coordinates, as reported by `diff_world_region_chunks`.
#[tauri::command]
pub async fn diff_world_chunk_blocks(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    path: String,
    chunk_x: i32,
    chunk_z: i32,
    state: State<'_, AppState>,
) -> Result<Vec<BlockDiffDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let diffs = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        let current = mcgit_core::git::current_branch(&world_dir)?;
        mcgit_core::git::diff_chunk_blocks(&world_dir, &current, &other_branch, &path, chunk_x, chunk_z)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(diffs.into_iter().map(BlockDiffDto::from).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityDiffDto {
    pub id: String,
    pub uuid: String,
    pub presence: String,
}

impl From<mcgit_world::EntityDiff> for EntityDiffDto {
    fn from(diff: mcgit_world::EntityDiff) -> Self {
        EntityDiffDto {
            id: diff.id,
            uuid: diff.uuid,
            presence: presence_str(diff.presence).to_string(),
        }
    }
}

fn presence_str(presence: mcgit_world::Presence) -> &'static str {
    match presence {
        mcgit_world::Presence::Added => "added",
        mcgit_world::Presence::Removed => "removed",
    }
}

/// Diffs one chunk's entities (by `UUID`) between the world's current
/// branch and `other_branch` — which ones appeared and which disappeared.
/// `path` is an `entities/` file (e.g. `"entities/r.0.0.mca"`), the folder
/// mobs and dropped items live in since 1.17 — not `region/`, which
/// `diff_world_chunk_blocks` reads.
#[tauri::command]
pub async fn diff_world_chunk_entities(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    path: String,
    chunk_x: i32,
    chunk_z: i32,
    state: State<'_, AppState>,
) -> Result<Vec<EntityDiffDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let diffs = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        let current = mcgit_core::git::current_branch(&world_dir)?;
        mcgit_core::git::diff_chunk_entities(&world_dir, &current, &other_branch, &path, chunk_x, chunk_z)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(diffs.into_iter().map(EntityDiffDto::from).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct StructureDiffDto {
    pub id: String,
    pub presence: String,
}

impl From<mcgit_world::StructureDiff> for StructureDiffDto {
    fn from(diff: mcgit_world::StructureDiff) -> Self {
        StructureDiffDto {
            id: diff.id,
            presence: presence_str(diff.presence).to_string(),
        }
    }
}

/// Diffs one chunk's generated structures (by structure id) between the
/// world's current branch and `other_branch` — which types started or
/// stopped being recorded as starting there. `path` is a `region/` file,
/// same folder `diff_world_chunk_blocks` reads.
#[tauri::command]
pub async fn diff_world_chunk_structures(
    instance_id: i64,
    folder_name: String,
    other_branch: String,
    path: String,
    chunk_x: i32,
    chunk_z: i32,
    state: State<'_, AppState>,
) -> Result<Vec<StructureDiffDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let diffs = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        let current = mcgit_core::git::current_branch(&world_dir)?;
        mcgit_core::git::diff_chunk_structures(&world_dir, &current, &other_branch, &path, chunk_x, chunk_z)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(diffs.into_iter().map(StructureDiffDto::from).collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockCountDto {
    pub name: String,
    pub count: u64,
}

/// Tallies every block (by bare name, ignoring block-state properties)
/// across the world's `region/` folder as it existed at `commit_hash` — a
/// single snapshot's totals, not a diff between two. Sorted most common
/// block first.
#[tauri::command]
pub async fn world_block_stats(
    instance_id: i64,
    folder_name: String,
    commit_hash: String,
    state: State<'_, AppState>,
) -> Result<Vec<BlockCountDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let stats = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::world_block_stats(&world_dir, &commit_hash)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(stats
        .into_iter()
        .map(|(name, count)| BlockCountDto { name, count })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct StructureCountDto {
    pub name: String,
    pub count: u64,
}

/// Tallies every generated structure (villages, trial chambers, ...) by
/// type across the world's `region/` folder as it existed at `commit_hash`
/// — each instance counted once, at the chunk where it started generating.
/// Sorted most common first.
#[tauri::command]
pub async fn world_structure_stats(
    instance_id: i64,
    folder_name: String,
    commit_hash: String,
    state: State<'_, AppState>,
) -> Result<Vec<StructureCountDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let stats = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::world_structure_stats(&world_dir, &commit_hash)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(stats
        .into_iter()
        .map(|(name, count)| StructureCountDto { name, count })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct EntityCountDto {
    pub name: String,
    pub count: u64,
}

/// Tallies every living entity (mobs, dropped items, ...) by `id` across
/// the world's `entities/` folder as it existed at `commit_hash`. Sorted
/// most common first.
#[tauri::command]
pub async fn world_entity_stats(
    instance_id: i64,
    folder_name: String,
    commit_hash: String,
    state: State<'_, AppState>,
) -> Result<Vec<EntityCountDto>, String> {
    let instances_dir = state.instances_dir.clone();
    let stats = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::world_entity_stats(&world_dir, &commit_hash)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(stats
        .into_iter()
        .map(|(name, count)| EntityCountDto { name, count })
        .collect())
}
