use mcgit_java::install::download_and_install;

/// Ignored by default — downloads a real ~200MB JDK over the network. Run manually with:
///   cargo test -p mcgit-java -- --ignored installs_real_jdk_21 --nocapture
#[tokio::test]
#[ignore]
async fn installs_real_jdk_21() {
    let install_root =
        std::env::temp_dir().join(format!("mcgit-test-install-root-{}", std::process::id()));

    let result = download_and_install(21, &install_root, |stage| {
        println!("{stage:?}");
    })
    .await
    .unwrap();

    assert_eq!(result.major_version, 21);
    assert!(result.path.is_file());

    let output = std::process::Command::new(&result.path)
        .arg("-version")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("java -version output:\n{stderr}");
    assert!(stderr.contains("21."));

    std::fs::remove_dir_all(&install_root).unwrap();
}
