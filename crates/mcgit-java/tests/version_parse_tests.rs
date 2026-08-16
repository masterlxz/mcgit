use mcgit_java::version_parse::parse_java_version;

#[test]
fn parses_temurin_21() {
    let output = "openjdk version \"21.0.4\" 2024-07-16\n\
                   OpenJDK Runtime Environment Temurin-21.0.4+7 (build 21.0.4+7)\n\
                   OpenJDK 64-Bit Server VM Temurin-21.0.4+7 (build 21.0.4+7, mixed mode, sharing)\n";

    let (major, vendor) = parse_java_version(output).unwrap();
    assert_eq!(major, 21);
    assert_eq!(vendor, "Eclipse Temurin");
}

#[test]
fn parses_temurin_8_old_version_scheme() {
    let output = "openjdk version \"1.8.0_412\"\n\
                   OpenJDK Runtime Environment Temurin-8.0.412+8 (build 1.8.0_412-b08)\n\
                   OpenJDK 64-Bit Server VM Temurin-8.0.412+8 (build 25.412-b08, mixed mode)\n";

    let (major, vendor) = parse_java_version(output).unwrap();
    assert_eq!(major, 8);
    assert_eq!(vendor, "Eclipse Temurin");
}

#[test]
fn parses_oracle_jdk() {
    let output = "java version \"17.0.9\" 2023-10-17 LTS\n\
                   Java(TM) SE Runtime Environment (build 17.0.9+11-LTS-201)\n\
                   Java HotSpot(TM) 64-Bit Server VM (build 17.0.9+11-LTS-201, mixed mode, sharing)\n";

    let (major, vendor) = parse_java_version(output).unwrap();
    assert_eq!(major, 17);
    assert_eq!(vendor, "Oracle");
}

#[test]
fn parses_generic_openjdk_debian_build() {
    let output = "openjdk version \"17.0.9\" 2023-10-17\n\
                   OpenJDK Runtime Environment (build 17.0.9+9-Debian-1)\n\
                   OpenJDK 64-Bit Server VM (build 17.0.9+9-Debian-1, mixed mode, sharing)\n";

    let (major, vendor) = parse_java_version(output).unwrap();
    assert_eq!(major, 17);
    assert_eq!(vendor, "OpenJDK");
}

#[test]
fn parses_amazon_corretto() {
    let output = "openjdk version \"21.0.4\" 2024-07-16 LTS\n\
                   OpenJDK Runtime Environment Corretto-21.0.4.7.1 (build 21.0.4+7-LTS)\n\
                   OpenJDK 64-Bit Server VM Corretto-21.0.4.7.1 (build 21.0.4+7-LTS, mixed mode, sharing)\n";

    let (major, vendor) = parse_java_version(output).unwrap();
    assert_eq!(major, 21);
    assert_eq!(vendor, "Amazon Corretto");
}

#[test]
fn rejects_garbage_output() {
    let result = parse_java_version("not actually java output\nsecond line\n");
    assert!(result.is_err());
}
