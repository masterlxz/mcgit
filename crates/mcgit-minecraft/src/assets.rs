use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::manifest::AssetIndexRef;
use crate::types::MinecraftError;

#[derive(Debug, Clone, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

pub async fn fetch_asset_index(
    client: &reqwest::Client,
    index_ref: &AssetIndexRef,
) -> Result<AssetIndex, MinecraftError> {
    let body = client
        .get(&index_ref.url)
        .send()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?
        .text()
        .await
        .map_err(|e| MinecraftError::Network(e.to_string()))?;
    parse_asset_index(&body)
}

fn parse_asset_index(body: &str) -> Result<AssetIndex, MinecraftError> {
    serde_json::from_str(body).map_err(|e| MinecraftError::UnrecognizedApiResponse(e.to_string()))
}

/// Mojang's fixed resource-download convention — content-addressed by the
/// object's own sha1, not part of the JSON payload itself. Verified live in
/// this session: a real hash from the asset index resolved with HTTP 200
/// and the exact declared byte size.
pub fn asset_download_url(hash: &str) -> String {
    format!("https://resources.download.minecraft.net/{}/{}", &hash[0..2], hash)
}

/// Same `<hash[0:2]>/<hash>` layout, mirrored locally under the shared cache
/// directory so a cache hit is a plain file-existence + checksum check.
pub fn asset_cache_path(cache_dir: &Path, hash: &str) -> PathBuf {
    cache_dir
        .join("assets")
        .join("objects")
        .join(&hash[0..2])
        .join(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Trimmed from the real asset index (id "32") referenced by Minecraft
    // 26.2's version detail, captured live in this session.
    const ASSET_INDEX_FIXTURE: &str = r#"{
        "objects": {
            "icons/icon_128x128.png": { "hash": "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356", "size": 9101 },
            "icons/icon_16x16.png": { "hash": "5ff04807c356f1beed0b86ccf659b44b9983e3fa", "size": 781 }
        }
    }"#;

    #[test]
    fn parses_asset_index_fixture() {
        let index = parse_asset_index(ASSET_INDEX_FIXTURE).unwrap();
        assert_eq!(index.objects.len(), 2);
        let icon = &index.objects["icons/icon_128x128.png"];
        assert_eq!(icon.hash, "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356");
        assert_eq!(icon.size, 9101);
    }

    #[test]
    fn rejects_malformed_json() {
        assert!(parse_asset_index("{ not json").is_err());
    }

    #[test]
    fn asset_download_url_uses_content_addressed_layout() {
        let hash = "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356";
        assert_eq!(
            asset_download_url(hash),
            "https://resources.download.minecraft.net/b6/b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356"
        );
    }

    #[test]
    fn asset_cache_path_mirrors_the_same_layout_locally() {
        let cache_dir = Path::new("/tmp/mcgit-cache");
        let hash = "b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356";
        assert_eq!(
            asset_cache_path(cache_dir, hash),
            Path::new("/tmp/mcgit-cache/assets/objects/b6/b62ca8ec10d07e6bf5ac8dae0c8c1d2e6a1e3356")
        );
    }
}
