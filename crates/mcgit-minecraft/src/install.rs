use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use futures_util::{stream, StreamExt};
use sha1::{Digest, Sha1};

use crate::assets::{self, AssetIndex, AssetObject};
use crate::libraries;
use crate::manifest::{Library, LibraryArtifact, VersionDetail};
use crate::types::{InstallStage, MinecraftError};

const LIBRARY_CONCURRENCY: usize = 12;
const ASSET_CONCURRENCY: usize = 12;

/// Downloads the client jar, the OS-appropriate libraries, and every asset
/// for `detail` into the shared cache under `cache_dir`, skipping any file
/// whose sha1 already matches what's on disk.
///
/// Does NOT fetch `detail` itself — the caller (the Tauri command layer)
/// fetches the manifest/version-detail first, since it needs
/// `detail.java_version.major_version` to resolve/install the right JDK via
/// `mcgit-java` *before* this runs, and `mcgit-minecraft` must not depend on
/// `mcgit-java` (only the Tauri layer is allowed to wire both together).
pub async fn download_files(
    client: &reqwest::Client,
    detail: &VersionDetail,
    cache_dir: &Path,
    mut on_progress: impl FnMut(InstallStage),
) -> Result<PathBuf, MinecraftError> {
    let client_jar_path = download_client(client, detail, cache_dir, &mut on_progress).await?;

    let os_name = libraries::piston_os_name();
    let wanted: Vec<&Library> = detail
        .libraries
        .iter()
        .filter(|lib| libraries::is_library_allowed(&lib.rules, os_name))
        .collect();
    download_libraries(client, &wanted, cache_dir, &mut on_progress).await?;

    let asset_index = assets::fetch_asset_index(client, &detail.asset_index).await?;
    download_assets(client, &asset_index, cache_dir, &mut on_progress).await?;

    on_progress(InstallStage::Verifying);
    on_progress(InstallStage::Done);
    Ok(client_jar_path)
}

async fn download_client(
    client: &reqwest::Client,
    detail: &VersionDetail,
    cache_dir: &Path,
    on_progress: &mut impl FnMut(InstallStage),
) -> Result<PathBuf, MinecraftError> {
    let dest = cache_dir.join("versions").join(&detail.id).join("client.jar");
    let artifact = &detail.downloads.client;

    if is_cached(&dest, &artifact.sha1) {
        on_progress(InstallStage::DownloadingClient {
            bytes_done: artifact.size,
            bytes_total: artifact.size,
        });
        return Ok(dest);
    }

    create_parent_dir(&dest)?;

    let mut throttle = ProgressThrottle::new();
    download_verified_streaming(
        client,
        &artifact.url,
        &dest,
        &artifact.sha1,
        |done, total| {
            if throttle.should_emit(done) {
                on_progress(InstallStage::DownloadingClient {
                    bytes_done: done,
                    bytes_total: total,
                });
            }
        },
    )
    .await?;

    on_progress(InstallStage::DownloadingClient {
        bytes_done: artifact.size,
        bytes_total: artifact.size,
    });
    Ok(dest)
}

