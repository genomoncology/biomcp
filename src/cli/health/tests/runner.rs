//! Runner and report assembly tests for `biomcp health`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::super::HealthStatus;
use super::super::catalog::{ProbeKind, SourceDescriptor};
use super::super::runner::{
    HEALTH_API_PROBE_CONCURRENCY_LIMIT, ProbeClass, ProbeOutcome, report_from_outcomes,
    run_buffered_in_order, timed_out_probe_outcome_for_test,
};
use super::super::{HealthReport, HealthRow};
use super::{block_on, update_max};
#[test]
fn markdown_shows_affects_column_when_present() {
    let report = HealthReport {
        healthy: 1,
        warning: 0,
        excluded: 0,
        error: 1,
        total: 2,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: HealthStatus::Ok,
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            HealthRow {
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
        ],
    };
    let md = report.to_markdown();
    assert!(md.contains("| API | Status | Latency | Affects |"));
    assert!(md.contains("adverse-event search"));
}

#[test]
fn markdown_omits_affects_column_when_all_healthy() {
    let report = HealthReport {
        healthy: 2,
        warning: 0,
        excluded: 0,
        error: 0,
        total: 2,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: HealthStatus::Ok,
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            HealthRow {
                api: "MyVariant".into(),
                status: HealthStatus::Ok,
                latency: "11ms".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
        ],
    };
    let md = report.to_markdown();
    assert!(md.contains("| API | Status | Latency |"));
    assert!(!md.contains("| API | Status | Latency | Affects |"));
}

#[test]
fn markdown_decorates_keyed_success_rows_without_changing_status() {
    let report = HealthReport {
        healthy: 1,
        warning: 0,
        excluded: 0,
        error: 0,
        total: 1,
        rows: vec![HealthRow {
            api: "OncoKB".into(),
            status: HealthStatus::Ok,
            latency: "10ms".into(),
            affects: None,
            key_configured: Some(true),
            local_path: None,
            stale: None,
            required_env_var: None,
            missing_files: None,
            not_built: None,
        }],
    };

    assert_eq!(report.rows[0].status, HealthStatus::Ok);
    let md = report.to_markdown();
    assert!(md.contains("| OncoKB | ok (key configured) | 10ms |"));
}

#[test]
fn markdown_decorates_keyed_error_rows_without_changing_status() {
    let report = HealthReport {
        healthy: 0,
        warning: 0,
        excluded: 0,
        error: 1,
        total: 1,
        rows: vec![HealthRow {
            api: "OncoKB".into(),
            status: HealthStatus::Error,
            latency: "10ms (HTTP 401)".into(),
            affects: Some("variant oncokb command and variant evidence section".into()),
            key_configured: Some(true),
            local_path: None,
            stale: None,
            required_env_var: None,
            missing_files: None,
            not_built: None,
        }],
    };

    assert_eq!(report.rows[0].status, HealthStatus::Error);
    let md = report.to_markdown();
    assert!(md.contains(
        "| OncoKB | error (key configured) | 10ms (HTTP 401) | variant oncokb command and variant evidence section |",
    ));
}

