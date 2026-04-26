//! Runner and report assembly tests for `biomcp health`.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::super::catalog::{ProbeKind, SourceDescriptor};
use super::super::runner::{
    HEALTH_API_PROBE_CONCURRENCY_LIMIT, ProbeClass, ProbeOutcome,
    probe_source_with_timeout_for_test, report_from_outcomes, run_buffered_in_order,
};
use super::super::{HealthReport, HealthRow};
use super::{block_on, env_lock, update_max};
use crate::test_support::set_env_var;
#[test]
fn markdown_shows_affects_column_when_present() {
    let report = HealthReport {
        healthy: 1,
        warning: 0,
        excluded: 0,
        total: 2,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: "ok".into(),
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
            },
            HealthRow {
                api: "OpenFDA".into(),
                status: "error".into(),
                latency: "timeout".into(),
                affects: Some("adverse-event search".into()),
                key_configured: None,
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
        total: 2,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: "ok".into(),
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
            },
            HealthRow {
                api: "MyVariant".into(),
                status: "ok".into(),
                latency: "11ms".into(),
                affects: None,
                key_configured: None,
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
        total: 1,
        rows: vec![HealthRow {
            api: "OncoKB".into(),
            status: "ok".into(),
            latency: "10ms".into(),
            affects: None,
            key_configured: Some(true),
        }],
    };

    assert_eq!(report.rows[0].status, "ok");
    let md = report.to_markdown();
    assert!(md.contains("| OncoKB | ok (key configured) | 10ms |"));
}

#[test]
fn markdown_decorates_keyed_error_rows_without_changing_status() {
    let report = HealthReport {
        healthy: 0,
        warning: 0,
        excluded: 0,
        total: 1,
        rows: vec![HealthRow {
            api: "OncoKB".into(),
            status: "error".into(),
            latency: "10ms (HTTP 401)".into(),
            affects: Some("variant oncokb command and variant evidence section".into()),
            key_configured: Some(true),
        }],
    };

    assert_eq!(report.rows[0].status, "error");
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
            status: "ok".into(),
            latency: "10ms".into(),
            affects: None,
            key_configured: None,
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
        status: "ok".into(),
        latency: "10ms".into(),
        affects: None,
        key_configured: Some(true),
    })
    .expect("serialize keyed row");

    assert_eq!(value["status"], "ok");
    assert_eq!(value["key_configured"], true);
}

#[test]
fn all_healthy_includes_warning_and_excluded_rows() {
    let report = HealthReport {
        healthy: 1,
        warning: 1,
        excluded: 1,
        total: 3,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: "ok".into(),
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
            },
            HealthRow {
                api: "OncoKB".into(),
                status: "excluded (set ONCOKB_TOKEN)".into(),
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
            },
            HealthRow {
                api: "Cache limits".into(),
                status: "warning".into(),
                latency: "referenced bytes 12 exceed max_size 8; run biomcp cache clean".into(),
                affects: None,
                key_configured: None,
            },
        ],
    };

    assert!(report.all_healthy());
}

#[test]
fn markdown_summary_reports_ok_error_excluded_and_warning_counts() {
    let report = HealthReport {
        healthy: 1,
        warning: 1,
        excluded: 1,
        total: 4,
        rows: vec![
            HealthRow {
                api: "MyGene".into(),
                status: "ok".into(),
                latency: "10ms".into(),
                affects: None,
                key_configured: None,
            },
            HealthRow {
                api: "OpenFDA".into(),
                status: "error".into(),
                latency: "timeout".into(),
                affects: Some("adverse-event search".into()),
                key_configured: None,
            },
            HealthRow {
                api: "OncoKB".into(),
                status: "excluded (set ONCOKB_TOKEN)".into(),
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
            },
            HealthRow {
                api: "Cache limits".into(),
                status: "warning".into(),
                latency: "available disk 10 B is below min_disk_free 20 B; run biomcp cache clean"
                    .into(),
                affects: None,
                key_configured: None,
            },
        ],
    };

    let md = report.to_markdown();
    assert!(md.contains("Status: 1 ok, 1 error, 1 excluded, 1 warning"));
}

