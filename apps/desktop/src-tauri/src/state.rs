use std::path::PathBuf;

use mcgit_db::Db;

/// Shared state handed to every Tauri command via `State<'_, AppState>`.
/// This is the only place `mcgit-java`, `mcgit-minecraft`, `mcgit-instance`,
/// and `mcgit-db` get wired together. `Db` wraps a SeaORM connection pool
/// internally — safe to share across concurrent commands without a `Mutex`.
pub struct AppState {
    pub db: Db,
    pub java_dir: PathBuf,
    /// `instances/<id>/` folders live here.
    pub instances_dir: PathBuf,
    /// Shared cache of client jars/libraries/assets, keyed by version/hash —
    /// outside any instance folder, reused across instances. See
    /// `mcgit-minecraft`'s `install::download_files`.
    pub library_cache_dir: PathBuf,
}
