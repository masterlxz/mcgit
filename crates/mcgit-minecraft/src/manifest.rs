use serde::Deserialize;

use crate::types::MinecraftError;

const VERSION_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Deserialize)]
pub struct VersionManifestList {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEntry>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub url: String,
    pub sha1: String,
    #[serde(rename = "releaseTime")]
    pub release_time: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionDetail {
    pub id: String,
    pub downloads: Downloads,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    #[serde(rename = "javaVersion")]
    pub java_version: JavaVersionRef,
    pub libraries: Vec<Library>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Downloads {
    pub client: DownloadArtifact,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DownloadArtifact {
    pub url: String,
    pub sha1: String,
    pub size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    #[serde(rename = "totalSize")]
    pub total_size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JavaVersionRef {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: LibraryDownloads,
    /// Absent on most libraries (no OS restriction). `#[serde(default)]`
    /// makes a missing `"rules"` key deserialize to an empty `Vec`, instead
    /// of failing, since the field simply doesn't exist on unrestricted
    /// libraries (confirmed against the live API, not an assumption).
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryDownloads {
    /// `Option` defensively: every library in the live version checked
    /// (26.2, 131 libraries) had this populated, but the field isn't
    /// documented as guaranteed, so a missing artifact is a value to
    /// handle, not a parse failure.
    pub artifact: Option<LibraryArtifact>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsRule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
}

pub async fn fetch_version_manifest(
    client: &reqwest::Client,
) -> Result<VersionManifestList, MinecraftError> {
    let body = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?
        .text()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?;
    parse_version_manifest(&body)
}

pub async fn fetch_version_detail(
    client: &reqwest::Client,
    url: &str,
) -> Result<VersionDetail, MinecraftError> {
    let body = client
        .get(url)
        .send()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?
        .text()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?;
    parse_version_detail(&body)
}

fn parse_version_manifest(body: &str) -> Result<VersionManifestList, MinecraftError> {
    serde_json::from_str(body).map_err(|e| MinecraftError::UnrecognizedApiResponse(e.to_string()))
}

fn parse_version_detail(body: &str) -> Result<VersionDetail, MinecraftError> {
    serde_json::from_str(body).map_err(|e| MinecraftError::UnrecognizedApiResponse(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed from a real `GET version_manifest_v2.json` response captured
    // live in this session (curl, not WebFetch's paraphrased summary).
    const MANIFEST_FIXTURE: &str = r#"{
        "latest": { "release": "26.2", "snapshot": "26.3-snapshot-8" },
        "versions": [
            {
                "id": "26.3-snapshot-8",
                "type": "snapshot",
                "url": "https://piston-meta.mojang.com/v1/packages/082bd3c9b14e99464333d1fbdf910ae055b23265/26.3-snapshot-8.json",
                "time": "2026-08-12T09:48:23+00:00",
                "releaseTime": "2026-08-12T09:39:37+00:00",
                "sha1": "082bd3c9b14e99464333d1fbdf910ae055b23265",
                "complianceLevel": 1
            },
            {
                "id": "26.2",
                "type": "release",
                "url": "https://piston-meta.mojang.com/v1/packages/dc69be58cf16ad99f4b1ae7360c9a29c8c819ca5/26.2.json",
                "time": "2026-08-12T06:45:43+00:00",
                "releaseTime": "2026-06-16T12:03:33+00:00",
                "sha1": "dc69be58cf16ad99f4b1ae7360c9a29c8c819ca5",
                "complianceLevel": 1
            }
        ]
    }"#;

    // Trimmed from a real `GET .../26.2.json` response captured live in this
    // session, keeping the two real library entries that matter for the
    // `rules`-filtering logic in `libraries.rs` (one unrestricted, one
    // macOS-only).
    const DETAIL_FIXTURE: &str = r#"{
        "id": "26.2",
        "downloads": {
            "client": {
                "sha1": "2dc72797acbc1b63fc16a11c4ac393605f453754",
                "size": 39193383,
                "url": "https://piston-data.mojang.com/v1/objects/2dc72797acbc1b63fc16a11c4ac393605f453754/client.jar"
            }
        },
        "assetIndex": {
            "id": "32",
            "sha1": "cf75b185cb35b32e299b0c8e674fa202d7911a3c",
            "size": 586366,
            "totalSize": 478689403,
            "url": "https://piston-meta.mojang.com/v1/packages/cf75b185cb35b32e299b0c8e674fa202d7911a3c/32.json"
        },
        "javaVersion": { "component": "java-runtime-epsilon", "majorVersion": 25 },
        "libraries": [
            {
                "downloads": {
                    "artifact": {
                        "path": "at/yawk/lz4/lz4-java/1.10.1/lz4-java-1.10.1.jar",
                        "sha1": "f541d7f910fe3d76f38f799c507c48cc81b12ecb",
                        "size": 910232,
                        "url": "https://libraries.minecraft.net/at/yawk/lz4/lz4-java/1.10.1/lz4-java-1.10.1.jar"
                    }
                },
                "name": "at.yawk.lz4:lz4-java:1.10.1"
            },
            {
                "downloads": {
                    "artifact": {
                        "path": "ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar",
                        "sha1": "1227f9e0666314f9de41477e3ec277e542ed7f7b",
                        "size": 1330045,
                        "url": "https://libraries.minecraft.net/ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar"
                    }
                },
                "name": "ca.weblite:java-objc-bridge:1.1",
                "rules": [ { "action": "allow", "os": { "name": "osx" } } ]
            }
        ]
    }"#;

    #[test]
    fn parses_version_manifest_fixture() {
        let manifest = parse_version_manifest(MANIFEST_FIXTURE).unwrap();
        assert_eq!(manifest.latest.release, "26.2");
        assert_eq!(manifest.versions.len(), 2);
        assert_eq!(manifest.versions[1].id, "26.2");
        assert_eq!(manifest.versions[1].kind, "release");
    }

    #[test]
    fn parses_version_detail_fixture() {
        let detail = parse_version_detail(DETAIL_FIXTURE).unwrap();
        assert_eq!(detail.id, "26.2");
        assert_eq!(detail.downloads.client.size, 39193383);
        assert_eq!(detail.asset_index.total_size, 478689403);
        assert_eq!(detail.java_version.major_version, 25);
        assert_eq!(detail.libraries.len(), 2);

        let unrestricted = &detail.libraries[0];
        assert!(unrestricted.rules.is_empty());
        assert_eq!(
            unrestricted.downloads.artifact.as_ref().unwrap().path,
            "at/yawk/lz4/lz4-java/1.10.1/lz4-java-1.10.1.jar"
        );

        let macos_only = &detail.libraries[1];
        assert_eq!(macos_only.rules.len(), 1);
        assert_eq!(macos_only.rules[0].action, "allow");
        assert_eq!(macos_only.rules[0].os.as_ref().unwrap().name.as_deref(), Some("osx"));
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_version_manifest("{ not json").is_err());
        assert!(parse_version_detail("{ not json").is_err());
    }
}
