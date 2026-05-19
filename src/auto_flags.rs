//! Auto-inject default flags from per-subcommand env vars.
//!
//! For each subcommand path (e.g. `services people`), an env var of the form
//! `ELVANTO_<PATH_JOINED_BY_UNDERSCORE>` (here: `ELVANTO_SERVICES_PEOPLE`)
//! contains shell-tokenised flags that are inserted into argv after the
//! subcommand name — but only if:
//!   * the user did NOT pass `--no-env`, AND
//!   * the user did NOT manually provide any `-`-prefixed flag *after* the
//!     subcommand path.
//!
//! Recognised subcommand groups (where the first non-flag token is followed by
//! a second non-flag token forming a two-level path): `auth`, `services`,
//! `songs`. Other tokens are treated as a single-level path.

const KNOWN_GROUPS: &[&str] = &["auth", "people", "services", "songs"];

/// Apply env-based auto-flag injection to `args` (which includes argv[0]).
/// Returns the (possibly augmented) argv. Looks up env via `lookup`.
pub fn apply<F>(args: Vec<String>, lookup: F) -> Vec<String>
where
    F: Fn(&str) -> Option<String>,
{
    let no_env = args.iter().any(|a| a == "--no-env");
    if no_env {
        return args;
    }

    // Find subcommand path: first non-flag (and not equal to "--no-env") tokens
    // after argv[0]. Up to 2 levels deep. The path also tracks where in argv it
    // ends so we know where to splice env-injected args.
    let mut path: Vec<&str> = Vec::new();
    let mut path_end_idx = 1usize;
    for (i, a) in args.iter().enumerate().skip(1) {
        if a.starts_with('-') {
            continue;
        }
        path.push(a.as_str());
        path_end_idx = i + 1;
        if path.len() == 1 && !KNOWN_GROUPS.contains(&a.as_str()) {
            break;
        }
        if path.len() == 2 {
            break;
        }
    }
    if path.is_empty() {
        return args;
    }

    // Skip injection if the user already supplied any flag after the path.
    let user_has_flag_after_path = args
        .iter()
        .skip(path_end_idx)
        .any(|a| a.starts_with('-') && a != "--no-env");
    if user_has_flag_after_path {
        return args;
    }

    let env_key = format!("ELVANTO_{}", path.join("_").to_ascii_uppercase());
    let env_val = match lookup(&env_key) {
        Some(v) if !v.trim().is_empty() => v,
        _ => return args,
    };

    let injected = match shlex::split(&env_val) {
        Some(v) if !v.is_empty() => v,
        _ => return args,
    };

    // Splice: argv[..path_end_idx] + injected + argv[path_end_idx..]
    let mut out = Vec::with_capacity(args.len() + injected.len());
    out.extend_from_slice(&args[..path_end_idx]);
    out.extend(injected);
    out.extend_from_slice(&args[path_end_idx..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |k| map.get(k).cloned()
    }

    #[test]
    fn no_env_var_set_returns_args_unchanged() {
        let result = apply(argv(&["elvanto", "services", "list"]), env(&[]));
        assert_eq!(result, argv(&["elvanto", "services", "list"]));
    }

    #[test]
    fn empty_env_var_is_ignored() {
        let result = apply(
            argv(&["elvanto", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "  ")]),
        );
        assert_eq!(result, argv(&["elvanto", "services", "list"]));
    }

    #[test]
    fn env_injects_flags_after_subcommand_path() {
        let result = apply(
            argv(&["elvanto", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "--json --full-id")]),
        );
        assert_eq!(
            result,
            argv(&["elvanto", "services", "list", "--json", "--full-id"])
        );
    }

    #[test]
    fn env_injects_for_two_level_path_keeping_positionals() {
        let result = apply(
            argv(&["elvanto", "services", "people", "abc"]),
            env(&[("ELVANTO_SERVICES_PEOPLE", "--hide-unfilled")]),
        );
        assert_eq!(
            result,
            argv(&["elvanto", "services", "people", "--hide-unfilled", "abc"])
        );
    }

    #[test]
    fn manual_flag_after_path_disables_injection() {
        let result = apply(
            argv(&["elvanto", "services", "list", "--json"]),
            env(&[("ELVANTO_SERVICES_LIST", "--full-id")]),
        );
        assert_eq!(result, argv(&["elvanto", "services", "list", "--json"]));
    }

    #[test]
    fn no_env_flag_disables_injection_even_when_var_set() {
        let result = apply(
            argv(&["elvanto", "--no-env", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "--json")]),
        );
        assert_eq!(result, argv(&["elvanto", "--no-env", "services", "list"]));
    }

    #[test]
    fn shell_quoting_in_env_value_is_respected() {
        let result = apply(
            argv(&["elvanto", "songs", "chart", "abc"]),
            env(&[("ELVANTO_SONGS_CHART", "--arrangement \"Acoustic Set\"")]),
        );
        assert_eq!(
            result,
            argv(&[
                "elvanto",
                "songs",
                "chart",
                "--arrangement",
                "Acoustic Set",
                "abc"
            ])
        );
    }

    #[test]
    fn unparseable_env_value_is_ignored() {
        // Unbalanced quote → shlex returns None.
        let result = apply(
            argv(&["elvanto", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "--foo \"bar")]),
        );
        assert_eq!(result, argv(&["elvanto", "services", "list"]));
    }

    #[test]
    fn unknown_top_level_subcommand_still_works() {
        // Single-level path (not a known group). Should look up
        // ELVANTO_<NAME> using just that one segment.
        let result = apply(
            argv(&["elvanto", "future", "--id"]),
            env(&[("ELVANTO_FUTURE", "--debug")]),
        );
        // --id is a flag after path; injection skipped.
        assert_eq!(result, argv(&["elvanto", "future", "--id"]));
    }

    #[test]
    fn empty_argv_after_binary_returns_unchanged() {
        let result = apply(argv(&["elvanto"]), env(&[("ELVANTO_X", "--y")]));
        assert_eq!(result, argv(&["elvanto"]));
    }
}
