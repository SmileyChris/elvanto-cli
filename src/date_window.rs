use crate::error::CliError;
use chrono::{Duration, Months, NaiveDate};

/// Returns (date_from, date_to) where date_to = `now` and date_from = `now` minus 6 months.
/// Day-of-month is clamped to the last valid day of the resulting month
/// (so e.g. Aug 31 → Feb 28 in a non-leap year).
pub fn default_window(now: NaiveDate) -> (NaiveDate, NaiveDate) {
    let from =
        subtract_months(now, 6).expect("six month default window stays within chrono date range");
    (from, now)
}

fn subtract_months(date: NaiveDate, months: u32) -> Option<NaiveDate> {
    date.checked_sub_months(Months::new(months))
}

pub fn parse_date(input: &str, flag_name: &str) -> Result<NaiveDate, CliError> {
    NaiveDate::parse_from_str(input, "%Y-%m-%d").map_err(|_| {
        CliError::Usage(format!(
            "invalid {flag_name} value {input:?}; expected YYYY-MM-DD"
        ))
    })
}

pub fn parse_duration_start(
    input: &str,
    today: NaiveDate,
    flag_name: &str,
) -> Result<NaiveDate, CliError> {
    let trimmed = input.trim();
    if trimmed.len() < 2 {
        return Err(invalid_duration(flag_name, input));
    }

    let (amount, unit) = trimmed.split_at(trimmed.len() - 1);
    let amount: u32 = amount
        .parse()
        .map_err(|_| invalid_duration(flag_name, input))?;
    if amount == 0 {
        return Err(invalid_duration(flag_name, input));
    }

    match unit {
        "d" => today
            .checked_sub_signed(Duration::days(amount as i64))
            .ok_or_else(|| invalid_duration(flag_name, input)),
        "w" => {
            let days = amount
                .checked_mul(7)
                .ok_or_else(|| invalid_duration(flag_name, input))?;
            today
                .checked_sub_signed(Duration::days(days as i64))
                .ok_or_else(|| invalid_duration(flag_name, input))
        }
        "m" => subtract_months(today, amount).ok_or_else(|| invalid_duration(flag_name, input)),
        "y" => {
            let months = amount
                .checked_mul(12)
                .ok_or_else(|| invalid_duration(flag_name, input))?;
            subtract_months(today, months).ok_or_else(|| invalid_duration(flag_name, input))
        }
        _ => Err(invalid_duration(flag_name, input)),
    }
}

fn invalid_duration(flag_name: &str, input: &str) -> CliError {
    CliError::Usage(format!(
        "invalid {flag_name} value {input:?}; expected a duration like 14d, 2w, 6m, or 1y"
    ))
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

    #[test]
    fn parse_duration_start_supports_days_weeks_months_and_years() {
        let today = d(2026, 5, 19);
        assert_eq!(
            parse_duration_start("14d", today, "--used-within").unwrap(),
            d(2026, 5, 5)
        );
        assert_eq!(
            parse_duration_start("2w", today, "--used-within").unwrap(),
            d(2026, 5, 5)
        );
        assert_eq!(
            parse_duration_start("6m", today, "--used-within").unwrap(),
            d(2025, 11, 19)
        );
        assert_eq!(
            parse_duration_start("1y", today, "--used-within").unwrap(),
            d(2025, 5, 19)
        );
    }

    #[test]
    fn parse_duration_start_rejects_bad_values() {
        assert!(matches!(
            parse_duration_start("soon", d(2026, 5, 19), "--used-within"),
            Err(CliError::Usage(_))
        ));
        assert!(matches!(
            parse_duration_start("0w", d(2026, 5, 19), "--used-within"),
            Err(CliError::Usage(_))
        ));
        assert!(matches!(
            parse_duration_start("2147483648m", d(2026, 5, 19), "--used-within"),
            Err(CliError::Usage(_))
        ));
    }
}
