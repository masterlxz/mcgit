use std::path::{Path, PathBuf};
use std::process::Command;

use crate::platform;
use crate::types::DetectedJava;
use crate::version_parse::parse_java_version;

/// Detects Java installations on this machine: scans platform-specific
/// candidate locations plus PATH, runs `java -version` against each one,
/// and keeps the ones whose output parses successfully.
pub fn detect_installations() -> Vec<DetectedJava> {
    let mut binaries: Vec<PathBuf> = platform::candidate_locations()
        .iter()
        .filter_map(|candidate| resolve_java_binary(candidate))
        .collect();

    if let Some(bin) = find_on_path() {
        binaries.push(bin);
    }

    binaries.sort();
    binaries.dedup();

    binaries.iter().filter_map(|bin| probe_path(bin)).collect()
}

/// Validates an arbitrary path by running it with `-version` — used both by
/// automatic detection and by a user manually pointing at a Java install.
pub fn probe_path(java_binary: &Path) -> Option<DetectedJava> {
    let output = Command::new(java_binary).arg("-version").output().ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    let (major_version, vendor) = parse_java_version(&stderr).ok()?;

    Some(DetectedJava {
        path: java_binary.to_path_buf(),
        major_version,
        vendor,
    })
}

/// A candidate might already be the `java` binary itself, or a JDK home
/// directory containing `bin/java`.
fn resolve_java_binary(candidate: &Path) -> Option<PathBuf> {
    let exe_name = format!("java{}", std::env::consts::EXE_SUFFIX);

    if candidate.is_file() {
        return Some(candidate.to_path_buf());
    }

    let nested = candidate.join("bin").join(&exe_name);
    if nested.is_file() {
        return Some(nested);
    }

    None
}

fn find_on_path() -> Option<PathBuf> {
    let path_var = std::env::var_os("PATH")?;
    let exe_name = format!("java{}", std::env::consts::EXE_SUFFIX);

    std::env::split_paths(&path_var)
        .map(|dir| dir.join(&exe_name))
        .find(|candidate| candidate.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn resolves_binary_from_jdk_home_directory() {
        let dir = std::env::temp_dir().join(format!("mcgit-test-jdk-home-{}", std::process::id()));
        let bin_dir = dir.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let java_path = bin_dir.join(format!("java{}", std::env::consts::EXE_SUFFIX));
        fs::write(&java_path, b"").unwrap();

        let resolved = resolve_java_binary(&dir);
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(resolved, Some(java_path));
    }

    #[test]
    fn resolves_binary_when_candidate_is_already_the_binary() {
        let path = std::env::temp_dir().join(format!("mcgit-test-java-bin-{}", std::process::id()));
        fs::write(&path, b"").unwrap();

        let resolved = resolve_java_binary(&path);
        fs::remove_file(&path).unwrap();

        assert_eq!(resolved, Some(path));
    }

    #[test]
    fn resolve_returns_none_for_directory_without_bin_java() {
        let dir = std::env::temp_dir().join(format!("mcgit-test-empty-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();

        let resolved = resolve_java_binary(&dir);
        fs::remove_dir_all(&dir).unwrap();

        assert_eq!(resolved, None);
    }
}
