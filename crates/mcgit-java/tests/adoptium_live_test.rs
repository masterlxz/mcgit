use mcgit_java::adoptium::{available_releases, latest_asset};

/// Ignored by default — hits the real Adoptium API over the network. Run manually with:
///   cargo test -p mcgit-java -- --ignored adoptium --nocapture
#[tokio::test]
#[ignore]
async fn fetches_real_available_releases() {
    let releases = available_releases().await.unwrap();
    println!("{releases:#?}");
    assert!(releases.available_lts_releases.contains(&21));
}

#[tokio::test]
#[ignore]
async fn fetches_real_latest_asset_for_java_21() {
    let asset = latest_asset(21).await.unwrap();
    println!("{asset:#?}");
    assert!(asset.binary.package.link.starts_with("https://"));
}
