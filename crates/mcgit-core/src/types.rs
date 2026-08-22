use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitError {
    #[error("could not run git: {0}")]
    Spawn(String),
    #[error("git command failed: {0}")]
    CommandFailed(String),
    #[error("could not write to the world's folder: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum RestoreError {
    #[error("this world appears to be open in Minecraft right now — close it before restoring")]
    WorldCurrentlyOpen,
    #[error(transparent)]
    Git(#[from] GitError),
    #[error("could not check whether the world is open: {0}")]
    Io(#[from] std::io::Error),
}