#[test]
fn public_row_omits_key_configured_in_json() {
    let report = report_from_outcomes(vec![ProbeOutcome {
        row: HealthRow {
            api: "MyGene".into(),
            status: HealthStatus::Ok,
            latency: "10ms".into(),
            affects: None,
            key_configured: None,
            local_path: None,
            stale: None,
            required_env_var: None,
            missing_files: None,
            not_built: None,
        },
        class: ProbeClass::Healthy,
    }]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("mygene row");

    assert!(row.get("key_configured").is_none());
}

#[test]
fn keyed_row_serializes_raw_status_with_key_configured_true() {
    let value = serde_json::to_value(HealthRow {
        api: "OncoKB".into(),
        status: HealthStatus::Ok,
        latency: "10ms".into(),
        affects: None,
        key_configured: Some(true),
        local_path: None,
        stale: None,
        required_env_var: None,
        missing_files: None,
        not_built: None,
    })
    .expect("serialize keyed row");

    assert_eq!(value["status"], "ok");
    assert_eq!(value["key_configured"], true);
}

#[test]
fn not_built_row_serializes_fact_and_preserves_markdown() {
    let row = HealthRow {
        api: "AlphaGenome".into(),
        status: HealthStatus::Unavailable,
        latency: "-".into(),
        affects: Some("variant predict section".into()),
        key_configured: None,
        local_path: None,
        stale: None,
        required_env_var: None,
        missing_files: None,
        not_built: Some(true),
    };

    let value = serde_json::to_value(&row).expect("serialize unavailable row");
    assert_eq!(value["status"], "unavailable");
    assert_eq!(value["not_built"], true);

    let report = HealthReport {
        healthy: 0,
        warning: 0,
        excluded: 1,
        error: 0,
        total: 1,
        rows: vec![row],
    };
    assert!(
        report
            .to_markdown()
            .contains("| AlphaGenome | unavailable (not built) | - | variant predict section |")
    );
}

#[test]
fn all_healthy_includes_warning_and_excluded_rows() {
    let report = HealthReport {
        healthy: 1,
        warning: 1,
        excluded: 1,
        error: 0,
        total: 3,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: HealthStatus::Ok,
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            HealthRow {
                api: "OncoKB".into(),
                status: HealthStatus::Excluded,
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            HealthRow {
                api: "Cache limits".into(),
                status: HealthStatus::Warning,
                latency: "referenced bytes 12 exceed max_size 8; run biomcp cache clean".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
        ],
    };

    assert!(report.all_healthy());
    assert_eq!(
        report.healthy + report.warning + report.excluded + report.error,
        report.total
    );
}

#[test]
fn markdown_summary_reports_ok_error_excluded_and_warning_counts() {
    let report = HealthReport {
        healthy: 1,
        warning: 1,
        excluded: 1,
        error: 1,
        total: 4,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: HealthStatus::Ok,
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            HealthRow {
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
            HealthRow {
                api: "OncoKB".into(),
                status: HealthStatus::Excluded,
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            HealthRow {
                api: "Cache limits".into(),
                status: HealthStatus::Warning,
                latency: "available disk 10 B is below min_disk_free 20 B; run biomcp cache clean"
                    .into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
        ],
    };

    let md = report.to_markdown();
    assert!(md.contains("Status: 1 ok, 1 error, 1 excluded, 1 warning"));
}

#[test]
fn empty_report_counts_reconcile() {
    let report = report_from_outcomes(Vec::new());

    assert_eq!(report.healthy, 0);
    assert_eq!(report.warning, 0);
    assert_eq!(report.excluded, 0);
    assert_eq!(report.error, 0);
    assert_eq!(report.total, 0);
}

#[test]
fn report_counts_use_probe_class_not_status_prefixes() {
    let report = report_from_outcomes(vec![
        ProbeOutcome {
            row: HealthRow {
                api: "Semantic Scholar".into(),
                status: HealthStatus::Available,
                latency: "15ms".into(),
                affects: None,
                key_configured: Some(false),
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            class: ProbeClass::Healthy,
        },
        ProbeOutcome {
            row: HealthRow {
                api: "OncoKB".into(),
                status: HealthStatus::Excluded,
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            class: ProbeClass::Excluded,
        },
        ProbeOutcome {
            row: HealthRow {
                api: "Cache limits".into(),
                status: HealthStatus::Warning,
                latency: "referenced bytes 12 exceed max_size 8; run biomcp cache clean".into(),
                affects: None,
                key_configured: None,
                local_path: None,
                stale: None,
                required_env_var: None,
                missing_files: None,
                not_built: None,
            },
            class: ProbeClass::Warning,
        },
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
    ]);

    assert_eq!(report.healthy, 1);
    assert_eq!(report.warning, 1);
    assert_eq!(report.excluded, 1);
    assert_eq!(report.error, 1);
    assert_eq!(report.total, 4);
    assert_eq!(
        report.healthy + report.warning + report.excluded + report.error,
        report.total
    );
    assert!(!report.all_healthy());

    let value = serde_json::to_value(&report).expect("serialize health report");
    assert_eq!(value["error"], 1);
}

#[test]
fn health_probes_respect_concurrency_limit_and_source_order() {
    let input: Vec<_> = (0..(HEALTH_API_PROBE_CONCURRENCY_LIMIT + 5)).collect();
    let in_flight = Arc::new(AtomicUsize::new(0));
    let max_in_flight = Arc::new(AtomicUsize::new(0));

    let output = block_on(run_buffered_in_order(
        input.clone(),
        HEALTH_API_PROBE_CONCURRENCY_LIMIT,
        {
            let in_flight = Arc::clone(&in_flight);
            let max_in_flight = Arc::clone(&max_in_flight);
            move |index| {
                let in_flight = Arc::clone(&in_flight);
                let max_in_flight = Arc::clone(&max_in_flight);
                async move {
                    let current = in_flight.fetch_add(1, Ordering::SeqCst) + 1;
                    update_max(&max_in_flight, current);
                    tokio::time::sleep(Duration::from_millis(25)).await;
                    in_flight.fetch_sub(1, Ordering::SeqCst);
                    index
                }
            }
        },
    ));

    assert_eq!(output, input);
    assert_eq!(
        max_in_flight.load(Ordering::SeqCst),
        HEALTH_API_PROBE_CONCURRENCY_LIMIT
    );
    assert_eq!(in_flight.load(Ordering::SeqCst), 0);
}

#[test]
fn timed_out_probe_returns_error_row_with_timeout_latency() {
    let optional_source = SourceDescriptor {
        api: "Semantic Scholar",
        affects: Some("Semantic Scholar features"),
        probe: ProbeKind::OptionalAuthGet {
            url: "https://example.test/health",
            env_var: "S2_API_KEY",
            header_name: "x-api-key",
            header_value_prefix: "",
        },
    };
    let optional_outcome =
        timed_out_probe_outcome_for_test(optional_source, Duration::from_millis(10), |_| None);
    assert_eq!(optional_outcome.class, ProbeClass::Error);
    assert_eq!(optional_outcome.row.status, HealthStatus::Error);
    assert_eq!(optional_outcome.row.latency, "10ms (timeout)");
    assert_eq!(
        optional_outcome.row.affects.as_deref(),
        Some("Semantic Scholar features")
    );
    assert_eq!(optional_outcome.row.key_configured, Some(false));

    let auth_source = SourceDescriptor {
        api: "OncoKB",
        affects: Some("variant oncokb command and variant evidence section"),
        probe: ProbeKind::AuthGet {
            url: "https://example.test/health",
            env_var: "ONCOKB_TOKEN",
            header_name: "Authorization",
            header_value_prefix: "Bearer ",
        },
    };
    let auth_outcome =
        timed_out_probe_outcome_for_test(auth_source, Duration::from_millis(10), |_| None);
    assert_eq!(auth_outcome.class, ProbeClass::Error);
    assert_eq!(auth_outcome.row.status, HealthStatus::Error);
    assert_eq!(auth_outcome.row.latency, "10ms (timeout)");
    assert_eq!(
        auth_outcome.row.affects.as_deref(),
        Some("variant oncokb command and variant evidence section")
    );
    assert_eq!(auth_outcome.row.key_configured, Some(true));

    let public_source = SourceDescriptor {
        api: "MyGene",
        affects: Some("gene search and gene get"),
        probe: ProbeKind::Get {
            url: "https://example.test/health",
        },
    };
    let public_outcome =
        timed_out_probe_outcome_for_test(public_source, Duration::from_millis(10), |_| None);
    assert_eq!(public_outcome.class, ProbeClass::Error);
    assert_eq!(public_outcome.row.status, HealthStatus::Error);
    assert_eq!(public_outcome.row.latency, "10ms (timeout)");
    assert_eq!(
        public_outcome.row.affects.as_deref(),
        Some("gene search and gene get")
    );
    assert_eq!(public_outcome.row.key_configured, None);
}
