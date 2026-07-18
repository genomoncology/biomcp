//! Tier 3 — response parsing and funding-section mapping. Pure: feeds committed
//! fixture bytes to `decode_json` and pure mappers. No network, no server.

use super::super::*;
use crate::sources::decode_json;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/nih_reporter/",
            $name
        ))
    };
}

fn json_ct() -> HeaderValue {
    HeaderValue::from_static("application/json")
}

fn test_grant(
    project_num: &str,
    core_project_num: Option<&str>,
    fiscal_year: i32,
    award_amount: u64,
) -> NihReporterGrant {
    NihReporterGrant {
        project_title: format!("Project {project_num}"),
        project_num: project_num.to_string(),
        core_project_num: core_project_num.map(str::to_string),
        project_detail_url: Some(format!("https://example.org/{project_num}")),
        pi_name: Some("Example PI".to_string()),
        organization: Some("Example Org".to_string()),
        fiscal_year,
        award_amount,
    }
}

#[test]
fn parses_funding_response_fixture_and_maps_section() {
    let response: NihReporterSearchResponse = decode_json(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NIH_REPORTER),
        StatusCode::OK,
        Some(&json_ct()),
        fixture!("funding_erbb2.json"),
        true,
    )
    .unwrap();
    let section = NihReporterClient::funding_section_from_response(
        "ERBB2".to_string(),
        vec![2022, 2023, 2024, 2025, 2026],
        response,
    );

    assert_eq!(section.query, "ERBB2");
    assert_eq!(section.fiscal_years, vec![2022, 2023, 2024, 2025, 2026]);
    assert_eq!(section.matching_project_years, 4);
    assert_eq!(section.grants.len(), 2);
    assert_eq!(section.grants[0].project_num, "P-1");
    assert_eq!(section.grants[0].pi_name.as_deref(), Some("DOE, JANE"));
    assert_eq!(section.grants[0].organization.as_deref(), Some("Org A"));
    assert_eq!(section.grants[1].project_num, "P-3");
    assert_eq!(section.grants[1].pi_name.as_deref(), Some("Ada Lovelace"));
}

#[test]
fn map_project_year_row_prefers_contact_pi_then_contact_investigator_then_first_pi() {
    let contact_name_row = NihReporterProjectYearRow {
        project_title: Some("Example".to_string()),
        principal_investigators: vec![NihReporterPrincipalInvestigator {
            full_name: Some("Ignored PI".to_string()),
            first_name: None,
            middle_name: None,
            last_name: None,
            is_contact_pi: Some(true),
        }],
        contact_pi_name: Some("  DOE, JANE  ".to_string()),
        organization: None,
        fiscal_year: Some(2026),
        award_amount: Some(1),
        project_num: Some("P1".to_string()),
        core_project_num: None,
        project_detail_url: None,
    };
    assert_eq!(
        map_project_year_row(contact_name_row)
            .and_then(|grant| grant.pi_name)
            .as_deref(),
        Some("DOE, JANE")
    );

    let contact_investigator_row = NihReporterProjectYearRow {
        project_title: Some("Example".to_string()),
        principal_investigators: vec![
            NihReporterPrincipalInvestigator {
                full_name: Some("Other PI".to_string()),
                first_name: None,
                middle_name: None,
                last_name: None,
                is_contact_pi: Some(false),
            },
            NihReporterPrincipalInvestigator {
                full_name: Some("Contact PI".to_string()),
                first_name: None,
                middle_name: None,
                last_name: None,
                is_contact_pi: Some(true),
            },
        ],
        contact_pi_name: None,
        organization: None,
        fiscal_year: Some(2026),
        award_amount: Some(1),
        project_num: Some("P2".to_string()),
        core_project_num: None,
        project_detail_url: None,
    };
    assert_eq!(
        map_project_year_row(contact_investigator_row)
            .and_then(|grant| grant.pi_name)
            .as_deref(),
        Some("Contact PI")
    );

    let first_investigator_row = NihReporterProjectYearRow {
        project_title: Some("Example".to_string()),
        principal_investigators: vec![NihReporterPrincipalInvestigator {
            full_name: None,
            first_name: Some("Ada".to_string()),
            middle_name: Some("M".to_string()),
            last_name: Some("Lovelace".to_string()),
            is_contact_pi: None,
        }],
        contact_pi_name: None,
        organization: None,
        fiscal_year: Some(2026),
        award_amount: Some(1),
        project_num: Some("P3".to_string()),
        core_project_num: None,
        project_detail_url: None,
    };
    assert_eq!(
        map_project_year_row(first_investigator_row)
            .and_then(|grant| grant.pi_name)
            .as_deref(),
        Some("Ada M Lovelace")
    );
}

#[test]
fn deduplicate_grants_groups_by_core_project_num_then_project_num() {
    let deduped = deduplicate_grants(vec![
        test_grant("P-002", Some("CORE-A"), 2025, 250),
        test_grant("P-001", Some("CORE-A"), 2026, 250),
        test_grant("P-100", None, 2024, 180),
        test_grant("P-100", None, 2023, 400),
        test_grant("P-200", Some("CORE-B"), 2025, 300),
    ]);

    assert_eq!(deduped.len(), 3);
    assert_eq!(deduped[0].project_num, "P-100");
    assert_eq!(deduped[0].award_amount, 400);
    assert_eq!(deduped[1].project_num, "P-200");
    assert_eq!(deduped[2].project_num, "P-001");
}

#[test]
fn deduplicate_grants_truncates_to_top_ten_after_sorting() {
    let grants = (0..12)
        .map(|idx| test_grant(&format!("P-{idx:03}"), None, 2026, 1_000 - idx))
        .collect::<Vec<_>>();

    let deduped = deduplicate_grants(grants);

    assert_eq!(deduped.len(), 10);
    assert_eq!(deduped[0].project_num, "P-000");
    assert_eq!(deduped[9].project_num, "P-009");
}

#[test]
fn decode_json_maps_http_error_status_with_excerpt() {
    let err = decode_json::<NihReporterSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NIH_REPORTER),
        StatusCode::INTERNAL_SERVER_ERROR,
        None,
        b"upstream failure",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert_eq!(err.code(), "api");
    assert!(msg.contains("NIH RePORTER"), "got: {msg}");
    assert!(msg.contains("500"), "got: {msg}");
}

#[test]
fn decode_json_rejects_non_json_content_type() {
    let html = HeaderValue::from_static("text/html");
    let err = decode_json::<NihReporterSearchResponse>(
        crate::error::SourceContext::retry(crate::error::SourceProvider::NIH_REPORTER),
        StatusCode::OK,
        Some(&html),
        b"<html><body>error</body></html>",
        true,
    )
    .unwrap_err();
    let msg = format!("{err:?}");
    assert!(msg.contains("NIH RePORTER"), "got: {msg}");
    assert!(msg.contains("HTML"), "got: {msg}");
}
