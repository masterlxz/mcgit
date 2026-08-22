use std::path::Path;
use std::process::Command;

use crate::types::GitError;

/// Runs `git init` in `world_dir`. Idempotent: `git init` on an
/// already-initialized repository is a safe no-op (Git's own guarantee),
/// so callers don't need to check `is_repository` first.
pub fn init(world_dir: &Path) -> Result<(), GitError> {
    let output = Command::new("git")
        .arg("init")
        .current_dir(world_dir)
        .output()
        .map_err(|e| GitError::Spawn(e.to_string()))?;

    if !output.status.success() {
        return Err(GitError::CommandFailed(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    Ok(())
}

/// Whether `world_dir` is already the root of a Git repository.
pub fn is_repository(world_dir: &Path) -> bool {
    world_dir.join(".git").is_dir()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_creates_git_dir() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-init-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        init(&world_dir).unwrap();

        let has_git_dir = world_dir.join(".git").is_dir();
        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(has_git_dir);
    }

    #[test]
    fn is_repository_reflects_init_state() {
        let world_dir =
            std::env::temp_dir().join(format!("mcgit-core-test-is-repository-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        let before = is_repository(&world_dir);
        init(&world_dir).unwrap();
        let after = is_repository(&world_dir);

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(!before);
        assert!(after);
    }

    #[test]
    fn init_twice_does_not_error() {
        let world_dir = std::env::temp_dir().join(format!("mcgit-core-test-init-twice-{}", std::process::id()));
        std::fs::create_dir_all(&world_dir).unwrap();

        init(&world_dir).unwrap();
        let second = init(&world_dir);

        std::fs::remove_dir_all(&world_dir).unwrap();
        assert!(second.is_ok());
    }
}
