use crate::api::raw::RawCategory;
use crate::resolve::{self, NodeKind, TreeNode};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Category {
    pub id: String,
    pub name: String,
}

impl From<RawCategory> for Category {
    fn from(raw: RawCategory) -> Self {
        Self {
            id: raw.id,
            name: raw.name,
        }
    }
}

pub fn category_tree(cats: &[Category]) -> Vec<TreeNode> {
    let out: Vec<TreeNode> = cats
        .iter()
        .filter(|c| !c.id.is_empty())
        .map(|c| TreeNode {
            id: c.id.clone(),
            path: vec![c.name.clone()],
            kind: NodeKind::Category,
        })
        .collect();
    resolve::dedupe(out)
}

pub fn short_id(id: &str) -> &str {
    id.split_once('-').map_or(id, |(head, _)| head)
}

pub fn id_matches(full_id: &str, requested: &str) -> bool {
    full_id == requested || short_id(full_id) == requested
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_raw_preserves_fields() {
        let raw = RawCategory {
            id: "cat-1".into(),
            name: "Worship".into(),
        };
        let cat: Category = raw.into();
        assert_eq!(
            cat,
            Category {
                id: "cat-1".into(),
                name: "Worship".into()
            }
        );
    }

    #[test]
    fn short_id_uses_first_uuid_block() {
        assert_eq!(short_id("02b06b47-c275-11e6-aad3-0219ad55c99b"), "02b06b47");
        assert_eq!(short_id("legacy-id"), "legacy");
        assert_eq!(short_id("c1"), "c1");
    }

    #[test]
    fn id_matches_full_or_short() {
        let full = "02b06b47-c275-11e6-aad3-0219ad55c99b";
        assert!(id_matches(full, full));
        assert!(id_matches(full, "02b06b47"));
        assert!(!id_matches(full, "c275"));
    }
}
