use std::path::{Path, PathBuf};

use crate::instance_json;
use crate::types::{InstanceError, InstanceManifest};

/// Subdirectories every instance's isolated `.minecraft`-equivalent gets,
/// per the isolation model in `ARCHITECTURE.md` §Gerenciamento de
/// Instâncias. Shared library/asset caching lives entirely outside this
/// tree (see `mcgit-minecraft`'s cache layout) — nothing here duplicates it.
pub const MINECRAFT_SUBDIRS: [&str; 7] = [
    "mods",
    "resourcepacks",
    "shaderpacks",
    "saves",
    "screenshots",
    "logs",
    "config",
];

/// `instances/<id>/` — named by the database's own autoincrement id, not a
/// UUID (see `InstanceManifest`'s doc comment).
pub fn instance_root(instances_dir: &Path, id: i64) -> PathBuf {
    instances_dir.join(id.to_string())
}

/// Creates `instances/<id>/minecraft/{mods,resourcepacks,...}` and writes
/// `instance.json` at the instance root. Returns the instance's root path.
pub fn create_instance_layout(
    instances_dir: &Path,
    id: i64,
    manifest: &InstanceManifest,
) -> Result<PathBuf, InstanceError> {
    let root = instance_root(instances_dir, id);
    let minecraft_dir = root.join("minecraft");
    for sub in MINECRAFT_SUBDIRS {
        std::fs::create_dir_all(minecraft_dir.join(sub))
            .map_err(|e| InstanceError::Io(e.to_string()))?;
    }
    instance_json::write(&root, manifest)?;
    Ok(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest(id: i64) -> InstanceManifest {
        InstanceManifest {
            id,
            name: "Survival".to_string(),
            mc_version: "26.2".to_string(),
            loader: "vanilla".to_string(),
            loader_version: None,
            java_installation_path: None,
            created_at: "2026-08-16T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn creates_all_subdirs_and_manifest_file() {
        let instances_dir = std::env::temp_dir().join(format!(
            "mcgit-instance-test-scaffold-{}",
            std::process::id()
        ));

        let root = create_instance_layout(&instances_dir, 1, &sample_manifest(1)).unwrap();

        assert_eq!(root, instances_dir.join("1"));
        for sub in MINECRAFT_SUBDIRS {
            assert!(root.join("minecraft").join(sub).is_dir(), "missing {sub}");
        }
        assert!(root.join("instance.json").is_file());

        std::fs::remove_dir_all(&instances_dir).unwrap();
    }

    #[test]
    fn manifest_written_by_scaffold_reads_back_correctly() {
        let instances_dir = std::env::temp_dir().join(format!(
            "mcgit-instance-test-scaffold-read-{}",
            std::process::id()
        ));

        let manifest = sample_manifest(2);
        let root = create_instance_layout(&instances_dir, 2, &manifest).unwrap();
        let read_back = instance_json::read(&root).unwrap();

        std::fs::remove_dir_all(&instances_dir).unwrap();
        assert_eq!(read_back, manifest);
    }
}
