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

#[cfg(test)]
mod tests {
    use super::validate_since;

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
    fn rejects_invalid_month() {
        let err = validate_since("2015-13").expect_err("month should fail");
        assert!(err.to_string().contains("Invalid month"));
    }
}
