use mcgit_java::detect::detect_installations;

/// Ignored by default — depends on real Java being installed on the machine
/// running the test. Run manually with:
///   cargo test -p mcgit-java -- --ignored detect_installations_finds_real_java --nocapture
/// and eyeball the printed list against `java -version` run by hand.
#[test]
#[ignore]
fn detect_installations_finds_real_java() {
    let found = detect_installations();
    println!("{found:#?}");
    assert!(
        !found.is_empty(),
        "expected to find at least one Java installation on this machine"
    );
}
