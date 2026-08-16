use std::io::{Read, Write};
use std::path::Path;

use futures_util::StreamExt;
use sha2::{Digest, Sha256};

use crate::adoptium::{self, JavaAsset};
use crate::archive;
use crate::types::{DetectedJava, InstallStage, JavaError};

/// Downloads, verifies, extracts, and installs the latest Temurin build for
/// `major_version` into `install_root/<major_version>/`, reporting progress
/// through `on_progress`. Returns the installed binary's location.
pub async fn download_and_install(
    major_version: u32,
    install_root: &Path,
    mut on_progress: impl FnMut(InstallStage),
) -> Result<DetectedJava, JavaError> {
    let asset = adoptium::latest_asset(major_version).await?;

    let download_dir = install_root.join("downloads");
    std::fs::create_dir_all(&download_dir).map_err(|e| JavaError::Io(e.to_string()))?;
    let archive_path = download_dir.join(&asset.binary.package.name);

    download_with_progress(&asset, &archive_path, &mut on_progress).await?;

    on_progress(InstallStage::Verifying);
    verify_checksum(&archive_path, &asset.binary.package.checksum)?;

    on_progress(InstallStage::Extracting);
    let version_dir = install_root.join(major_version.to_string());
    if version_dir.exists() {
        std::fs::remove_dir_all(&version_dir).map_err(|e| JavaError::Io(e.to_string()))?;
    }
    archive::extract(&archive_path, &version_dir)?;
    std::fs::remove_file(&archive_path).ok(); // best-effort cleanup, not fatal if it fails

    let java_binary = archive::find_java_binary(&version_dir).ok_or_else(|| {
        JavaError::UnrecognizedApiResponse(
            "could not locate java binary after extraction".to_string(),
        )
    })?;

    on_progress(InstallStage::Done);

    Ok(DetectedJava {
        path: java_binary,
        major_version,
        vendor: display_vendor(&asset.vendor),
    })
}

fn display_vendor(adoptium_vendor: &str) -> String {
    match adoptium_vendor {
        "eclipse" => "Eclipse Temurin".to_string(),
        other => other.to_string(),
    }
}

/// Downloads `asset`'s package to `destination`, calling `on_progress` after
/// every chunk written. Streams to disk instead of buffering the whole file
/// in memory.
pub(crate) async fn download_with_progress(
    asset: &JavaAsset,
    destination: &Path,
    on_progress: &mut impl FnMut(InstallStage),
) -> Result<(), JavaError> {
    let response = reqwest::get(&asset.binary.package.link)
        .await
        .map_err(|e| JavaError::Network(e.to_string()))?;

    let total = asset.binary.package.size;
    let mut file = std::fs::File::create(destination).map_err(|e| JavaError::Io(e.to_string()))?;
    let mut downloaded: u64 = 0;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| JavaError::Network(e.to_string()))?;
        file.write_all(&chunk).map_err(|e| JavaError::Io(e.to_string()))?;
        downloaded += chunk.len() as u64;
        on_progress(InstallStage::Downloading {
            bytes_done: downloaded,
            bytes_total: total,
        });
    }

    Ok(())
}

/// Verifies that the file at `path` matches the expected sha256 checksum
/// (hex-encoded, as Adoptium reports it). Reads the file in one streamed
/// pass rather than loading it into memory.
pub(crate) fn verify_checksum(path: &Path, expected_hex: &str) -> Result<(), JavaError> {
    let mut file = std::fs::File::open(path).map_err(|e| JavaError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| JavaError::Io(e.to_string()))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    let digest = hasher.finalize();
    let actual_hex: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();

    if actual_hex.eq_ignore_ascii_case(expected_hex) {
        Ok(())
    } else {
        Err(JavaError::ChecksumMismatch {
            expected: expected_hex.to_string(),
            actual: actual_hex,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_checksum_rejects_mismatched_hash() {
        let path = std::env::temp_dir().join(format!("mcgit-test-checksum-bad-{}", std::process::id()));
        std::fs::write(&path, b"hello mcgit").unwrap();

        let result = verify_checksum(&path, "0000000000000000000000000000000000000000000000000000000000000");

        std::fs::remove_file(&path).unwrap();

        assert!(matches!(result, Err(JavaError::ChecksumMismatch { .. })));
    }

    #[test]
    fn verify_checksum_accepts_actual_computed_hash() {
        let path = std::env::temp_dir().join(format!("mcgit-test-checksum-real-{}", std::process::id()));
        std::fs::write(&path, b"hello mcgit").unwrap();

        let mut hasher = Sha256::new();
        hasher.update(b"hello mcgit");
        let expected: String = hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect();

        let result = verify_checksum(&path, &expected);
        std::fs::remove_file(&path).unwrap();

        assert!(result.is_ok());
    }
}
