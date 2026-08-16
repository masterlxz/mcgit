use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use mcgit_db::java as db_java;
use mcgit_db::models::{JavaInstallation, JavaSource, NewJavaInstallation};
use mcgit_java::types::InstallStage;
use mcgit_java::{adoptium, detect, install};

use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
pub struct JavaInstallationDto {
    pub id: i64,
    pub major_version: u32,
    pub vendor: String,
    pub path: String,
    pub source: String,
    pub is_default: bool,
}

impl From<JavaInstallation> for JavaInstallationDto {
    fn from(row: JavaInstallation) -> Self {
        Self {
            id: row.id,
            major_version: row.major_version,
            vendor: row.vendor,
            path: row.path,
            source: row.source.as_str().to_string(),
            is_default: row.is_default,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallProgressPayload {
    pub major_version: u32,
    pub stage: String,
    pub bytes_done: Option<u64>,
    pub bytes_total: Option<u64>,
}

/// Scans this machine for Java installations, records what it finds, and
/// returns the up-to-date full list from the database.
#[tauri::command]
pub async fn scan_system_java(
    state: State<'_, AppState>,
) -> Result<Vec<JavaInstallationDto>, String> {
    // `detect_installations` spawns `java -version` a few times, which is a
    // real (if brief) blocking syscall — run it off the async executor.
    let found = tauri::async_runtime::spawn_blocking(detect::detect_installations)
        .await
        .map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    for installation in found {
        let new = NewJavaInstallation {
            major_version: installation.major_version,
            vendor: installation.vendor,
            path: installation.path.to_string_lossy().to_string(),
            source: JavaSource::Detected,
        };
        db_java::upsert_by_path(&db, &new).map_err(|e| e.to_string())?;
    }

    list_from_db(&db)
}

/// Pure database read — what the screen loads on mount.
#[tauri::command]
pub async fn list_java_installations(
    state: State<'_, AppState>,
) -> Result<Vec<JavaInstallationDto>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    list_from_db(&db)
}

/// LTS major versions available to install, from the Adoptium API.
#[tauri::command]
pub async fn list_installable_java_versions() -> Result<Vec<u32>, String> {
    adoptium::available_releases()
        .await
        .map(|releases| releases.available_lts_releases)
        .map_err(|e| e.to_string())
}

/// Downloads, verifies, extracts, and installs `major_version`, emitting
/// "java://install-progress" events as it goes.
#[tauri::command]
pub async fn install_java(
    app: AppHandle,
    state: State<'_, AppState>,
    major_version: u32,
) -> Result<JavaInstallationDto, String> {
    let java_dir = state.java_dir.clone();

    let installed = install::download_and_install(major_version, &java_dir, |stage| {
        let payload = match stage {
            InstallStage::Downloading {
                bytes_done,
                bytes_total,
            } => InstallProgressPayload {
                major_version,
                stage: "downloading".to_string(),
                bytes_done: Some(bytes_done),
                bytes_total: Some(bytes_total),
            },
            InstallStage::Verifying => InstallProgressPayload {
                major_version,
                stage: "verifying".to_string(),
                bytes_done: None,
                bytes_total: None,
            },
            InstallStage::Extracting => InstallProgressPayload {
                major_version,
                stage: "extracting".to_string(),
                bytes_done: None,
                bytes_total: None,
            },
            InstallStage::Done => InstallProgressPayload {
                major_version,
                stage: "done".to_string(),
                bytes_done: None,
                bytes_total: None,
            },
        };
        let _ = app.emit("java://install-progress", payload);
    })
    .await
    .map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let new = NewJavaInstallation {
        major_version: installed.major_version,
        vendor: installed.vendor,
        path: installed.path.to_string_lossy().to_string(),
        source: JavaSource::Managed,
    };
    let row = db_java::upsert_by_path(&db, &new).map_err(|e| e.to_string())?;

    Ok(row.into())
}

/// Validates a user-supplied Java path (advanced users) by running it with
/// `-version`, then records it.
#[tauri::command]
pub async fn add_manual_java(
    state: State<'_, AppState>,
    path: String,
) -> Result<JavaInstallationDto, String> {
    let path_buf = std::path::PathBuf::from(&path);
    let detected = tauri::async_runtime::spawn_blocking(move || detect::probe_path(&path_buf))
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("{path} does not look like a valid Java installation"))?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    let new = NewJavaInstallation {
        major_version: detected.major_version,
        vendor: detected.vendor,
        path: detected.path.to_string_lossy().to_string(),
        source: JavaSource::Manual,
    };
    let row = db_java::upsert_by_path(&db, &new).map_err(|e| e.to_string())?;

    Ok(row.into())
}

#[tauri::command]
pub async fn set_default_java(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db_java::set_default(&db, id).map_err(|e| e.to_string())
}

fn list_from_db(db: &mcgit_db::Db) -> Result<Vec<JavaInstallationDto>, String> {
    db_java::list_all(db)
        .map(|rows| rows.into_iter().map(JavaInstallationDto::from).collect())
        .map_err(|e| e.to_string())
}
