use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maps 1:1 onto `instances/<id>/instance.json` — the on-disk mirror of an
/// instance's `instances` row in `mcgit-db`. `id` is the database's own
/// autoincrement id (also used as the instance's folder name), not a UUID —
/// consistent with the rest of the project, no UUID precedent exists yet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InstanceManifest {
    pub id: i64,
    pub name: String,
    pub mc_version: String,
    pub loader: String,
    pub loader_version: Option<String>,
    pub java_installation_path: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Error)]
pub enum InstanceError {
    #[error("filesystem error: {0}")]
    Io(String),
    #[error("malformed instance.json: {0}")]
    MalformedManifest(String),
}