/// Bounded-concurrency download of every library's artifact. Reports
/// progress per completed file (libraries are individually small — unlike
/// the client jar, byte-level granularity within one file isn't useful
/// here), throttled the same way as the client download.
async fn download_libraries(
    client: &reqwest::Client,
    libs: &[&Library],
    cache_dir: &Path,
    on_progress: &mut impl FnMut(InstallStage),
) -> Result<(), MinecraftError> {
    // Owned (not borrowed) artifacts: a closure that captures a reference
    // borrowed from this vec's iterator, returning an `impl Future`, hits a
    // known rustc HRTB inference limit ("implementation of `FnOnce` is not
    // general enough") once wrapped in `buffer_unordered` inside a
    // `#[tauri::command]`. Cloning up front sidesteps it.
    let with_artifact: Vec<LibraryArtifact> = libs
        .iter()
        .filter_map(|lib| lib.downloads.artifact.clone())
        .collect();
    let files_total = with_artifact.len() as u32;
    let bytes_total: u64 = with_artifact.iter().map(|a| a.size).sum();

    on_progress(InstallStage::DownloadingLibraries {
        files_done: 0,
        files_total,
        bytes_done: 0,
        bytes_total,
    });
    if files_total == 0 {
        return Ok(());
    }

    let mut files_done = 0u32;
    let mut bytes_done = 0u64;
    let mut throttle = ProgressThrottle::new();

    let mut results = stream::iter(with_artifact.into_iter().map(|artifact| {
        let dest = cache_dir.join("libraries").join(&artifact.path);
        download_one_artifact(client, artifact, dest)
    }))
    .buffer_unordered(LIBRARY_CONCURRENCY);

    while let Some(result) = results.next().await {
        let downloaded_bytes = result?;
        files_done += 1;
        bytes_done += downloaded_bytes;
        if throttle.should_emit(bytes_done) || files_done == files_total {
            on_progress(InstallStage::DownloadingLibraries {
                files_done,
                files_total,
                bytes_done,
                bytes_total,
            });
        }
    }

    Ok(())
}

async fn download_one_artifact(
    client: &reqwest::Client,
    artifact: LibraryArtifact,
    dest: PathBuf,
) -> Result<u64, MinecraftError> {
    if is_cached(&dest, &artifact.sha1) {
        return Ok(artifact.size);
    }
    create_parent_dir(&dest)?;
    download_verified_streaming(client, &artifact.url, &dest, &artifact.sha1, |_, _| {}).await?;
    Ok(artifact.size)
}

/// Same bounded-concurrency, per-file-granularity approach as
/// `download_libraries`, keyed by asset hash instead of Maven path.
async fn download_assets(
    client: &reqwest::Client,
    index: &AssetIndex,
    cache_dir: &Path,
    on_progress: &mut impl FnMut(InstallStage),
) -> Result<(), MinecraftError> {
    // Owned, same reasoning as `download_libraries` above.
    let objects: Vec<AssetObject> = index.objects.values().cloned().collect();
    let files_total = objects.len() as u32;
    let bytes_total: u64 = objects.iter().map(|o| o.size).sum();

    on_progress(InstallStage::DownloadingAssets {
        files_done: 0,
        files_total,
        bytes_done: 0,
        bytes_total,
    });
    if files_total == 0 {
        return Ok(());
    }

    let mut files_done = 0u32;
    let mut bytes_done = 0u64;
    let mut throttle = ProgressThrottle::new();
    let cache_dir = cache_dir.to_path_buf();

    let mut results = stream::iter(
        objects
            .into_iter()
            .map(|obj| download_one_asset(client, obj, cache_dir.clone())),
    )
    .buffer_unordered(ASSET_CONCURRENCY);

    while let Some(result) = results.next().await {
        let downloaded_bytes = result?;
        files_done += 1;
        bytes_done += downloaded_bytes;
        if throttle.should_emit(bytes_done) || files_done == files_total {
            on_progress(InstallStage::DownloadingAssets {
                files_done,
                files_total,
                bytes_done,
                bytes_total,
            });
        }
    }

    Ok(())
}

async fn download_one_asset(
    client: &reqwest::Client,
    object: AssetObject,
    cache_dir: PathBuf,
) -> Result<u64, MinecraftError> {
    let dest = assets::asset_cache_path(&cache_dir, &object.hash);
    if is_cached(&dest, &object.hash) {
        return Ok(object.size);
    }
    create_parent_dir(&dest)?;
    let url = assets::asset_download_url(&object.hash);
    download_verified_streaming(client, &url, &dest, &object.hash, |_, _| {}).await?;
    Ok(object.size)
}

fn create_parent_dir(path: &Path) -> Result<(), MinecraftError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| MinecraftError::Io(e.to_string()))?;
    }
    Ok(())
}

