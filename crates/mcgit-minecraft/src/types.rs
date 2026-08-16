use thiserror::Error;

#[derive(Debug, Error)]
pub enum MinecraftError {
    #[error("network request failed: {0}")]
    Network(String),
    #[error("filesystem error: {0}")]
    Io(String),
    #[error("could not parse piston-meta response: {0}")]
    UnrecognizedApiResponse(String),
    #[error("checksum mismatch for {path}: expected {expected}, got {actual}")]
    ChecksumMismatch {
        path: String,
        expected: String,
        actual: String,
    },
}

/// Progress reported by `install::download_files`, one event per stage
/// transition. Mirrors `mcgit_java::types::InstallStage`'s shape: a plain
/// value threaded through a sync `FnMut` callback, not a channel/event —
/// the Tauri command layer is what turns callback calls into `app.emit(...)`.
#[derive(Debug, Clone, PartialEq)]
pub enum InstallStage {
    DownloadingClient {
        bytes_done: u64,
        bytes_total: u64,
    },
    DownloadingLibraries {
        files_done: u32,
        files_total: u32,
        bytes_done: u64,
        bytes_total: u64,
    },
    DownloadingAssets {
        files_done: u32,
        files_total: u32,
        bytes_done: u64,
        bytes_total: u64,
    },
    Verifying,
    Done,
}
