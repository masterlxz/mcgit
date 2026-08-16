//! Live network tests against the real Mojang piston-meta API. Not run by
//! default `cargo test` — run manually with:
//!   cargo test -p mcgit-minecraft -- --ignored live_install --nocapture
//!
//! Uses "rd-132211", a ~2009 pre-classic prototype release: a 26KB client
//! jar and ~49MB of assets, real but far smaller than a modern version
//! (~40MB client + ~480MB assets) — check free disk space before running.

use mcgit_minecraft::manifest;
use mcgit_minecraft::types::InstallStage;

#[tokio::test]
#[ignore]
async fn live_install_downloads_and_verifies_a_real_small_version() {
    let client = reqwest::Client::new();

    let list = manifest::fetch_version_manifest(&client)
        .await
        .expect("could not fetch version manifest");
    let entry = list
        .versions
        .iter()
        .find(|v| v.id == "rd-132211")
        .expect("rd-132211 missing from the live version manifest");

    let detail = manifest::fetch_version_detail(&client, &entry.url)
        .await
        .expect("could not fetch version detail");
    assert_eq!(detail.java_version.major_version, 8);

    let cache_dir = std::env::temp_dir().join(format!(
        "mcgit-minecraft-live-test-{}",
        std::process::id()
    ));

    let mut saw_done = false;
    let client_jar = mcgit_minecraft::install::download_files(&client, &detail, &cache_dir, |stage| {
        if stage == InstallStage::Done {
            saw_done = true;
        }
    })
    .await
    .expect("download_files failed");

    assert!(client_jar.exists());
    assert!(saw_done);

    std::fs::remove_dir_all(&cache_dir).ok();
}
