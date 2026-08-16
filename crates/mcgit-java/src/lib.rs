pub mod adoptium;
pub mod archive;
pub mod detect;
pub mod install;
pub mod platform;
pub mod types;
pub mod version_parse;

#[cfg(test)]
mod tests {
    use super::platform;

    #[test]
    fn candidate_locations_does_not_panic() {
        // This dev machine has no /usr/lib/jvm and no JAVA_HOME, so we can only
        // assert the function runs cleanly — not that it finds anything specific.
        let _candidates = platform::candidate_locations();
    }
}
