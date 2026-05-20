//! Lookup resolver: turn an id, name, or path into a single id.
//!
//! Used by every flag that accepts an id reference to an Elvanto entity in
//! the org tree (`people list --in`, `services people --in`,
//! `songs list --category`). A single query value may be:
//!
//! * **An id**: full UUID (`02b06b47-c275-11e6-…`) or short first-block
//!   (`02b06b47`). Detected when the value is all hex digits and dashes with
//!   at least 8 hex chars. Misses fail hard — no name-search fallback.
//! * **A name**: case-insensitive whole-segment match against the last
//!   component of a tree node's path. `Vocals` matches a sub-dept whose path
//!   ends in `Vocals`.
//! * **A path**: `/`-separated, matches when the query segments are a suffix
//!   of the node's path. `Music Team/Vocals` matches a sub-dept whose full
//!   path is `Music Team / Vocals`.
//!
//! Name/path matching runs in tiers: exact whole-segment match first; if that
//! finds nothing, a **unique-prefix fallback** treats the last query segment
//! as a prefix of the corresponding node segment (so `Contemporary` matches
//! `Contemporary (0-5 Years Old)` when it's the only category starting with
//! that word). Ambiguous prefix matches still fail with a disambiguation table.
//!
//! Subtree behaviour is **implicit and free**: the resolver returns one id,
//! and the existing `matches_department` impls OR-match against every level
//! of a person's department tree. So resolving `Music Team` to that dept's
//! id automatically includes everyone in any sub-dept or position under it.
//!
//! On ambiguity or miss the resolver returns a `CliError::Usage` with a
//! human-readable table; for misses it includes the top 3 Jaro-Winkler
//! suggestions over 0.7.

use crate::error::CliError;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Department,
    SubDepartment,
    Position,
    Category,
}

impl NodeKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Department => "department",
            Self::SubDepartment => "sub_department",
            Self::Position => "position",
            Self::Category => "category",
        }
    }
}

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub id: String,
    pub path: Vec<String>,
    pub kind: NodeKind,
}

impl TreeNode {
    pub fn display_path(&self) -> String {
        self.path.join(" / ")
    }
    fn short_id(&self) -> &str {
        self.id.split_once('-').map_or(self.id.as_str(), |(h, _)| h)
    }
}