/// Streams `url` to `dest`, calling `on_chunk(bytes_done, bytes_total)` as
/// data arrives, then verifies the written file's sha1 against
/// `expected_sha1`. Deletes the file and returns `ChecksumMismatch` on a
/// mismatch, rather than leaving a corrupt file behind for a later cache
/// check to wrongly trust.
async fn download_verified_streaming(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    expected_sha1: &str,
    mut on_chunk: impl FnMut(u64, u64),
) -> Result<(), MinecraftError> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?;
    let total = response.content_length().unwrap_or(0);

    let mut file = std::fs::File::create(dest).map_err(|e| MinecraftError::Io(e.to_string()))?;
    let mut hasher = Sha1::new();
    let mut downloaded: u64 = 0;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| MinecraftError::Network(e.to_string()))?;
        file.write_all(&chunk).map_err(|e| MinecraftError::Io(e.to_string()))?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        on_chunk(downloaded, total.max(downloaded));
    }

    let actual = hex_digest(hasher);
    if !actual.eq_ignore_ascii_case(expected_sha1) {
        std::fs::remove_file(dest).ok(); // best-effort cleanup, not fatal if it fails
        return Err(MinecraftError::ChecksumMismatch {
            path: dest.display().to_string(),
            expected: expected_sha1.to_string(),
            actual,
        });
    }
    Ok(())
}

/// A file counts as cached only if it exists AND its sha1 matches — guards
/// against trusting a prior partial/corrupt write.
fn is_cached(path: &Path, expected_sha1: &str) -> bool {
    if !path.exists() {
        return false;
    }
    match compute_sha1(path) {
        Ok(actual) => actual.eq_ignore_ascii_case(expected_sha1),
        Err(_) => false,
    }
}

fn compute_sha1(path: &Path) -> Result<String, MinecraftError> {
    let mut file = std::fs::File::open(path).map_err(|e| MinecraftError::Io(e.to_string()))?;
    let mut hasher = Sha1::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| MinecraftError::Io(e.to_string()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    Ok(hex_digest(hasher))
}

fn hex_digest(hasher: Sha1) -> String {
    hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Caps how often progress callbacks actually fire: at most once per ~100ms
/// OR once per ~1MB of additional bytes, whichever comes first. Direct fix
/// for the Java Manager's known tech debt (thousands of unthrottled
/// tiny-chunk progress events per install) — getting it right from the
/// start here, at a scale (hundreds/thousands of files) where it would
/// otherwise be much worse.
struct ProgressThrottle {
    last_emit: Instant,
    last_bytes: u64,
}

impl ProgressThrottle {
    fn new() -> Self {
        Self {
            last_emit: Instant::now(),
            last_bytes: 0,
        }
    }

    fn should_emit(&mut self, bytes_done: u64) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_emit);
        let byte_delta = bytes_done.saturating_sub(self.last_bytes);
        if elapsed >= Duration::from_millis(100) || byte_delta >= 1_000_000 {
            self.last_emit = now;
            self.last_bytes = bytes_done;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_throttle_suppresses_frequent_small_updates() {
        let mut throttle = ProgressThrottle::new();
        assert!(!throttle.should_emit(100));
        assert!(!throttle.should_emit(200));
        // Crossed the 1MB-since-last-emit threshold.
        assert!(throttle.should_emit(1_100_000));
    }

    #[test]
    fn is_cached_true_only_when_hash_matches() {
        let path = std::env::temp_dir().join(format!(
            "mcgit-minecraft-test-cache-{}",
            std::process::id()
        ));
        std::fs::write(&path, b"hello mcgit").unwrap();

        let mut hasher = Sha1::new();
        hasher.update(b"hello mcgit");
        let real_hash = hex_digest(hasher);

        assert!(is_cached(&path, &real_hash));
        assert!(!is_cached(&path, "0000000000000000000000000000000000"));

        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn is_cached_false_when_file_missing() {
        let path = std::env::temp_dir().join(format!(
            "mcgit-minecraft-test-missing-{}",
            std::process::id()
        ));
        assert!(!is_cached(&path, "anything"));
    }
}
