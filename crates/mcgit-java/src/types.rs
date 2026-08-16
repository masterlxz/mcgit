use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedJava {
    pub path: PathBuf,
    pub major_version: u32,
    pub vendor: String,
}

#[derive(Debug, Error)]
pub enum JavaError {
    #[error("could not parse Java version from output: {0:?}")]
    UnrecognizedVersionOutput(String),
    #[error("network request failed: {0}")]
    Network(String),
    #[error("could not parse Adoptium API response: {0}")]
    UnrecognizedApiResponse(String),
    #[error("filesystem error: {0}")]
    Io(String),
    #[error("checksum mismatch: expected {expected}, got {actual}")]
    ChecksumMismatch { expected: String, actual: String },
}

/// Progress reported while `install::download_and_install` runs, so a caller
/// (e.g. a Tauri command) can show a progress bar without knowing how the
/// download/verify/extract pipeline works internally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStage {
    Downloading { bytes_done: u64, bytes_total: u64 },
    Verifying,
    Extracting,
    Done,
}