/// Looks like a UUID or short id: all chars hex or dash, at least 8 hex chars.
fn looks_like_id(s: &str) -> bool {
    let hex = s.chars().filter(|c| c.is_ascii_hexdigit()).count();
    hex >= 8 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

fn id_matches(full: &str, requested: &str) -> bool {
    let short = full.split_once('-').map_or(full, |(h, _)| h);
    full.eq_ignore_ascii_case(requested) || short.eq_ignore_ascii_case(requested)
}

fn split_query(query: &str) -> Vec<&str> {
    query
        .split('/')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
}

fn path_suffix_matches(node_path: &[String], query: &[&str]) -> bool {
    if query.is_empty() || query.len() > node_path.len() {
        return false;
    }
    let start = node_path.len() - query.len();
    node_path[start..]
        .iter()
        .zip(query.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
}

/// Like `path_suffix_matches`, but the **last** query segment matches as a
/// prefix of the corresponding node segment. Earlier segments stay exact.
/// Used as a unique-prefix fallback when no exact suffix match exists.
fn path_suffix_matches_with_last_prefix(node_path: &[String], query: &[&str]) -> bool {
    if query.is_empty() || query.len() > node_path.len() {
        return false;
    }
    let start = node_path.len() - query.len();
    let (last_q, rest_q) = query.split_last().expect("non-empty");
    let suffix = &node_path[start..];
    let (last_n, rest_n) = suffix.split_last().expect("non-empty");

    if !rest_n
        .iter()
        .zip(rest_q.iter())
        .all(|(a, b)| a.eq_ignore_ascii_case(b))
    {
        return false;
    }
    last_n
        .to_ascii_lowercase()
        .starts_with(&last_q.to_ascii_lowercase())
}

/// Resolve a single query value against `tree`. Returns the resolved id on
/// success, or `CliError::Usage` with a formatted explanation on failure.
pub fn resolve(query: &str, tree: &[TreeNode]) -> Result<String, CliError> {
    let q = query.trim();
    if q.is_empty() {
        return Err(CliError::Usage("empty lookup value".into()));
    }

    if looks_like_id(q) {
        for n in tree {
            if id_matches(&n.id, q) {
                return Ok(n.id.clone());
            }
        }
        return Err(CliError::Usage(format!(
            "no node with id {q:?} (treated as id because it looks like a UUID)"
        )));
    }

    let segments = split_query(q);

    // Tier 1: exact whole-segment suffix match.
    let exact: Vec<&TreeNode> = tree
        .iter()
        .filter(|n| path_suffix_matches(&n.path, &segments))
        .collect();
    if exact.len() == 1 {
        return Ok(exact[0].id.clone());
    }
    if exact.len() > 1 {
        return Err(CliError::Usage(render_ambiguous(q, &exact)));
    }

    // Tier 2: unique-prefix on the last query segment (earlier segments stay
    // exact). Only consulted when Tier 1 finds zero matches.
    let prefix: Vec<&TreeNode> = tree
        .iter()
        .filter(|n| path_suffix_matches_with_last_prefix(&n.path, &segments))
        .collect();
    if prefix.len() == 1 {
        return Ok(prefix[0].id.clone());
    }
    if prefix.len() > 1 {
        return Err(CliError::Usage(render_ambiguous(q, &prefix)));
    }

    // Tier 3: nothing matched — fuzzy "did you mean" suggestions.
    Err(CliError::Usage(render_no_match(q, tree)))
}

/// Resolve every value in `queries`, short-circuiting on the first failure.
pub fn resolve_all(queries: &[String], tree: &[TreeNode]) -> Result<Vec<String>, CliError> {
    queries.iter().map(|q| resolve(q, tree)).collect()
}

fn render_ambiguous(query: &str, matches: &[&TreeNode]) -> String {
    let mut s = format!(
        "{query:?} matches {n} nodes — disambiguate with id or full path:\n",
        n = matches.len()
    );
    for n in matches {
        s.push_str(&format!(
            "  {short:8}  {kind:14}  {path}\n",
            short = n.short_id(),
            kind = n.kind.label(),
            path = n.display_path(),
        ));
    }
    s.trim_end().to_string()
}

fn render_no_match(query: &str, tree: &[TreeNode]) -> String {
    let q_lower = query.to_ascii_lowercase();
    let mut scored: Vec<(f64, &TreeNode)> = tree
        .iter()
        .map(|n| {
            let path_lc = n.display_path().to_ascii_lowercase();
            // Score against both the joined path and the last segment, take the better.
            let last_lc = n
                .path
                .last()
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_default();
            let s1 = strsim::jaro_winkler(&q_lower, &path_lc);
            let s2 = strsim::jaro_winkler(&q_lower, &last_lc);
            (s1.max(s2), n)
        })
        .filter(|(s, _)| *s >= 0.7)
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Dedupe by id so the same node doesn't appear twice via different scoring paths.
    let mut seen = HashSet::new();
    scored.retain(|(_, n)| seen.insert(n.id.clone()));
    scored.truncate(3);

    if scored.is_empty() {
        return format!("no match for {query:?}");
    }
    let mut s = format!("no match for {query:?}. Did you mean:\n");
    for (_, n) in scored {
        s.push_str(&format!(
            "  {short:8}  {kind:14}  {path}\n",
            short = n.short_id(),
            kind = n.kind.label(),
            path = n.display_path(),
        ));
    }
    s.trim_end().to_string()
}

/// Drop duplicate nodes (by id), keeping the first occurrence.
pub fn dedupe(nodes: Vec<TreeNode>) -> Vec<TreeNode> {
    let mut seen = HashSet::new();
    let mut out = Vec::with_capacity(nodes.len());
    for n in nodes {
        if seen.insert(n.id.clone()) {
            out.push(n);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(id: &str, kind: NodeKind, path: &[&str]) -> TreeNode {
        TreeNode {
            id: id.into(),
            path: path.iter().map(|s| (*s).into()).collect(),
            kind,
        }
    }

    fn sample_tree() -> Vec<TreeNode> {
        vec![
            n(
                "d1aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Department,
                &["Music Team"],
            ),
            n(
                "5d1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::SubDepartment,
                &["Music Team", "Vocals"],
            ),
            n(
                "01aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Position,
                &["Music Team", "Vocals", "Worship Leader"],
            ),
            n(
                "02aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Position,
                &["Music Team", "Vocals", "BV"],
            ),
            n(
                "5d2aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::SubDepartment,
                &["Music Team", "Instruments"],
            ),
            n(
                "03aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Position,
                &["Music Team", "Instruments", "Leader"],
            ),
            n(
                "d2aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Department,
                &["Welcome Team"],
            ),
            n(
                "04aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Position,
                &["Welcome Team", "Leader"],
            ),
        ]
    }

    #[test]
    fn looks_like_id_detects_hex_forms() {
        assert!(looks_like_id("02b06b47"));
        assert!(looks_like_id("02b06b47-c275-11e6-aad3-0219ad55c99b"));
        assert!(looks_like_id("ABCDEF01-1234"));
        assert!(!looks_like_id("Vocals"));
        assert!(!looks_like_id("Music Team")); // space
        assert!(!looks_like_id("abc")); // <8 hex
        assert!(!looks_like_id("ghijkl12")); // non-hex
    }

    #[test]
    fn id_query_returns_full_id_for_short_or_full() {
        let tree = sample_tree();
        assert_eq!(
            resolve("5d1aaaaa", &tree).unwrap(),
            "5d1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
        assert_eq!(
            resolve("5d1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee", &tree).unwrap(),
            "5d1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn id_miss_is_hard_error_no_name_fallback() {
        let tree = sample_tree();
        let err = resolve("deadbe01", &tree).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("treated as id"));
        assert!(msg.contains("deadbe01"));
    }

    const VOCALS_ID: &str = "5d1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    const WL_ID: &str = "01aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";
    const INSTR_LEADER_ID: &str = "03aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee";

    #[test]
    fn name_query_resolves_unique_segment() {
        let tree = sample_tree();
        assert_eq!(resolve("Vocals", &tree).unwrap(), VOCALS_ID);
        assert_eq!(resolve("Worship Leader", &tree).unwrap(), WL_ID);
    }

    #[test]
    fn name_query_is_case_insensitive() {
        let tree = sample_tree();
        assert_eq!(resolve("vocals", &tree).unwrap(), VOCALS_ID);
        assert_eq!(resolve("VOCALS", &tree).unwrap(), VOCALS_ID);
    }

    #[test]
    fn path_query_disambiguates_collisions() {
        let tree = sample_tree();
        // "Leader" alone is ambiguous (two positions).
        let err = resolve("Leader", &tree).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("matches 2 nodes"));
        assert!(msg.contains("Music Team / Instruments / Leader"));
        assert!(msg.contains("Welcome Team / Leader"));

        // Path picks one.
        assert_eq!(
            resolve("Music Team/Instruments/Leader", &tree).unwrap(),
            INSTR_LEADER_ID
        );
        // Partial path still disambiguates.
        assert_eq!(
            resolve("Instruments/Leader", &tree).unwrap(),
            INSTR_LEADER_ID
        );
    }

    #[test]
    fn unique_prefix_resolves_when_no_exact_match() {
        let tree = vec![
            n(
                "ca1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Category,
                &["Contemporary (0-5 Years Old)"],
            ),
            n(
                "ca2aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Category,
                &["Hymns"],
            ),
        ];
        assert_eq!(
            resolve("Contemporary", &tree).unwrap(),
            "ca1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn ambiguous_prefix_match_errors_with_table() {
        let tree = vec![
            n(
                "ca1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Category,
                &["Worship Songs"],
            ),
            n(
                "ca2aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Category,
                &["Worship Old"],
            ),
        ];
        let err = resolve("Worship", &tree).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("matches 2 nodes"));
        assert!(msg.contains("Worship Songs"));
        assert!(msg.contains("Worship Old"));
    }

    #[test]
    fn exact_match_wins_over_prefix_when_both_could_apply() {
        // "Music Team" should resolve to the dept exactly, not be treated as a
        // prefix of e.g. a hypothetical "Music Team Setup".
        let tree = vec![
            n(
                "d1aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Department,
                &["Music Team"],
            ),
            n(
                "d2aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
                NodeKind::Department,
                &["Music Team Setup"],
            ),
        ];
        assert_eq!(
            resolve("Music Team", &tree).unwrap(),
            "d1aaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn path_with_prefix_on_last_segment() {
        let tree = sample_tree();
        // "Music Team/Voc" → "Music Team / Vocals" by prefix on last segment.
        assert_eq!(
            resolve("Music Team/Voc", &tree).unwrap(),
            "5d1aaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn no_match_suggests_via_jaro_winkler() {
        let tree = sample_tree();
        let err = resolve("voclas", &tree).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("Did you mean"));
        assert!(msg.contains("Vocals"));
    }

    #[test]
    fn no_match_without_close_neighbours_is_terse() {
        let tree = sample_tree();
        let err = resolve("xyzzy", &tree).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("no match"));
        assert!(!msg.contains("Did you mean"));
    }

    #[test]
    fn empty_query_errors() {
        assert!(resolve("", &sample_tree()).is_err());
        assert!(resolve("   ", &sample_tree()).is_err());
    }

    #[test]
    fn resolve_all_short_circuits_on_first_failure() {
        let tree = sample_tree();
        let queries = vec!["Vocals".to_string(), "Leader".to_string()];
        let err = resolve_all(&queries, &tree).unwrap_err();
        assert!(format!("{err}").contains("matches 2 nodes"));
    }

    #[test]
    fn dedupe_keeps_first_occurrence() {
        let nodes = vec![
            n("x-1", NodeKind::Department, &["A"]),
            n("x-1", NodeKind::Department, &["A"]),
            n("y-2", NodeKind::Department, &["B"]),
        ];
        let out = dedupe(nodes);
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].id, "x-1");
        assert_eq!(out[1].id, "y-2");
    }
}
