use crate::error::BioMcpError;

const DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn normalize_since(value: &str) -> Result<String, BioMcpError> {
    let v = value.trim();
    if v.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "--since accepts YYYY, YYYY-MM, or YYYY-MM-DD format".into(),
        ));
    }

    if v.len() == 4 && v.chars().all(|c| c.is_ascii_digit()) {
        return Ok(format!("{v}-01-01"));
    }

    if v.len() == 7 {
        let bytes = v.as_bytes();
        if bytes[4] == b'-'
            && v.chars()
                .enumerate()
                .all(|(i, c)| i == 4 || c.is_ascii_digit())
        {
            return Ok(format!("{v}-01"));
        }
    }

    if v.len() == 10 {
        return Ok(v.to_string());
    }

    Err(BioMcpError::InvalidArgument(
        "--since accepts YYYY, YYYY-MM, or YYYY-MM-DD format".into(),
    ))
}

pub(crate) fn validate_since(value: &str) -> Result<String, BioMcpError> {
    let normalized = normalize_since(value)?;
    let v = normalized.as_str();

    let bytes = v.as_bytes();
    if bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(BioMcpError::InvalidArgument(
            "--since must be in YYYY-MM-DD format".into(),
        ));
    }
    if !v
        .chars()
        .enumerate()
        .all(|(i, c)| (i == 4 || i == 7) || c.is_ascii_digit())
    {
        return Err(BioMcpError::InvalidArgument(
            "--since must be in YYYY-MM-DD format".into(),
        ));
    }

    let year: u32 = v[0..4]
        .parse()
        .map_err(|_| BioMcpError::InvalidArgument("Invalid year in --since".into()))?;
    let month: u32 = v[5..7]
        .parse()
        .map_err(|_| BioMcpError::InvalidArgument("Invalid month in --since".into()))?;
    let day: u32 = v[8..10]
        .parse()
        .map_err(|_| BioMcpError::InvalidArgument("Invalid day in --since".into()))?;

    if !(1..=12).contains(&month) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid month {month} in --since (must be 01-12)"
        )));
    }

    let max_day = if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[(month - 1) as usize]
    };
    if day < 1 || day > max_day as u32 {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid day {day} for month {month} in --since"
        )));
    }

    Ok(normalized)
}

/// Strict day shape for provider-supplied ClinVar evaluation dates: exactly
/// `YYYY-MM-DD`, naming a real Gregorian day. Provider spellings such as
/// "01 Apr 2019" or reordered forms are omitted rather than trusted. The
/// variant headline date (ticket 2022) uses this rule; ticket 1291 still
/// carries its own inline check on its branch, and whichever of the two
/// lands second collapses the duplicate onto this helper.
pub(crate) fn is_day_shaped(value: &str) -> bool {
    let bytes = value.as_bytes();
    let digits_at =
        |indices: [usize; 8]| indices.iter().all(|index| bytes[*index].is_ascii_digit());
    if value.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !digits_at([0, 1, 2, 3, 5, 6, 8, 9])
    {
        return false;
    }
    let (year, month, day) = match (
        value[0..4].parse::<u32>(),
        value[5..7].parse::<u8>(),
        value[8..10].parse::<u8>(),
    ) {
        // Digit-only fixed-width slices always parse; the arms below are the
        // real gate: a real month and a day that month can hold.
        (Ok(year), Ok(month), Ok(day)) => (year, month, day),
        _ => return false,
    };
    if !(1..=12).contains(&month) {
        return false;
    }
    let max_day = if month == 2 && is_leap_year(year) {
        29
    } else {
        DAYS_IN_MONTH[usize::from(month) - 1]
    };
    (1..=max_day).contains(&day)
}

#[cfg(test)]
mod tests {
    use super::{is_day_shaped, validate_since};

    #[test]
    fn expands_year_only() {
        assert_eq!(
            validate_since("2015").expect("valid year"),
            "2015-01-01".to_string()
        );
    }

    #[test]
    fn expands_year_month() {
        assert_eq!(
            validate_since("2015-06").expect("valid year-month"),
            "2015-06-01".to_string()
        );
    }

    #[test]
    fn keeps_full_date() {
        assert_eq!(
            validate_since("2015-06-15").expect("valid full date"),
            "2015-06-15".to_string()
        );
    }

    #[test]
    fn trims_outer_whitespace() {
        assert_eq!(
            validate_since(" 2015-06 ").expect("trimmed year-month"),
            "2015-06-01".to_string()
        );
    }

    #[test]
    fn accepts_leap_day_only_in_leap_years() {
        assert_eq!(
            validate_since("2024-02-29").expect("valid leap day"),
            "2024-02-29".to_string()
        );

        let err = validate_since("2023-02-29").expect_err("non-leap day should fail");
        assert!(err.to_string().contains("Invalid day 29 for month 2"));
    }

    #[test]
    fn rejects_invalid_day_for_month() {
        let err = validate_since("2024-04-31").expect_err("April 31 should fail");
        assert!(err.to_string().contains("Invalid day 31 for month 4"));
    }

    #[test]
    fn rejects_invalid_month() {
        let err = validate_since("2015-13").expect_err("month should fail");
        assert!(err.to_string().contains("Invalid month"));
    }

    #[test]
    fn rejects_malformed_dates() {
        for value in ["", "2015/06/01", "2015-6", "2015-06-1", "June 2015"] {
            let err = validate_since(value).expect_err("malformed date should fail");
            assert!(
                err.to_string().contains("--since"),
                "unexpected error for {value:?}: {err}"
            );
        }
    }

    #[test]
    fn is_day_shaped_accepts_real_days_only() {
        for value in ["2014-09-04", "2025-01-23", "2024-02-29"] {
            assert!(is_day_shaped(value), "expected day shape: {value}");
        }
    }

    #[test]
    fn is_day_shaped_rejects_provider_spellings_and_impossible_days() {
        for value in [
            "",
            "01 Apr 2019",
            "2019-4-11",
            "2019-13-01",
            "2021-02-30",
            "2023-02-29",
            "2023-00-10",
            "2023-10-00",
        ] {
            assert!(!is_day_shaped(value), "expected rejection: {value:?}");
        }
    }
}
