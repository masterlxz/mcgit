use std::path::Path;

use crate::types::{InstanceError, InstanceManifest};

const MANIFEST_FILE_NAME: &str = "instance.json";

pub fn write(root: &Path, manifest: &InstanceManifest) -> Result<(), InstanceError> {
    let json = serde_json::to_string_pretty(manifest)
        .map_err(|e| InstanceError::MalformedManifest(e.to_string()))?;
    std::fs::write(root.join(MANIFEST_FILE_NAME), json).map_err(|e| InstanceError::Io(e.to_string()))
}

pub fn read(root: &Path) -> Result<InstanceManifest, InstanceError> {
    let contents = std::fs::read_to_string(root.join(MANIFEST_FILE_NAME))
        .map_err(|e| InstanceError::Io(e.to_string()))?;
    serde_json::from_str(&contents).map_err(|e| InstanceError::MalformedManifest(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> InstanceManifest {
        InstanceManifest {
            id: 1,
            name: "Survival".to_string(),
            mc_version: "26.2".to_string(),
            loader: "vanilla".to_string(),
            loader_version: None,
            java_installation_path: Some("/usr/lib/jvm/temurin-25/bin/java".to_string()),
            created_at: "2026-08-16T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn write_then_read_round_trips() {
        let root = std::env::temp_dir().join(format!(
            "mcgit-instance-test-roundtrip-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();

        write(&root, &sample()).unwrap();
        let read_back = read(&root).unwrap();

        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(read_back, sample());
    }

    #[test]
    fn read_missing_file_returns_error_not_panic() {
        let root = std::env::temp_dir().join(format!(
            "mcgit-instance-test-missing-{}",
            std::process::id()
        ));
        let result = read(&root);
        assert!(matches!(result, Err(InstanceError::Io(_))));
    }

    #[test]
    fn read_corrupt_file_returns_error_not_panic() {
        let root = std::env::temp_dir().join(format!(
            "mcgit-instance-test-corrupt-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join(MANIFEST_FILE_NAME), b"{ not json").unwrap();

        let result = read(&root);

        std::fs::remove_dir_all(&root).unwrap();
        assert!(matches!(result, Err(InstanceError::MalformedManifest(_))));
    }
}
