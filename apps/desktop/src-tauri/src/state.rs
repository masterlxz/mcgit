use std::path::PathBuf;

use mcgit_db::Db;

/// Shared state handed to every Tauri command via `State<'_, AppState>`.
/// This is the only place `mcgit-java` and `mcgit-db` get wired together.
/// `Db` wraps a SeaORM connection pool internally — safe to share across
/// concurrent commands without a `Mutex`.
pub struct AppState {
    pub db: Db,
    pub java_dir: PathBuf,
}
