use std::path::Path;

use crate::types::InstanceError;

/// A Minecraft world discovered under an instance's `saves/` directory.
/// Pure directory-name enumeration for now — no `level.dat` NBT parsing yet
/// (no NBT crate is a workspace dependency), so there's no display
/// name/icon, only the folder name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct World {
    pub folder_name: String,
}

/// Lists the subdirectories of `<instance_root>/minecraft/saves/` that look
/// like real worlds (contain a `level.dat`), sorted by name. `saves/` not
/// existing at all is not an error — a freshly scaffolded instance that
/// hasn't been played yet has zero worlds, not a broken one.
pub fn list_worlds(instance_root: &Path) -> Result<Vec<World>, InstanceError> {
    let saves_dir = instance_root.join("minecraft").join("saves");
    if !saves_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut worlds: Vec<World> = std::fs::read_dir(&saves_dir)
        .map_err(|e| InstanceError::Io(e.to_string()))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir() && entry.path().join("level.dat").is_file())
        .map(|entry| World {
            folder_name: entry.file_name().to_string_lossy().into_owned(),
        })
        .collect();

    worlds.sort_by(|a, b| a.folder_name.cmp(&b.folder_name));
    Ok(worlds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_empty_when_saves_dir_does_not_exist() {
        let instance_root = std::env::temp_dir().join(format!(
            "mcgit-instance-test-worlds-missing-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&instance_root).unwrap();

        let worlds = list_worlds(&instance_root).unwrap();

        std::fs::remove_dir_all(&instance_root).unwrap();
        assert!(worlds.is_empty());
    }

    #[test]
    fn lists_world_dirs_sorted_and_skips_non_world_entries() {
        let instance_root = std::env::temp_dir().join(format!(
            "mcgit-instance-test-worlds-list-{}",
            std::process::id()
        ));
        let saves_dir = instance_root.join("minecraft").join("saves");
        std::fs::create_dir_all(saves_dir.join("Zeta")).unwrap();
        std::fs::create_dir_all(saves_dir.join("Alpha")).unwrap();
        std::fs::write(saves_dir.join("Zeta").join("level.dat"), b"").unwrap();
        std::fs::write(saves_dir.join("Alpha").join("level.dat"), b"").unwrap();
        std::fs::create_dir_all(saves_dir.join("NotAWorld")).unwrap(); // no level.dat
        std::fs::write(saves_dir.join("notes.txt"), b"").unwrap(); // stray file, not a dir

        let worlds = list_worlds(&instance_root).unwrap();

        std::fs::remove_dir_all(&instance_root).unwrap();
        let names: Vec<_> = worlds.iter().map(|w| w.folder_name.clone()).collect();
        assert_eq!(names, vec!["Alpha".to_string(), "Zeta".to_string()]);
    }
}
