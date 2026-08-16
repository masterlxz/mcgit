use std::path::PathBuf;
use std::sync::Mutex;

use mcgit_db::Db;

/// Shared state handed to every Tauri command via `State<'_, AppState>`.
/// This is the only place `mcgit-java` and `mcgit-db` get wired together.
pub struct AppState {
    pub db: Mutex<Db>,
    pub java_dir: PathBuf,
}
