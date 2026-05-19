use crate::api::raw::RawCategory;
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
}
