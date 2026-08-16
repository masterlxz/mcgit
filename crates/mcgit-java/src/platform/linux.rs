use std::path::PathBuf;

/// Directories where a Java installation might live on Linux.
/// Doesn't validate anything yet — just gathers candidates to check later.
pub fn candidate_locations() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Ok(entries) = std::fs::read_dir("/usr/lib/jvm") {
        for entry in entries.flatten() {
            candidates.push(entry.path());
        }
    }

    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        candidates.push(PathBuf::from(java_home));
    }

    candidates
}
