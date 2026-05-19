use crate::domain::arrangement::Arrangement;
use crate::error::CliError;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Selection<'a> {
    pub chosen: &'a Arrangement,
    pub others: Vec<&'a Arrangement>,
}

#[allow(dead_code)]
pub fn select<'a>(
    arrangements: &'a [Arrangement],
    requested: Option<&str>,
) -> Result<Selection<'a>, CliError> {
    if arrangements.is_empty() {
        return Err(CliError::NotFound("song has no arrangements".into()));
    }

    let chosen_idx = match requested {
        Some(name) => arrangements
            .iter()
            .position(|a| a.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| {
                let available: Vec<&str> = arrangements.iter().map(|a| a.name.as_str()).collect();
                CliError::Usage(format!(
                    "arrangement {:?} not found; available: {}",
                    name,
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

    #[test]
    fn requested_match_case_insensitive() {
        let list = vec![arr("Acoustic"), arr("Default")];
        let sel = select(&list, Some("acoustic")).unwrap();
        assert_eq!(sel.chosen.name, "Acoustic");
    }

    #[test]
    fn missing_request_errors_with_usage() {
        let list = vec![arr("Default")];
        match select(&list, Some("Live")) {
            Err(CliError::Usage(msg)) => {
                assert!(msg.contains("not found"));
                assert!(msg.contains("Default"));
            }
            other => panic!("expected Usage error, got {other:?}"),
        }
    }
}
