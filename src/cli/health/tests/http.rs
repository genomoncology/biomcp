//! HTTP/auth probe tests for `biomcp health`.

use reqwest::StatusCode;
use std::time::Instant;

use super::super::catalog::health_sources;
use super::super::http::{
    configured_key_from_value, excluded_outcome, optional_auth_status_outcome, vaers_query_outcome,
};
use super::super::runner::{ProbeClass, ProbeOutcome, report_from_outcomes};
use super::super::{HealthRow, HealthStatus};
use super::assert_millisecond_latency;

fn semantic_scholar_optional_outcome(status: StatusCode, key_configured: bool) -> ProbeOutcome {
    optional_auth_status_outcome(
        "Semantic Scholar",
        status,
        7,
        Some(key_configured),
        "S2_API_KEY",
        Some("Semantic Scholar features"),
    )
}
#[test]
fn vaers_query_success_reports_healthy_row() {
    let outcome = vaers_query_outcome(
        "CDC WONDER VAERS",
        Some("adverse-event vaers"),
        Instant::now(),
        Ok(()),
    );

    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.api, "CDC WONDER VAERS");
    assert_eq!(outcome.row.status, HealthStatus::Ok);
    assert_eq!(outcome.row.affects, None);
    assert_millisecond_latency(&outcome.row.latency);
}

#[test]
fn vaers_query_error_reports_error_row_with_affects() {
    let outcome = vaers_query_outcome(
        "CDC WONDER VAERS",
        Some("adverse-event vaers"),
        Instant::now(),
        Err(crate::error::BioMcpError::Api {
            api: "vaers".to_string(),
            message: "bad gateway".to_string(),
        }),
    );

    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.api, "CDC WONDER VAERS");
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert!(outcome.row.latency.ends_with("ms (error)"));
    assert_eq!(outcome.row.affects.as_deref(), Some("adverse-event vaers"));
    assert_eq!(outcome.row.key_configured, None);
}

#[test]
fn key_gated_source_is_excluded_when_env_missing() {
    assert!(configured_key_from_value(None).is_none());
    let source = health_sources()
        .iter()
        .find(|source| source.api == "OncoKB")
        .expect("oncokb health source");

    let outcome = excluded_outcome("OncoKB", "ONCOKB_TOKEN", source.affects);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(outcome.row.status, HealthStatus::Excluded);
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some("variant oncokb command and variant evidence section")
    );
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn excluded_key_gated_row_serializes_key_configured_false() {
    let report = report_from_outcomes(vec![ProbeOutcome {
        row: HealthRow {
            api: "OncoKB".into(),
            status: HealthStatus::Excluded,
            latency: "n/a".into(),
            affects: Some("variant oncokb command and variant evidence section".into()),
            key_configured: Some(false),
            local_path: None,
            stale: None,
            required_env_var: Some("ONCOKB_TOKEN".into()),
            missing_files: None,
            not_built: None,
        },
        class: ProbeClass::Excluded,
    }]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("oncokb row");

    assert_eq!(row["status"], "excluded");
    assert_eq!(row["required_env_var"], "ONCOKB_TOKEN");
    assert_eq!(row["key_configured"], false);
}

#[test]
fn empty_key_is_treated_as_missing() {
    assert!(configured_key_from_value(Some("   ".to_string())).is_none());
    let source = health_sources()
        .iter()
        .find(|source| source.api == "NCI CTS")
        .expect("nci health source");

    let outcome = excluded_outcome("NCI CTS", "NCI_API_KEY", source.affects);

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(outcome.row.status, HealthStatus::Excluded);
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn optional_auth_get_reports_unauthed_semantic_scholar_as_healthy() {
    let outcome = semantic_scholar_optional_outcome(StatusCode::OK, false);
    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Available);
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn optional_auth_get_reports_authed_semantic_scholar_as_configured() {
    let outcome = semantic_scholar_optional_outcome(StatusCode::OK, true);
    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Configured);
    assert_eq!(outcome.row.key_configured, Some(true));
}

#[test]
fn optional_auth_get_reports_unauthenticated_429_as_unavailable() {
    let outcome = semantic_scholar_optional_outcome(StatusCode::TOO_MANY_REQUESTS, false);
    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, HealthStatus::Unavailable);
    assert_eq!(outcome.row.required_env_var.as_deref(), Some("S2_API_KEY"));
    assert_millisecond_latency(&outcome.row.latency);
    assert!(!outcome.row.latency.contains("HTTP 429"));
    assert_eq!(outcome.row.affects, None);
    assert_eq!(outcome.row.key_configured, Some(false));

    let report = report_from_outcomes(vec![outcome.clone()]);
    assert_eq!(report.healthy, 1);
    assert_eq!(report.excluded, 0);
    assert_eq!(report.total, 1);
    assert!(report.all_healthy());

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("semantic scholar row");
    assert!(row.get("affects").is_none());
    assert_eq!(row["key_configured"], false);

    let md = report_from_outcomes(vec![
        outcome.clone(),
        ProbeOutcome {
            row: HealthRow {
                api: "OpenFDA".into(),
                status: HealthStatus::Error,
                latency: "timeout".into(),
                affects: Some("adverse-event search".into()),
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            class: ProbeClass::Error,
        },
    ])
    .to_markdown();
    assert!(md.contains(&format!(
        "| Semantic Scholar | unavailable (set S2_API_KEY for reliable access) | {} | - |",
        outcome.row.latency
    )));
}

#[test]
fn optional_auth_get_reports_unauthenticated_non_429_as_error() {
    let outcome = semantic_scholar_optional_outcome(StatusCode::FORBIDDEN, false);
    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert!(outcome.row.latency.contains("HTTP 403"));
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some("Semantic Scholar features")
    );
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn optional_auth_get_reports_authenticated_429_as_error() {
    let outcome = semantic_scholar_optional_outcome(StatusCode::TOO_MANY_REQUESTS, true);
    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, HealthStatus::Error);
    assert!(outcome.row.latency.contains("HTTP 429"));
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some("Semantic Scholar features")
    );
    assert_eq!(outcome.row.key_configured, Some(true));
}
