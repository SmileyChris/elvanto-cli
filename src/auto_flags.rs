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
//! `apply` returns both the (possibly augmented) argv and an optional
//! `InjectionNote` describing what was injected or what was suppressed by a
//! manual override. The CLI emits the note to stderr at the end of the command
//! so the user can spot when their `.env` defaults are (or aren't) in effect.
//!
//! Recognised subcommand groups: `people`, `services`, `songs`. Other tokens
//! are treated as a single-level path.

const KNOWN_GROUPS: &[&str] = &["people", "services", "songs"];

/// Result of an `apply` call. `argv` is the (possibly augmented) command line
/// to hand to clap; `note` is `Some(_)` when there's something the user should
/// know about env-based injection.
pub struct Applied {
    pub argv: Vec<String>,
    pub note: Option<InjectionNote>,
}

/// What happened to env-based injection on this invocation.
pub enum InjectionNote {
    /// The env var was set and the flags were spliced into argv.
    Injected { env_var: String, flags: Vec<String> },
    /// The env var was set but a manual flag suppressed injection.
    Suppressed { env_var: String, flags: Vec<String> },
}

impl InjectionNote {
    /// Render the note as a one- or two-line stderr message.
    pub fn render(&self) -> String {
        match self {
            Self::Injected { env_var, flags } => format!(
                "note: applied {env_var} defaults (use --no-env to disable):\n      {}",
                flags.join(" ")
            ),
            Self::Suppressed { env_var, flags } => format!(
                "note: {env_var} defaults suppressed by manual flag (use --no-env to silence this note):\n      would have applied: {}",
                flags.join(" ")
            ),
        }
    }
}

/// Apply env-based auto-flag injection to `args` (which includes argv[0]).
pub fn apply<F>(args: Vec<String>, lookup: F) -> Applied
where
    F: Fn(&str) -> Option<String>,
{
    let no_env = args.iter().any(|a| a == "--no-env");
    if no_env {
        return Applied {
            argv: args,
            note: None,
        };
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
        return Applied {
            argv: args,
            note: None,
        };
    }

    let env_key = format!("ELVANTO_{}", path.join("_").to_ascii_uppercase());
    let env_val = match lookup(&env_key) {
        Some(v) if !v.trim().is_empty() => v,
        _ => {
            return Applied {
                argv: args,
                note: None,
            }
        }
    };

    let injected = match shlex::split(&env_val) {
        Some(v) if !v.is_empty() => v,
        _ => {
            return Applied {
                argv: args,
                note: None,
            }
        }
    };

    // If the user already supplied any flag after the path, suppress injection
    // but tell them what would have applied.
    let user_has_flag_after_path = args
        .iter()
        .skip(path_end_idx)
        .any(|a| a.starts_with('-') && a != "--no-env");
    if user_has_flag_after_path {
        return Applied {
            argv: args,
            note: Some(InjectionNote::Suppressed {
                env_var: env_key,
                flags: injected,
            }),
        };
    }

    // Splice: argv[..path_end_idx] + injected + argv[path_end_idx..]
    let mut out = Vec::with_capacity(args.len() + injected.len());
    out.extend_from_slice(&args[..path_end_idx]);
    out.extend(injected.iter().cloned());
    out.extend_from_slice(&args[path_end_idx..]);
    Applied {
        argv: out,
        note: Some(InjectionNote::Injected {
            env_var: env_key,
            flags: injected,
        }),
    }
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

    fn note_kind(n: &Option<InjectionNote>) -> &'static str {
        match n {
            Some(InjectionNote::Injected { .. }) => "injected",
            Some(InjectionNote::Suppressed { .. }) => "suppressed",
            None => "none",
        }
    }

    #[test]
    fn no_env_var_set_returns_args_unchanged() {
        let r = apply(argv(&["elvanto", "services", "list"]), env(&[]));
        assert_eq!(r.argv, argv(&["elvanto", "services", "list"]));
        assert_eq!(note_kind(&r.note), "none");
    }

    #[test]
    fn empty_env_var_is_ignored() {
        let r = apply(
            argv(&["elvanto", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "  ")]),
        );
        assert_eq!(r.argv, argv(&["elvanto", "services", "list"]));
        assert_eq!(note_kind(&r.note), "none");
    }

    #[test]
    fn env_injects_flags_after_subcommand_path() {
        let r = apply(
            argv(&["elvanto", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "--json --id long")]),
        );
        assert_eq!(
            r.argv,
            argv(&["elvanto", "services", "list", "--json", "--id", "long"])
        );
        assert_eq!(note_kind(&r.note), "injected");
    }

    #[test]
    fn env_injects_for_two_level_path_keeping_positionals() {
        let r = apply(
            argv(&["elvanto", "services", "people", "abc"]),
            env(&[("ELVANTO_SERVICES_PEOPLE", "--hide-unfilled")]),
        );
        assert_eq!(
            r.argv,
            argv(&["elvanto", "services", "people", "--hide-unfilled", "abc"])
        );
        assert_eq!(note_kind(&r.note), "injected");
    }

    #[test]
    fn manual_flag_after_path_emits_suppressed_note() {
        let r = apply(
            argv(&["elvanto", "services", "list", "--json"]),
            env(&[("ELVANTO_SERVICES_LIST", "--id long")]),
        );
        assert_eq!(r.argv, argv(&["elvanto", "services", "list", "--json"]));
        assert_eq!(note_kind(&r.note), "suppressed");
        let rendered = r.note.unwrap().render();
        assert!(rendered.contains("ELVANTO_SERVICES_LIST"));
        assert!(rendered.contains("--id long"));
    }

    #[test]
    fn no_env_flag_disables_injection_silently() {
        let r = apply(
            argv(&["elvanto", "--no-env", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "--json")]),
        );
        assert_eq!(r.argv, argv(&["elvanto", "--no-env", "services", "list"]));
        assert_eq!(note_kind(&r.note), "none");
    }

    #[test]
    fn shell_quoting_in_env_value_is_respected() {
        let r = apply(
            argv(&["elvanto", "songs", "chart", "abc"]),
            env(&[("ELVANTO_SONGS_CHART", "--arrangement \"Acoustic Set\"")]),
        );
        assert_eq!(
            r.argv,
            argv(&[
                "elvanto",
                "songs",
                "chart",
                "--arrangement",
                "Acoustic Set",
                "abc"
            ])
        );
        assert_eq!(note_kind(&r.note), "injected");
    }

    #[test]
    fn unparseable_env_value_is_ignored() {
        let r = apply(
            argv(&["elvanto", "services", "list"]),
            env(&[("ELVANTO_SERVICES_LIST", "--foo \"bar")]),
        );
        assert_eq!(r.argv, argv(&["elvanto", "services", "list"]));
        assert_eq!(note_kind(&r.note), "none");
    }

    #[test]
    fn unknown_top_level_subcommand_still_works() {
        let r = apply(
            argv(&["elvanto", "future", "--id"]),
            env(&[("ELVANTO_FUTURE", "--debug")]),
        );
        assert_eq!(r.argv, argv(&["elvanto", "future", "--id"]));
        // --id is a flag after the path → suppression note.
        assert_eq!(note_kind(&r.note), "suppressed");
    }

    #[test]
    fn empty_argv_after_binary_returns_unchanged() {
        let r = apply(argv(&["elvanto"]), env(&[("ELVANTO_X", "--y")]));
        assert_eq!(r.argv, argv(&["elvanto"]));
        assert_eq!(note_kind(&r.note), "none");
    }
}
