use crate::api::raw::RawPersonRecord;
use crate::domain::category::id_matches;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Person {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    pub status: String,
    pub departments: Vec<PersonDepartmentEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PersonDepartmentEntry {
    pub department: String,
    pub department_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_department_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_id: Option<String>,
}

impl Person {
    /// True if any filter id (full UUID or short first-block) matches the
    /// department, sub-department, OR position id of at least one entry in
    /// this person's tree. Empty `filters` matches everyone.
    pub fn matches_department(&self, filters: &[String]) -> bool {
        if filters.is_empty() {
            return true;
        }
        filters.iter().any(|f| {
            self.departments.iter().any(|d| {
                id_matches(&d.department_id, f)
                    || d.sub_department_id
                        .as_deref()
                        .is_some_and(|id| id_matches(id, f))
                    || d.position_id.as_deref().is_some_and(|id| id_matches(id, f))
            })
        })
    }
}

fn display_name(raw: &RawPersonRecord) -> String {
    let first = if raw.preferred_name.is_empty() {
        raw.firstname.as_str()
    } else {
        raw.preferred_name.as_str()
    };
    match (first.is_empty(), raw.lastname.is_empty()) {
        (true, true) => String::new(),
        (false, true) => first.to_string(),
        (true, false) => raw.lastname.clone(),
        (false, false) => format!("{} {}", first, raw.lastname),
    }
}

fn none_if_empty(s: String) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

impl From<RawPersonRecord> for Person {
    fn from(raw: RawPersonRecord) -> Self {
        let name = display_name(&raw);
        let status = if raw.status.is_empty() {
            "unknown".to_string()
        } else {
            raw.status.to_ascii_lowercase()
        };
        let mut departments = Vec::new();
        for d in &raw.departments.department {
            if d.sub_departments.sub_department.is_empty() {
                departments.push(PersonDepartmentEntry {
                    department: d.name.clone(),
                    department_id: d.id.clone(),
                    sub_department: None,
                    sub_department_id: None,
                    position: None,
                    position_id: None,
                });
                continue;
            }
            for sd in &d.sub_departments.sub_department {
                if sd.positions.position.is_empty() {
                    departments.push(PersonDepartmentEntry {
                        department: d.name.clone(),
                        department_id: d.id.clone(),
                        sub_department: Some(sd.name.clone()),
                        sub_department_id: Some(sd.id.clone()),
                        position: None,
                        position_id: None,
                    });
                    continue;
                }
                for p in &sd.positions.position {
                    departments.push(PersonDepartmentEntry {
                        department: d.name.clone(),
                        department_id: d.id.clone(),
                        sub_department: Some(sd.name.clone()),
                        sub_department_id: Some(sd.id.clone()),
                        position: Some(p.name.clone()),
                        position_id: Some(p.id.clone()),
                    });
                }
            }
        }
        Self {
            id: raw.id,
            name,
            email: none_if_empty(raw.email),
            status,
            departments,
        }
    }
}

/// One row per node in the department tree: top-level departments,
/// sub-departments (parent = department name), and positions (parent =
/// sub-department name). Deduplicated by id. Emitted in DFS order so that
/// each department's sub-departments and positions appear immediately under it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DepartmentRow {
    pub id: String,
    pub name: String,
    /// `"department"`, `"sub_department"`, or `"position"`.
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

pub fn collect_departments(people: &[RawPersonRecord]) -> Vec<DepartmentRow> {
    use std::collections::BTreeMap;

    // dept_id -> (dept_name, sub_dept_id -> (sub_name, position_id -> position_name))
    type SubMap = BTreeMap<String, (String, BTreeMap<String, String>)>;
    type Tree = BTreeMap<String, (String, SubMap)>;

    // Build by name for stable alphabetical order. Use name as the BTreeMap key.
    let mut tree: Tree = BTreeMap::new();
    for p in people {
        for d in &p.departments.department {
            if d.id.is_empty() {
                continue;
            }
            let dept_entry = tree
                .entry(d.name.clone())
                .or_insert_with(|| (d.id.clone(), BTreeMap::new()));
            for sd in &d.sub_departments.sub_department {
                if sd.id.is_empty() {
                    continue;
                }
                let sub_entry = dept_entry
                    .1
                    .entry(sd.name.clone())
                    .or_insert_with(|| (sd.id.clone(), BTreeMap::new()));
                for pos in &sd.positions.position {
                    if pos.id.is_empty() {
                        continue;
                    }
                    sub_entry
                        .1
                        .entry(pos.name.clone())
                        .or_insert_with(|| pos.id.clone());
                }
            }
        }
    }

    let mut out: Vec<DepartmentRow> = Vec::new();
    for (dept_name, (dept_id, subs)) in &tree {
        out.push(DepartmentRow {
            id: dept_id.clone(),
            name: dept_name.clone(),
            kind: "department".into(),
            parent: None,
        });
        for (sub_name, (sub_id, positions)) in subs {
            out.push(DepartmentRow {
                id: sub_id.clone(),
                name: sub_name.clone(),
                kind: "sub_department".into(),
                parent: Some(dept_name.clone()),
            });
            for (pos_name, pos_id) in positions {
                out.push(DepartmentRow {
                    id: pos_id.clone(),
                    name: pos_name.clone(),
                    kind: "position".into(),
                    parent: Some(sub_name.clone()),
                });
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::raw::{
        RawPersonDepartment, RawPersonDepartments, RawPersonPosition, RawPersonPositions,
        RawPersonSubDepartment, RawPersonSubDepartments,
    };

    fn pos(id: &str, name: &str) -> RawPersonPosition {
        RawPersonPosition {
            id: id.into(),
            name: name.into(),
        }
    }

    fn rec_with_depts() -> RawPersonRecord {
        RawPersonRecord {
            id: "p-1".into(),
            email: "alice@example.com".into(),
            firstname: "Alice".into(),
            preferred_name: "".into(),
            lastname: "Brown".into(),
            status: "Active".into(),
            departments: RawPersonDepartments {
                department: vec![RawPersonDepartment {
                    id: "d-1".into(),
                    name: "Music Team".into(),
                    sub_departments: RawPersonSubDepartments {
                        sub_department: vec![RawPersonSubDepartment {
                            id: "sd-1".into(),
                            name: "Vocals".into(),
                            positions: RawPersonPositions {
                                position: vec![pos("p-wl", "Worship Leader"), pos("p-bv", "BV")],
                            },
                        }],
                    },
                }],
            },
        }
    }

    #[test]
    fn person_from_raw_emits_one_row_per_position() {
        let p: Person = rec_with_depts().into();
        assert_eq!(p.departments.len(), 2);
        assert_eq!(p.departments[0].position.as_deref(), Some("Worship Leader"));
        assert_eq!(p.departments[0].position_id.as_deref(), Some("p-wl"));
        assert_eq!(p.departments[1].position.as_deref(), Some("BV"));
    }

    #[test]
    fn matches_department_by_id_at_every_level() {
        let p: Person = rec_with_depts().into();
        assert!(p.matches_department(&["d-1".into()])); // department id
        assert!(p.matches_department(&["sd-1".into()])); // sub-department id
        assert!(p.matches_department(&["p-wl".into()])); // position id
        assert!(!p.matches_department(&["never".into()]));
        assert!(p.matches_department(&[])); // empty matches everyone
                                            // Name match no longer works:
        assert!(!p.matches_department(&["Vocals".into()]));
    }

    #[test]
    fn collect_departments_emits_dfs_tree_with_positions() {
        let rows = collect_departments(&[rec_with_depts()]);
        let names: Vec<(&str, &str, Option<&str>)> = rows
            .iter()
            .map(|r| (r.name.as_str(), r.kind.as_str(), r.parent.as_deref()))
            .collect();
        assert_eq!(
            names,
            vec![
                ("Music Team", "department", None),
                ("Vocals", "sub_department", Some("Music Team")),
                ("BV", "position", Some("Vocals")),
                ("Worship Leader", "position", Some("Vocals")),
            ]
        );
    }
}
