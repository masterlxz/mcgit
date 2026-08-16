use std::path::{Path, PathBuf};

use crate::types::JavaError;

#[cfg(not(target_os = "windows"))]
pub fn extract(archive_path: &Path, destination: &Path) -> Result<(), JavaError> {
    let file = std::fs::File::open(archive_path).map_err(|e| JavaError::Io(e.to_string()))?;
    let decompressed = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(decompressed);
    archive
        .unpack(destination)
        .map_err(|e| JavaError::Io(e.to_string()))?;
    Ok(())
}

#[cfg(target_os = "windows")]
pub fn extract(archive_path: &Path, destination: &Path) -> Result<(), JavaError> {
    let file = std::fs::File::open(archive_path).map_err(|e| JavaError::Io(e.to_string()))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| JavaError::Io(e.to_string()))?;
    archive
        .extract(destination)
        .map_err(|e| JavaError::Io(e.to_string()))?;
    Ok(())
}

/// Walks the extracted tree looking for a `bin/java[.exe]` binary — the
/// archive's top-level folder name varies by version (e.g. `jdk-21.0.12+8`),
/// so we can't assume a fixed relative path.
pub fn find_java_binary(root: &Path) -> Option<PathBuf> {
    let exe_name = format!("java{}", std::env::consts::EXE_SUFFIX);
    find_recursive(root, &exe_name, 4)
}

fn find_recursive(dir: &Path, exe_name: &str, remaining_depth: u32) -> Option<PathBuf> {
    if remaining_depth == 0 {
        return None;
    }

    let entries = std::fs::read_dir(dir).ok()?;
    let mut subdirs = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            subdirs.push(path);
        } else if path.is_file()
            && path.file_name().and_then(|n| n.to_str()) == Some(exe_name)
            && path
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                == Some("bin")
        {
            return Some(path);
        }
    }

    for subdir in subdirs {
        if let Some(found) = find_recursive(&subdir, exe_name, remaining_depth - 1) {
            return Some(found);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_binary_nested_inside_extracted_tree() {
        let root = std::env::temp_dir().join(format!("mcgit-test-extract-{}", std::process::id()));
        let bin_dir = root.join("jdk-21.0.12+8").join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let java_path = bin_dir.join(format!("java{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&java_path, b"").unwrap();

        let found = find_java_binary(&root);
        std::fs::remove_dir_all(&root).unwrap();

        assert_eq!(found, Some(java_path));
    }

    #[test]
    fn returns_none_when_no_binary_present() {
        let root = std::env::temp_dir().join(format!("mcgit-test-extract-empty-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();

        let found = find_java_binary(&root);
        std::fs::remove_dir_all(&root).unwrap();

        assert_eq!(found, None);
    }
}
