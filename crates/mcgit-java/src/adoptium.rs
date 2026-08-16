use serde::Deserialize;

use crate::types::JavaError;

const API_BASE: &str = "https://api.adoptium.net/v3";

#[derive(Debug, Deserialize)]
pub struct AvailableReleases {
    pub available_releases: Vec<u32>,
    pub available_lts_releases: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaAsset {
    pub vendor: String,
    pub version: AssetVersion,
    pub binary: AssetBinary,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetVersion {
    pub openjdk_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetBinary {
    pub package: AssetPackage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetPackage {
    pub name: String,
    pub link: String,
    pub checksum: String,
    pub size: u64,
}

/// Rust's `std::env::consts::OS` doesn't match Adoptium's naming 1:1 — only "macos" differs.
fn adoptium_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "mac",
        other => other,
    }
}

/// Rust's `std::env::consts::ARCH` doesn't match Adoptium's naming 1:1 — only "x86_64" differs.
fn adoptium_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        other => other,
    }
}

fn parse_available_releases(body: &str) -> Result<AvailableReleases, JavaError> {
    serde_json::from_str(body).map_err(|e| JavaError::UnrecognizedApiResponse(e.to_string()))
}

fn parse_asset_list(body: &str) -> Result<Vec<JavaAsset>, JavaError> {
    serde_json::from_str(body).map_err(|e| JavaError::UnrecognizedApiResponse(e.to_string()))
}

/// GET /v3/info/available_releases
pub async fn available_releases() -> Result<AvailableReleases, JavaError> {
    let url = format!("{API_BASE}/info/available_releases");
    let body = fetch(&url).await?;
    parse_available_releases(&body)
}

/// GET /v3/assets/latest/{feature_version}/hotspot?image_type=jdk&os=...&architecture=...
/// Returns the first asset in Adoptium's response list, matching this machine's OS/architecture.
pub async fn latest_asset(feature_version: u32) -> Result<JavaAsset, JavaError> {
    let url = format!(
        "{API_BASE}/assets/latest/{feature_version}/hotspot?image_type=jdk&os={}&architecture={}",
        adoptium_os(),
        adoptium_arch(),
    );
    let body = fetch(&url).await?;
    let assets = parse_asset_list(&body)?;
    assets
        .into_iter()
        .next()
        .ok_or_else(|| JavaError::UnrecognizedApiResponse("empty asset list".to_string()))
}

async fn fetch(url: &str) -> Result<String, JavaError> {
    reqwest::get(url)
        .await
        .map_err(|e| JavaError::Network(e.to_string()))?
        .text()
        .await
        .map_err(|e| JavaError::Network(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_available_releases_fixture() {
        let body = r#"{
            "available_releases": [8, 11, 16, 17, 18, 19, 20, 21],
            "available_lts_releases": [8, 11, 17, 21],
            "most_recent_lts": 21,
            "most_recent_feature_release": 21,
            "tip_version": 24
        }"#;

        let parsed = parse_available_releases(body).unwrap();
        assert_eq!(parsed.available_lts_releases, vec![8, 11, 17, 21]);
    }

    #[test]
    fn parses_asset_list_fixture() {
        let body = r#"[{
            "vendor": "eclipse",
            "version": { "openjdk_version": "21.0.4+7" },
            "binary": {
                "package": {
                    "name": "OpenJDK21U-jdk_x64_linux_hotspot_21.0.4_7.tar.gz",
                    "link": "https://example.com/OpenJDK21U-jdk_x64_linux_hotspot_21.0.4_7.tar.gz",
                    "checksum": "abc123",
                    "size": 207486543
                }
            }
        }]"#;

        let assets = parse_asset_list(body).unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].vendor, "eclipse");
        assert_eq!(assets[0].binary.package.checksum, "abc123");
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_available_releases("not json").is_err());
        assert!(parse_asset_list("not json").is_err());
    }
}
