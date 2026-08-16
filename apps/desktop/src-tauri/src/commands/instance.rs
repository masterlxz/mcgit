use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use mcgit_db::entities::instance::Model as InstanceRow;
use mcgit_db::entities::java_installation::{JavaSource, Model as JavaInstallationRow};
use mcgit_db::instance as db_instance;
use mcgit_db::java as db_java;
use mcgit_minecraft::manifest;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct InstanceDto {
    pub id: i64,
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub java_installation_id: Option<i64>,
    pub status: String,
    pub created_at: String,
}

impl From<InstanceRow> for InstanceDto {
    fn from(row: InstanceRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            mc_version: row.mc_version,
            loader: row.loader.as_str().to_string(),
            loader_version: row.loader_version,
            java_installation_id: row.java_installation_id,
            status: row.status.as_str().to_string(),
            created_at: row.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct McVersionDto {
    pub id: String,
    pub kind: String,
    pub release_time: String,
}

impl From<manifest::VersionEntry> for McVersionDto {
    fn from(entry: manifest::VersionEntry) -> Self {
        Self {
            id: entry.id,
            kind: entry.kind,
            release_time: entry.release_time,
        }
    }
}

/// Pure database read — what the Instances screen loads on mount.
#[tauri::command]
pub async fn list_instances(state: State<'_, AppState>) -> Result<Vec<InstanceDto>, String> {
    db_instance::list_all(&state.db)
        .await
        .map(|rows| rows.into_iter().map(InstanceDto::from).collect())
        .map_err(|e| e.to_string())
}

/// Minecraft versions available to create an instance from, straight from
/// Mojang's live piston-meta manifest — no database involved.
#[tauri::command]
pub async fn list_mc_versions(include_snapshots: bool) -> Result<Vec<McVersionDto>, String> {
    let client = reqwest::Client::new();
    let list = manifest::fetch_version_manifest(&client)
        .await
        .map_err(|e| e.to_string())?;

    let versions = list
        .versions
        .into_iter()
        .filter(|entry| include_snapshots || entry.kind == "release")
        .map(McVersionDto::from)
        .collect();

    Ok(versions)
}

#[derive(Debug, Clone, Serialize)]
pub struct InstanceInstallProgressPayload {
    pub instance_id: i64,
    pub phase: String,
    pub files_done: Option<u32>,
    pub files_total: Option<u32>,
    pub bytes_done: Option<u64>,
    pub bytes_total: Option<u64>,
}

/// Creates a Vanilla Minecraft instance: fetches the version manifest,
/// resolves (or installs) the required Java version, scaffolds the
/// instance's folder layout, and downloads the client jar/libraries/assets.
/// Does NOT launch the game — that's the separate, still-unimplemented Game
/// Runner (out of scope: it needs an authenticated Minecraft Services
/// session, which the paused Microsoft login work would provide).
///
/// The instance row is created in `Installing` status before any file
/// exists, so a failure partway through leaves a diagnosable `Failed` row
/// and partial folder on disk instead of silently vanishing.
#[tauri::command]
pub async fn create_vanilla_instance(
    app: AppHandle,
    state: State<'_, AppState>,
    name: String,
    mc_version: String,
) -> Result<InstanceDto, String> {
    let client = reqwest::Client::new();

    let list = manifest::fetch_version_manifest(&client)
        .await
        .map_err(|e| e.to_string())?;
    let entry = list
        .versions
        .iter()
        .find(|entry| entry.id == mc_version)
        .ok_or_else(|| format!("Minecraft version {mc_version} not found"))?;
    let detail = manifest::fetch_version_detail(&client, &entry.url)
        .await
        .map_err(|e| e.to_string())?;

    let row = db_instance::create_installing(
        &state.db,
        db_instance::NewInstance {
            name,
            mc_version,
        },
    )
    .await
    .map_err(|e| e.to_string())?;

    match run_vanilla_install(&app, &state, &client, &row, &detail).await {
        Ok(java_installation_id) => {
            let ready = db_instance::mark_ready(&state.db, row.id, java_installation_id)
                .await
                .map_err(|e| e.to_string())?;
            Ok(ready.into())
        }
        Err(err) => {
            // Best-effort: if this also fails, the row stays `Installing`
            // forever, but the original error is still the one that
            // matters to the caller.
            let _ = db_instance::mark_failed(&state.db, row.id).await;
            Err(err)
        }
    }
}

/// The part of instance creation that can fail partway through: folder
/// scaffolding, Java resolution, and the Vanilla file download. Returns the
/// resolved Java installation's id on success.
async fn run_vanilla_install(
    app: &AppHandle,
    state: &AppState,
    client: &reqwest::Client,
    row: &InstanceRow,
    detail: &manifest::VersionDetail,
) -> Result<i64, String> {
    let manifest_stub = mcgit_instance::types::InstanceManifest {
        id: row.id,
        name: row.name.clone(),
        mc_version: row.mc_version.clone(),
        loader: row.loader.as_str().to_string(),
        loader_version: row.loader_version.clone(),
        java_installation_path: None,
        created_at: row.created_at.clone(),
    };

    let instances_dir = state.instances_dir.clone();
    let instance_id = row.id;
    let scaffold_manifest = manifest_stub.clone();
    let instance_root = tauri::async_runtime::spawn_blocking(move || {
        mcgit_instance::scaffold::create_instance_layout(&instances_dir, instance_id, &scaffold_manifest)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    let java_installation = resolve_java(app, state, row.id, detail.java_version.major_version).await?;

    let cache_dir = state.library_cache_dir.clone();
    let app_for_progress = app.clone();
    mcgit_minecraft::install::download_files(client, detail, &cache_dir, move |stage| {
        emit_minecraft_stage(&app_for_progress, instance_id, stage);
    })
    .await
    .map_err(|e| e.to_string())?;

    let final_manifest = mcgit_instance::types::InstanceManifest {
        java_installation_path: Some(java_installation.path.clone()),
        ..manifest_stub
    };
    mcgit_instance::instance_json::write(&instance_root, &final_manifest).map_err(|e| e.to_string())?;

    Ok(java_installation.id)
}

/// Reuses an existing Java installation matching the required major version
/// if one still probes as valid (same validation `add_manual_java` uses),
/// otherwise downloads and installs one — the exact flow the Java Manager's
/// "Install" button already performs by hand.
async fn resolve_java(
    app: &AppHandle,
    state: &AppState,
    instance_id: i64,
    required_major_version: u32,
) -> Result<JavaInstallationRow, String> {
    let candidates = db_java::list_all(&state.db).await.map_err(|e| e.to_string())?;
    for candidate in candidates
        .into_iter()
        .filter(|c| c.major_version as u32 == required_major_version)
    {
        let path = std::path::PathBuf::from(&candidate.path);
        let still_valid = tauri::async_runtime::spawn_blocking(move || mcgit_java::detect::probe_path(&path))
            .await
            .map_err(|e| e.to_string())?
            .is_some();
        if still_valid {
            return Ok(candidate);
        }
    }

    emit(app, instance_id, "resolving_java");
    let java_dir = state.java_dir.clone();
    let app_for_progress = app.clone();
    let installed = mcgit_java::install::download_and_install(required_major_version, &java_dir, move |stage| {
        emit_java_stage(&app_for_progress, instance_id, stage);
    })
    .await
    .map_err(|e| e.to_string())?;

    let new = db_java::NewJavaInstallation {
        major_version: installed.major_version as i32,
        vendor: installed.vendor,
        path: installed.path.to_string_lossy().to_string(),
        source: JavaSource::Managed,
    };
    db_java::upsert_by_path(&state.db, new)
        .await
        .map_err(|e| e.to_string())
}

fn emit(app: &AppHandle, instance_id: i64, phase: &str) {
    let _ = app.emit(
        "instance://install-progress",
        InstanceInstallProgressPayload {
            instance_id,
            phase: phase.to_string(),
            files_done: None,
            files_total: None,
            bytes_done: None,
            bytes_total: None,
        },
    );
}

fn emit_java_stage(app: &AppHandle, instance_id: i64, stage: mcgit_java::types::InstallStage) {
    use mcgit_java::types::InstallStage;

    // All sub-stages of Java resolution are reported under the single
    // "resolving_java" phase — the Instances screen doesn't need to
    // distinguish "downloading the JDK" from "extracting it," only that
    // Java resolution is what's currently happening.
    let (bytes_done, bytes_total) = match stage {
        InstallStage::Downloading { bytes_done, bytes_total } => (Some(bytes_done), Some(bytes_total)),
        InstallStage::Verifying | InstallStage::Extracting | InstallStage::Done => (None, None),
    };
    let _ = app.emit(
        "instance://install-progress",
        InstanceInstallProgressPayload {
            instance_id,
            phase: "resolving_java".to_string(),
            files_done: None,
            files_total: None,
            bytes_done,
            bytes_total,
        },
    );
}

fn emit_minecraft_stage(app: &AppHandle, instance_id: i64, stage: mcgit_minecraft::types::InstallStage) {
    use mcgit_minecraft::types::InstallStage;

    let payload = match stage {
        InstallStage::DownloadingClient { bytes_done, bytes_total } => InstanceInstallProgressPayload {
            instance_id,
            phase: "downloading_client".to_string(),
            files_done: None,
            files_total: None,
            bytes_done: Some(bytes_done),
            bytes_total: Some(bytes_total),
        },
        InstallStage::DownloadingLibraries { files_done, files_total, bytes_done, bytes_total } => {
            InstanceInstallProgressPayload {
                instance_id,
                phase: "downloading_libraries".to_string(),
                files_done: Some(files_done),
                files_total: Some(files_total),
                bytes_done: Some(bytes_done),
                bytes_total: Some(bytes_total),
            }
        }
        InstallStage::DownloadingAssets { files_done, files_total, bytes_done, bytes_total } => {
            InstanceInstallProgressPayload {
                instance_id,
                phase: "downloading_assets".to_string(),
                files_done: Some(files_done),
                files_total: Some(files_total),
                bytes_done: Some(bytes_done),
                bytes_total: Some(bytes_total),
            }
        }
        InstallStage::Verifying => InstanceInstallProgressPayload {
            instance_id,
            phase: "verifying".to_string(),
            files_done: None,
            files_total: None,
            bytes_done: None,
            bytes_total: None,
        },
        InstallStage::Done => InstanceInstallProgressPayload {
            instance_id,
            phase: "done".to_string(),
            files_done: None,
            files_total: None,
            bytes_done: None,
            bytes_total: None,
        },
    };
    let _ = app.emit("instance://install-progress", payload);
}