#[test]
fn report_counts_use_probe_class_not_status_prefixes() {
    let report = report_from_outcomes(vec![
        ProbeOutcome {
            row: HealthRow {
                api: "Semantic Scholar".into(),
                status: "available (unauthenticated, shared rate limit)".into(),
                latency: "15ms".into(),
                affects: None,
                key_configured: Some(false),
            },
            class: ProbeClass::Healthy,
        },
        ProbeOutcome {
            row: HealthRow {
                api: "OncoKB".into(),
                status: "excluded (set ONCOKB_TOKEN)".into(),
                latency: "n/a".into(),
                affects: Some("variant oncokb command and variant evidence section".into()),
                key_configured: Some(false),
            },
            class: ProbeClass::Excluded,
        },
        ProbeOutcome {
            row: HealthRow {
                api: "Cache limits".into(),
                status: "warning".into(),
                latency: "referenced bytes 12 exceed max_size 8; run biomcp cache clean".into(),
                affects: None,
                key_configured: None,
            },
            class: ProbeClass::Warning,
        },
    ]);

    assert_eq!(report.healthy, 1);
    assert_eq!(report.warning, 1);
    assert_eq!(report.excluded, 1);
    assert_eq!(report.total, 3);
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
    let _lock = env_lock();
    let server = block_on(MockServer::start());
    let slow_url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());

    block_on(async {
        Mock::given(method("GET"))
            .and(path("/health"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
            .expect(3)
            .mount(&server)
            .await;
    });

    let optional_source = SourceDescriptor {
        api: "Semantic Scholar",
        affects: Some("Semantic Scholar features"),
        probe: ProbeKind::OptionalAuthGet {
            url: slow_url,
            env_var: "S2_API_KEY",
            header_name: "x-api-key",
            header_value_prefix: "",
            unauthenticated_ok_status: "available (unauthenticated, shared rate limit)",
            authenticated_ok_status: "configured (authenticated)",
            unauthenticated_rate_limited_status: Some(
                "unavailable (set S2_API_KEY for reliable access)",
            ),
        },
    };
    let _optional_env = set_env_var("S2_API_KEY", None);
    let optional_outcome = block_on(probe_source_with_timeout_for_test(
        reqwest::Client::new(),
        optional_source,
        Duration::from_millis(10),
    ));
    assert_eq!(optional_outcome.class, ProbeClass::Error);
    assert_eq!(optional_outcome.row.status, "error");
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
            url: slow_url,
            env_var: "ONCOKB_TOKEN",
            header_name: "Authorization",
            header_value_prefix: "Bearer ",
        },
    };
    let _auth_env = set_env_var("ONCOKB_TOKEN", Some("test-token"));
    let auth_outcome = block_on(probe_source_with_timeout_for_test(
        reqwest::Client::new(),
        auth_source,
        Duration::from_millis(10),
    ));
    assert_eq!(auth_outcome.class, ProbeClass::Error);
    assert_eq!(auth_outcome.row.status, "error");
    assert_eq!(auth_outcome.row.latency, "10ms (timeout)");
    assert_eq!(
        auth_outcome.row.affects.as_deref(),
        Some("variant oncokb command and variant evidence section")
    );
    assert_eq!(auth_outcome.row.key_configured, Some(true));

    let public_source = SourceDescriptor {
        api: "MyGene",
        affects: Some("gene search and gene get"),
        probe: ProbeKind::Get { url: slow_url },
    };
    let public_outcome = block_on(probe_source_with_timeout_for_test(
        reqwest::Client::new(),
        public_source,
        Duration::from_millis(10),
    ));
    assert_eq!(public_outcome.class, ProbeClass::Error);
    assert_eq!(public_outcome.row.status, "error");
    assert_eq!(public_outcome.row.latency, "10ms (timeout)");
    assert_eq!(
        public_outcome.row.affects.as_deref(),
        Some("gene search and gene get")
    );
    assert_eq!(public_outcome.row.key_configured, None);
}
