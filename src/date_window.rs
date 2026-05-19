use crate::error::CliError;
use chrono::{Datelike, NaiveDate};

/// Returns (date_from, date_to) where date_to = `now` and date_from = `now` minus 6 months.
/// Day-of-month is clamped to the last valid day of the resulting month
/// (so e.g. Aug 31 → Feb 28 in a non-leap year).
#[allow(dead_code)]
pub fn default_window(now: NaiveDate) -> (NaiveDate, NaiveDate) {
    let from = subtract_months(now, 6);
    (from, now)
}

fn subtract_months(date: NaiveDate, months: u32) -> NaiveDate {
    let mut year = date.year();
    let mut month = date.month() as i32 - months as i32;
    while month < 1 {
        month += 12;
        year -= 1;
    }
    let month = month as u32;
    let day = date.day().min(last_day_of_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).expect("valid clamped date")
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    for d in (28..=31).rev() {
        if NaiveDate::from_ymd_opt(year, month, d).is_some() {
            return d;
        }
    }
    28
}

#[allow(dead_code)]
pub fn parse_date(input: &str, flag_name: &str) -> Result<NaiveDate, CliError> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").map_err(|_| {
        CliError::Usage(format!(
            "invalid {flag_name} value {input:?}; expected YYYY-MM-DD"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn six_months_back_middle_of_month() {
        let now = d(2026, 5, 19);
        let (from, to) = default_window(now);
        assert_eq!(from, d(2025, 11, 19));
        assert_eq!(to, d(2026, 5, 19));
    }

    #[test]
    fn six_months_back_wraps_year() {
        let now = d(2026, 2, 10);
        let (from, _to) = default_window(now);
        assert_eq!(from, d(2025, 8, 10));
    }

    #[test]
    fn six_months_back_clamps_to_last_day() {
        // Aug 31 - 6 months = Feb 28 (2026 is not a leap year)
        let now = d(2026, 8, 31);
        let (from, _to) = default_window(now);
        assert_eq!(from, d(2026, 2, 28));
    }

    #[test]
    fn six_months_back_handles_leap_february() {
        // Aug 31 2024 - 6 months = Feb 29 (2024 is a leap year)
        let now = d(2024, 8, 31);
        let (from, _to) = default_window(now);
        assert_eq!(from, d(2024, 2, 29));
    }

    #[test]
    fn parse_date_ok() {
        assert_eq!(parse_date("2026-01-15", "--from").unwrap(), d(2026, 1, 15));
    }

    #[test]
    fn parse_date_rejects_bad_format() {
        let err = parse_date("01/15/2026", "--from").unwrap_err();
        assert!(matches!(err, CliError::Usage(_)));
        assert!(err.to_string().contains("--from"));
        assert!(err.to_string().contains("YYYY-MM-DD"));
    }

    #[test]
    fn parse_date_rejects_invalid_calendar_date() {
        let err = parse_date("2026-02-30", "--from").unwrap_err();
        assert!(matches!(err, CliError::Usage(_)));
    }
}
