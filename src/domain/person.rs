use crate::api::raw::RawPersonRecord;
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_department: Option<String>,
}

impl Person {
    /// True if any department or sub-department name case-insensitively matches one of `filters`.
    /// Empty `filters` matches everyone.
    pub fn matches_department(&self, filters: &[String]) -> bool {
        if filters.is_empty() {
            return true;
        }
        filters.iter().any(|f| {
            self.departments.iter().any(|d| {
                d.department.eq_ignore_ascii_case(f)
                    || d.sub_department
                        .as_deref()
                        .is_some_and(|s| s.eq_ignore_ascii_case(f))
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
                    sub_department: None,
                });
            } else {
                for sd in &d.sub_departments.sub_department {
                    departments.push(PersonDepartmentEntry {
                        department: d.name.clone(),
                        sub_department: Some(sd.name.clone()),
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

/// One row per (department, sub_department?) pair encountered across `people`.
/// Deduplicated by (department_id, sub_department_id_or_empty).
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DepartmentRow {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
}

pub fn collect_departments(people: &[RawPersonRecord]) -> Vec<DepartmentRow> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<DepartmentRow> = Vec::new();
    for p in people {
        for d in &p.departments.department {
            if !d.id.is_empty() && seen.insert(d.id.clone()) {
                out.push(DepartmentRow {
                    id: d.id.clone(),
                    name: d.name.clone(),
                    parent: None,
                });
            }
            for sd in &d.sub_departments.sub_department {
                if !sd.id.is_empty() && seen.insert(sd.id.clone()) {
                    out.push(DepartmentRow {
                        id: sd.id.clone(),
                        name: sd.name.clone(),
                        parent: Some(d.name.clone()),
                    });
                }
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::raw::{
        RawPersonDepartment, RawPersonDepartments, RawPersonSubDepartment, RawPersonSubDepartments,
    };

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
                    name: "Service Teams".into(),
                    sub_departments: RawPersonSubDepartments {
                        sub_department: vec![RawPersonSubDepartment {
                            id: "sd-1".into(),
                            name: "Vocals".into(),
                        }],
                    },
                }],
            },
        }
    }

    #[test]
    fn person_from_raw_normalises_fields() {
        let p: Person = rec_with_depts().into();
        assert_eq!(p.id, "p-1");
        assert_eq!(p.name, "Alice Brown");
        assert_eq!(p.email.as_deref(), Some("alice@example.com"));
        assert_eq!(p.status, "active");
        assert_eq!(p.departments.len(), 1);
        assert_eq!(p.departments[0].department, "Service Teams");
        assert_eq!(p.departments[0].sub_department.as_deref(), Some("Vocals"));
    }

    #[test]
    fn person_matches_department_against_dept_or_sub_dept() {
        let p: Person = rec_with_depts().into();
        assert!(p.matches_department(&["Vocals".to_string()]));
        assert!(p.matches_department(&["service teams".to_string()]));
        assert!(!p.matches_department(&["Sound".to_string()]));
        assert!(p.matches_department(&[]));
    }

    #[test]
    fn person_with_no_subdepartment_emits_single_entry() {
        let mut r = rec_with_depts();
        r.departments.department[0]
            .sub_departments
            .sub_department
            .clear();
        let p: Person = r.into();
        assert_eq!(p.departments.len(), 1);
        assert_eq!(p.departments[0].sub_department, None);
    }

    #[test]
    fn collect_departments_dedupes_and_marks_parent() {
        let p1 = rec_with_depts();
        let mut p2 = rec_with_depts();
        p2.id = "p-2".into();
        let rows = collect_departments(&[p1, p2]);
        // 2 unique entries: top-level + sub-dept (deduped across the two people).
        assert_eq!(rows.len(), 2);
        let vocals = rows.iter().find(|r| r.name == "Vocals").unwrap();
        assert_eq!(vocals.parent.as_deref(), Some("Service Teams"));
        let top = rows.iter().find(|r| r.name == "Service Teams").unwrap();
        assert_eq!(top.parent, None);
    }
}
