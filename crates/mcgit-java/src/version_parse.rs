use crate::types::JavaError;

/// Parses the text `java -version` prints (to stderr), returning `(major_version, vendor)`.
///
/// Expects two lines like:
///   openjdk version "21.0.4" 2024-07-16
///   OpenJDK Runtime Environment Temurin-21.0.4+7 (build 21.0.4+7)
pub fn parse_java_version(output: &str) -> Result<(u32, String), JavaError> {
    let malformed = || JavaError::UnrecognizedVersionOutput(output.to_string());

    let mut lines = output.lines();

    let version_line = lines.next().ok_or_else(malformed)?;
    let version_string = extract_quoted(version_line).ok_or_else(malformed)?;
    let major = major_from_version_string(&version_string).ok_or_else(malformed)?;

    let vendor_line = lines.next().unwrap_or("");
    let vendor = vendor_from_line(vendor_line);

    Ok((major, vendor))
}

fn extract_quoted(line: &str) -> Option<String> {
    let start = line.find('"')? + 1;
    let rest = &line[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn major_from_version_string(version: &str) -> Option<u32> {
    let mut parts = version.split('.');
    let first: u32 = parts.next()?.parse().ok()?;
    if first == 1 {
        // Old scheme (Java 8 and earlier): "1.8.0_412" -> major is the second component.
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

fn vendor_from_line(line: &str) -> String {
    if line.contains("Temurin") {
        "Eclipse Temurin".to_string()
    } else if line.contains("Corretto") {
        "Amazon Corretto".to_string()
    } else if line.contains("Zulu") {
        "Azul Zulu".to_string()
    } else if line.contains("Microsoft") {
        "Microsoft Build of OpenJDK".to_string()
    } else if line.contains("Java(TM) SE Runtime") {
        "Oracle".to_string()
    } else {
        "OpenJDK".to_string()
    }
}
