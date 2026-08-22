use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("could not run git: {0}")]
    Spawn(String),
    #[error("git command failed: {0}")]
    CommandFailed(String),
}
