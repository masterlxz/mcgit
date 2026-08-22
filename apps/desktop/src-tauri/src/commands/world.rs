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
    let instances_dir = state.instances_dir.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::commit(&world_dir, &message)
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
    let instances_dir = state.instances_dir.clone();
    let outcome = tauri::async_runtime::spawn_blocking(move || {
        let world_dir = scaffold::instance_root(&instances_dir, instance_id)
            .join("minecraft")
            .join("saves")
            .join(&folder_name);
        mcgit_core::git::restore(&world_dir, &commit_hash)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    Ok(RestoreDto {
        backup_created: matches!(outcome.backup, mcgit_core::git::CommitOutcome::Created(_)),
        restored: matches!(outcome.restore, mcgit_core::git::CommitOutcome::Created(_)),
    })
}
