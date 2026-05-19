use crate::domain::arrangement::Arrangement;
use crate::domain::category::{id_matches, short_id};
use crate::error::CliError;

#[derive(Debug)]
pub struct Selection<'a> {
    pub chosen: &'a Arrangement,
    pub others: Vec<&'a Arrangement>,
}

pub fn select<'a>(
    arrangements: &'a [Arrangement],
    requested: Option<&str>,
) -> Result<Selection<'a>, CliError> {
    if arrangements.is_empty() {
        return Err(CliError::NotFound("song has no arrangements".into()));
    }

    let chosen_idx = match requested {
        Some(id) => arrangements
            .iter()
            .position(|a| id_matches(&a.id, id))
            .ok_or_else(|| {
                let available: Vec<String> = arrangements
                    .iter()
                    .map(|a| format!("{} ({})", short_id(&a.id), a.name))
                    .collect();
                CliError::Usage(format!(
                    "arrangement id {:?} not found; available: {}",
                    id,
                    available.join(", ")
                ))
            })?,
        None => arrangements
            .iter()
            .position(|a| a.name.eq_ignore_ascii_case("Default"))
            .unwrap_or(0),
    };

    let chosen = &arrangements[chosen_idx];
    let others = arrangements
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != chosen_idx)
        .map(|(_, a)| a)
        .collect();
    Ok(Selection { chosen, others })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::arrangement::Arrangement;

    fn arr(name: &str) -> Arrangement {
        Arrangement {
            id: name.into(),
            name: name.into(),
            sequence: None,
            bpm: None,
            duration: None,
            keys: vec![],
            lyrics: None,
            chord_chart: None,
        }
    }

    #[test]
    fn empty_list_errors() {
        assert!(matches!(select(&[], None), Err(CliError::NotFound(_))));
    }

    #[test]
    fn default_is_picked_when_no_request() {
        let list = vec![arr("Acoustic"), arr("Default"), arr("Live")];
        let sel = select(&list, None).unwrap();
        assert_eq!(sel.chosen.name, "Default");
        assert_eq!(sel.others.len(), 2);
    }

    #[test]
    fn falls_back_to_first_without_default() {
        let list = vec![arr("Acoustic"), arr("Live")];
        let sel = select(&list, None).unwrap();
        assert_eq!(sel.chosen.name, "Acoustic");
    }

    fn arr_with_id(id: &str, name: &str) -> Arrangement {
        Arrangement {
            id: id.into(),
            name: name.into(),
            sequence: None,
            bpm: None,
            duration: None,
            keys: vec![],
            lyrics: None,
            chord_chart: None,
        }
    }

    #[test]
    fn requested_match_by_short_id() {
        let list = vec![
            arr_with_id("a1b2c3d4-aaaa-bbbb-cccc-acoustic000", "Acoustic"),
            arr_with_id("d5e6f7g8-aaaa-bbbb-cccc-default0000", "Default"),
        ];
        let sel = select(&list, Some("a1b2c3d4")).unwrap();
        assert_eq!(sel.chosen.name, "Acoustic");
    }

    #[test]
    fn requested_match_by_full_id() {
        let full = "a1b2c3d4-aaaa-bbbb-cccc-acoustic000";
        let list = vec![arr_with_id(full, "Acoustic"), arr_with_id("d", "Default")];
        let sel = select(&list, Some(full)).unwrap();
        assert_eq!(sel.chosen.name, "Acoustic");
    }

    #[test]
    fn missing_request_errors_with_usage_including_short_ids() {
        let list = vec![arr_with_id("abcd1234-x-y-z-default", "Default")];
        match select(&list, Some("zzzzzzzz")) {
            Err(CliError::Usage(msg)) => {
                assert!(msg.contains("not found"));
                assert!(msg.contains("abcd1234"));
                assert!(msg.contains("Default"));
            }
            other => panic!("expected Usage error, got {other:?}"),
        }
    }
}
