use crate::error::CliError;

const KEYS_SHARP: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];
const KEYS_FLAT: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
];

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Request {
    Named(String),
    Offset(i32),
}

pub fn parse(input: &str) -> Result<Request, CliError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(CliError::Usage("--transpose value is empty".into()));
    }
    let first = trimmed.chars().next().unwrap();
    if first == '+' || first == '-' || first.is_ascii_digit() {
        let n: i32 = trimmed
            .parse()
            .map_err(|_| CliError::Usage(format!("invalid transpose offset {trimmed:?}")))?;
        return Ok(Request::Offset(n));
    }
    let normalized = normalize_key(trimmed)
        .ok_or_else(|| CliError::Usage(format!("invalid key {trimmed:?}")))?;
    Ok(Request::Named(normalized))
}

pub fn resolve(req: &Request, starting: &str) -> Result<String, CliError> {
    match req {
        Request::Named(k) => Ok(k.clone()),
        Request::Offset(n) => {
            let base = key_index(starting).ok_or_else(|| {
                CliError::NotFound(format!(
                    "cannot transpose: unknown starting key {starting:?}"
                ))
            })?;
            let prefer_flats = starting.contains('b');
            let idx = ((base as i32 + *n).rem_euclid(12)) as usize;
            let table = if prefer_flats { KEYS_FLAT } else { KEYS_SHARP };
            Ok(table[idx].to_string())
        }
    }
}

fn normalize_key(s: &str) -> Option<String> {
    let upper = capitalize(s);
    if KEYS_SHARP.contains(&upper.as_str()) || KEYS_FLAT.contains(&upper.as_str()) {
        Some(upper)
    } else {
        None
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
    }
}

fn key_index(k: &str) -> Option<usize> {
    KEYS_SHARP
        .iter()
        .position(|x| x.eq_ignore_ascii_case(k))
        .or_else(|| KEYS_FLAT.iter().position(|x| x.eq_ignore_ascii_case(k)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_named() {
        assert_eq!(parse("G").unwrap(), Request::Named("G".into()));
        assert_eq!(parse("f#").unwrap(), Request::Named("F#".into()));
        assert_eq!(parse("Bb").unwrap(), Request::Named("Bb".into()));
    }

    #[test]
    fn parse_offset() {
        assert_eq!(parse("+3").unwrap(), Request::Offset(3));
        assert_eq!(parse("-2").unwrap(), Request::Offset(-2));
        assert_eq!(parse("5").unwrap(), Request::Offset(5));
    }

    #[test]
    fn parse_invalid() {
        assert!(matches!(parse("Q"), Err(CliError::Usage(_))));
        assert!(matches!(parse(""), Err(CliError::Usage(_))));
    }

    #[test]
    fn resolve_offset_uses_sharps() {
        let r = resolve(&Request::Offset(2), "G").unwrap();
        assert_eq!(r, "A");
    }

    #[test]
    fn resolve_offset_wraps() {
        let r = resolve(&Request::Offset(13), "C").unwrap();
        assert_eq!(r, "C#");
        let r2 = resolve(&Request::Offset(-1), "C").unwrap();
        assert_eq!(r2, "B");
    }

    #[test]
    fn resolve_offset_uses_flats_when_starting_has_flat() {
        let r = resolve(&Request::Offset(2), "Bb").unwrap();
        assert_eq!(r, "C");
        let r2 = resolve(&Request::Offset(1), "Bb").unwrap();
        assert_eq!(r2, "B");
    }

    #[test]
    fn resolve_named_passes_through() {
        let r = resolve(&Request::Named("F#".into()), "G").unwrap();
        assert_eq!(r, "F#");
    }
}
