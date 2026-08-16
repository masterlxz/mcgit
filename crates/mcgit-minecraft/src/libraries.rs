use crate::manifest::Rule;

/// Mojang's `os.name` values don't match Rust's `std::env::consts::OS` 1:1 —
/// only `"macos"` differs (piston-meta uses `"osx"`). Same translation
/// pattern as `mcgit_java::adoptium`'s `adoptium_os()`.
pub fn piston_os_name() -> &'static str {
    match std::env::consts::OS {
        "macos" => "osx",
        other => other,
    }
}

/// Evaluates a library's `rules` array against `os_name` (passed explicitly
/// rather than read from `env::consts` internally, so this stays testable
/// against every OS from a single test run).
///
/// Mojang's own rule semantics: no rules at all means always allowed.
/// Otherwise, walk the rules in order — a rule with no `os` key matches
/// unconditionally, a rule with an `os.name` only matches that OS — and the
/// *last* matching rule's action decides the outcome.
pub fn is_library_allowed(rules: &[Rule], os_name: &str) -> bool {
    if rules.is_empty() {
        return true;
    }

    let mut allowed = false;
    for rule in rules {
        let matches = match &rule.os {
            Some(os) => os.name.as_deref() == Some(os_name),
            None => true,
        };
        if matches {
            allowed = rule.action == "allow";
        }
    }
    allowed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::OsRule;

    fn allow_rule(os_name: &str) -> Rule {
        Rule {
            action: "allow".to_string(),
            os: Some(OsRule {
                name: Some(os_name.to_string()),
            }),
        }
    }

    #[test]
    fn no_rules_means_always_allowed() {
        assert!(is_library_allowed(&[], "linux"));
        assert!(is_library_allowed(&[], "osx"));
        assert!(is_library_allowed(&[], "windows"));
    }

    #[test]
    fn os_specific_rule_only_allows_matching_os() {
        // The three real `rules` shapes observed live against Minecraft
        // 26.2: single-entry, single-OS "allow" rules.
        let macos_only = [allow_rule("osx")];
        assert!(is_library_allowed(&macos_only, "osx"));
        assert!(!is_library_allowed(&macos_only, "linux"));
        assert!(!is_library_allowed(&macos_only, "windows"));

        let linux_only = [allow_rule("linux")];
        assert!(is_library_allowed(&linux_only, "linux"));
        assert!(!is_library_allowed(&linux_only, "osx"));

        let windows_only = [allow_rule("windows")];
        assert!(is_library_allowed(&windows_only, "windows"));
        assert!(!is_library_allowed(&windows_only, "linux"));
    }

    #[test]
    fn last_matching_rule_wins() {
        let allow_then_deny_for_linux = [
            Rule {
                action: "allow".to_string(),
                os: None,
            },
            Rule {
                action: "disallow".to_string(),
                os: Some(OsRule {
                    name: Some("linux".to_string()),
                }),
            },
        ];
        assert!(!is_library_allowed(&allow_then_deny_for_linux, "linux"));
        assert!(is_library_allowed(&allow_then_deny_for_linux, "osx"));
    }

    #[test]
    fn piston_os_name_translates_macos_to_osx() {
        // Only meaningful on macOS, but always safe to call.
        let name = piston_os_name();
        assert!(name == "osx" || name == "linux" || name == "windows");
    }
}
