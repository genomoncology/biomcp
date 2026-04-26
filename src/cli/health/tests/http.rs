//! HTTP/auth probe tests for `biomcp health`.

use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

use super::super::HealthRow;
use super::super::catalog::health_sources;
use super::super::runner::{ProbeClass, ProbeOutcome, probe_source, report_from_outcomes};
use super::{assert_millisecond_latency, block_on, env_lock, semantic_scholar_source};
use crate::test_support::set_env_var;
#[test]
fn probe_source_runs_vaers_query_against_fixture_server() {
    const REACTIONS_RESPONSE_FIXTURE: &str =
        include_str!("../../../../spec/fixtures/vaers/reactions-response.xml");

    let _env_lock = env_lock();
    block_on(async {
        let server = MockServer::start().await;
        let _vaers_env = set_env_var("BIOMCP_VAERS_BASE", Some(&server.uri()));
        let source = health_sources()
            .iter()
            .find(|source| source.api == "CDC WONDER VAERS")
            .expect("vaers health source");

        Mock::given(method("POST"))
            .and(path("/controller/datarequest/D8"))
            .and(body_string_contains("request_xml="))
            .and(body_string_contains("MMR"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/html; charset=ISO-8859-1")
                    .set_body_raw(REACTIONS_RESPONSE_FIXTURE, "text/html; charset=ISO-8859-1"),
            )
            .mount(&server)
            .await;

        let outcome = probe_source(reqwest::Client::new(), source).await;
        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.api, "CDC WONDER VAERS");
        assert_eq!(outcome.row.status, "ok");
        assert_eq!(outcome.row.affects, None);
        assert_millisecond_latency(&outcome.row.latency);
    });
}

#[test]
fn key_gated_source_is_excluded_when_env_missing() {
    let _lock = env_lock();
    let _env = set_env_var("ONCOKB_TOKEN", None);
    let source = health_sources()
        .iter()
        .find(|source| source.api == "OncoKB")
        .expect("oncokb health source");

    let outcome = block_on(probe_source(reqwest::Client::new(), source));

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(outcome.row.status, "excluded (set ONCOKB_TOKEN)");
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
            status: "excluded (set ONCOKB_TOKEN)".into(),
            latency: "n/a".into(),
            affects: Some("variant oncokb command and variant evidence section".into()),
            key_configured: Some(false),
        },
        class: ProbeClass::Excluded,
    }]);

    let value = serde_json::to_value(&report).expect("serialize health report");
    let rows = value["rows"].as_array().expect("rows array");
    let row = rows.first().expect("oncokb row");

    assert_eq!(row["status"], "excluded (set ONCOKB_TOKEN)");
    assert_eq!(row["key_configured"], false);
}

#[test]
fn empty_key_is_treated_as_missing() {
    let _lock = env_lock();
    let _env = set_env_var("NCI_API_KEY", Some("   "));
    let source = health_sources()
        .iter()
        .find(|source| source.api == "NCI CTS")
        .expect("nci health source");

    let outcome = block_on(probe_source(reqwest::Client::new(), source));

    assert_eq!(outcome.class, ProbeClass::Excluded);
    assert_eq!(outcome.row.status, "excluded (set NCI_API_KEY)");
    assert_eq!(outcome.row.latency, "n/a");
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn optional_auth_get_reports_unauthed_semantic_scholar_as_healthy() {
    let _lock = env_lock();
    let _env = set_env_var("S2_API_KEY", None);
    let server = block_on(MockServer::start());
    let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
    let source = semantic_scholar_source(url);

    block_on(async {
        Mock::given(method("GET"))
            .and(path("/health"))
            .and(|request: &wiremock::Request| !request.headers.contains_key("x-api-key"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
    });

    let outcome = block_on(probe_source(reqwest::Client::new(), &source));
    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.status,
        "available (unauthenticated, shared rate limit)"
    );
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn optional_auth_get_reports_authed_semantic_scholar_as_configured() {
    let _lock = env_lock();
    let _env = set_env_var("S2_API_KEY", Some("test-key-abc"));
    let server = block_on(MockServer::start());
    let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
    let source = semantic_scholar_source(url);

    block_on(async {
        Mock::given(method("GET"))
            .and(path("/health"))
            .and(header("x-api-key", "test-key-abc"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;
    });

    let outcome = block_on(probe_source(reqwest::Client::new(), &source));
    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(outcome.row.status, "configured (authenticated)");
    assert_eq!(outcome.row.key_configured, Some(true));
}

#[test]
fn optional_auth_get_reports_unauthenticated_429_as_unavailable() {
    let _lock = env_lock();
    let _env = set_env_var("S2_API_KEY", None);
    let server = block_on(MockServer::start());
    let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
    let source = semantic_scholar_source(url);

    block_on(async {
        Mock::given(method("GET"))
            .and(path("/health"))
            .and(|request: &wiremock::Request| !request.headers.contains_key("x-api-key"))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&server)
            .await;
    });

    let outcome = block_on(probe_source(reqwest::Client::new(), &source));
    assert_eq!(outcome.class, ProbeClass::Healthy);
    assert_eq!(
        outcome.row.status,
        "unavailable (set S2_API_KEY for reliable access)"
    );
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
                status: "error".into(),
                latency: "timeout".into(),
                affects: Some("adverse-event search".into()),
                key_configured: None,
            },
            class: ProbeClass::Error,
        },
    ])
    .to_markdown();
    assert!(md.contains(&format!(
        "| Semantic Scholar | {} | {} | - |",
        outcome.row.status, outcome.row.latency
    )));
}

#[test]
fn optional_auth_get_reports_unauthenticated_non_429_as_error() {
    let _lock = env_lock();
    let _env = set_env_var("S2_API_KEY", None);
    let server = block_on(MockServer::start());
    let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
    let source = semantic_scholar_source(url);

    block_on(async {
        Mock::given(method("GET"))
            .and(path("/health"))
            .and(|request: &wiremock::Request| !request.headers.contains_key("x-api-key"))
            .respond_with(ResponseTemplate::new(403))
            .expect(1)
            .mount(&server)
            .await;
    });

    let outcome = block_on(probe_source(reqwest::Client::new(), &source));
    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, "error");
    assert!(outcome.row.latency.contains("HTTP 403"));
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some("Semantic Scholar features")
    );
    assert_eq!(outcome.row.key_configured, Some(false));
}

#[test]
fn optional_auth_get_reports_authenticated_429_as_error() {
    let _lock = env_lock();
    let _env = set_env_var("S2_API_KEY", Some("test-key-abc"));
    let server = block_on(MockServer::start());
    let url = Box::leak(format!("{}/health", server.uri()).into_boxed_str());
    let source = semantic_scholar_source(url);

    block_on(async {
        Mock::given(method("GET"))
            .and(path("/health"))
            .and(header("x-api-key", "test-key-abc"))
            .respond_with(ResponseTemplate::new(429))
            .expect(1)
            .mount(&server)
            .await;
    });

    let outcome = block_on(probe_source(reqwest::Client::new(), &source));
    assert_eq!(outcome.class, ProbeClass::Error);
    assert_eq!(outcome.row.status, "error");
    assert!(outcome.row.latency.contains("HTTP 429"));
    assert_eq!(
        outcome.row.affects.as_deref(),
        Some("Semantic Scholar features")
    );
    assert_eq!(outcome.row.key_configured, Some(true));
}
