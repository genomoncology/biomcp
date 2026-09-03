//! Tests for ESSIE trial search helpers.

use super::super::eligibility::collect_eligibility_keywords;
use super::*;

#[test]
fn criteria_query_and_verification_share_case_sensitive_operator_rules() {
    let corpus = [
        (
            "patients not previously treated",
            "\"patients not previously treated\"",
            true,
        ),
        (
            "measurable disease and adequate organ function",
            "\"measurable disease and adequate organ function\"",
            true,
        ),
        (
            "chemotherapy or radiotherapy",
            "\"chemotherapy or radiotherapy\"",
            true,
        ),
        ("BRAF OR NRAS", "\"BRAF\" OR \"NRAS\"", false),
        ("BRAF AND NOT EGFR", "\"BRAF\" AND NOT \"EGFR\"", false),
    ];

    for (criteria, expected_query, verification_enabled) in corpus {
        assert_eq!(
            essie_escape_boolean_expression(criteria),
            expected_query,
            "unexpected ESSIE query for {criteria:?}"
        );
        let filters = TrialSearchFilters {
            criteria: Some(criteria.into()),
            ..Default::default()
        };
        assert_eq!(
            collect_eligibility_keywords(&filters).contains(&criteria.to_string()),
            verification_enabled,
            "query and eligibility verification disagree for {criteria:?}"
        );
    }
}

#[test]
fn essie_escape_boolean_expression_preserves_or_operators() {
    assert_eq!(
        essie_escape_boolean_expression("dMMR OR MSI-H"),
        "\"dMMR\" OR \"MSI\\-H\""
    );
}

#[test]
fn essie_escape_boolean_expression_handles_leading_not() {
    assert_eq!(
        essie_escape_boolean_expression("NOT MSI-H"),
        "NOT \"MSI\\-H\""
    );
}

#[test]
fn essie_escape_boolean_expression_handles_and_not() {
    assert_eq!(
        essie_escape_boolean_expression("dMMR AND NOT MSI-H"),
        "\"dMMR\" AND NOT \"MSI\\-H\""
    );
}

#[test]
fn line_of_therapy_patterns_accepts_supported_values() {
    assert!(line_of_therapy_patterns("1L").is_some());
    assert!(line_of_therapy_patterns("2L").is_some());
    assert!(line_of_therapy_patterns("3L+").is_some());
    assert!(line_of_therapy_patterns("2l").is_some());
}

#[test]
fn line_of_therapy_patterns_rejects_invalid_values() {
    assert!(line_of_therapy_patterns("4L").is_none());
    assert!(line_of_therapy_patterns("frontline").is_none());
}
