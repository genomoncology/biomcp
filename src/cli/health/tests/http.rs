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

mod orcid_row {
    use super::super::super::http::check_orcid_get;
    use super::super::super::runner::ProbeClass;
    use super::super::super::{HealthRow, HealthStatus};
    use std::io::{Read as _, Write};
    use std::net::TcpListener;

    /// Counts connections; serves one canned 200 response per connection.
    fn probe_server() -> (String, std::sync::Arc<std::sync::Mutex<usize>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind probe listener");
        let address = listener.local_addr().expect("probe address");
        let hits = std::sync::Arc::new(std::sync::Mutex::new(0usize));
        let counter = hits.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else {
                    break;
                };
                *counter.lock().unwrap() += 1;
                let mut buffer = [0_u8; 4096];
                let _ = stream.read(&mut buffer);
                let body = r#"{"path":"/0000-0002-1825-0097/person","name":{"visibility":"PUBLIC","given-names":{"value":"Josiah"},"family-name":{"value":"Carberry"}}}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/vnd.orcid+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        (format!("http://{address}/0000-0002-1825-0097/person"), hits)
    }

    struct TokenEnv {
        previous: Option<std::ffi::OsString>,
    }
    impl TokenEnv {
        fn set(value: Option<&str>) -> Self {
            let previous = std::env::var_os("ORCID_ACCESS_TOKEN");
            // SAFETY: serial-guarded test environment mutation.
            unsafe {
                match value {
                    Some(value) => std::env::set_var("ORCID_ACCESS_TOKEN", value),
                    None => std::env::remove_var("ORCID_ACCESS_TOKEN"),
                }
            }
            Self { previous }
        }
    }
    impl Drop for TokenEnv {
        fn drop(&mut self) {
            // SAFETY: restoring the serial-guarded environment.
            unsafe {
                match &self.previous {
                    Some(value) => std::env::set_var("ORCID_ACCESS_TOKEN", value),
                    None => std::env::remove_var("ORCID_ACCESS_TOKEN"),
                }
            }
        }
    }

    fn client() -> reqwest::Client {
        reqwest::Client::builder().build().expect("probe client")
    }

    fn row_json(row: &HealthRow) -> String {
        serde_json::to_string(row).expect("row json")
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn a_missing_token_excludes_the_row_without_any_request() {
        let _env = TokenEnv::set(None);
        let (url, hits) = probe_server();
        let outcome = check_orcid_get(
            client(),
            "ORCID",
            &url,
            "ORCID_ACCESS_TOKEN",
            Some("get author and author papers for ORCID IDs"),
        )
        .await;
        assert_eq!(outcome.class, ProbeClass::Excluded);
        assert_eq!(outcome.row.status, HealthStatus::Excluded);
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(outcome.row.key_configured, Some(false));
        assert_eq!(
            outcome.row.required_env_var.as_deref(),
            Some("ORCID_ACCESS_TOKEN")
        );
        assert_eq!(*hits.lock().unwrap(), 0, "zero GETs while excluded");
        let json = row_json(&outcome.row);
        assert_eq!(
            json,
            r#"{"api":"ORCID","status":"excluded","latency":"n/a","affects":"get author and author papers for ORCID IDs","key_configured":false,"required_env_var":"ORCID_ACCESS_TOKEN"}"#
        );
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn an_invalid_nonblank_token_is_an_error_row_without_any_request() {
        let _env = TokenEnv::set(Some("bad token"));
        let (url, hits) = probe_server();
        let outcome = check_orcid_get(
            client(),
            "ORCID",
            &url,
            "ORCID_ACCESS_TOKEN",
            Some("get author and author papers for ORCID IDs"),
        )
        .await;
        assert_eq!(outcome.class, ProbeClass::Error);
        assert_eq!(outcome.row.status, HealthStatus::Error);
        assert_eq!(outcome.row.latency, "n/a");
        assert_eq!(outcome.row.key_configured, Some(true));
        assert_eq!(*hits.lock().unwrap(), 0, "zero GETs for an invalid key");
        let json = row_json(&outcome.row);
        assert!(json.contains(r#""status":"error""#), "{json}");
        assert!(json.contains(r#""key_configured":true"#), "{json}");
    }

    #[tokio::test]
    #[serial_test::serial(source_env)]
    async fn a_configured_token_probes_the_public_person_endpoint() {
        let _env = TokenEnv::set(Some("fixture-public-read-token"));
        let (url, hits) = probe_server();
        let outcome = check_orcid_get(
            client(),
            "ORCID",
            &url,
            "ORCID_ACCESS_TOKEN",
            Some("get author and author papers for ORCID IDs"),
        )
        .await;
        assert_eq!(outcome.class, ProbeClass::Healthy);
        assert_eq!(outcome.row.status, HealthStatus::Ok);
        assert_eq!(outcome.row.key_configured, Some(true));
        assert_eq!(*hits.lock().unwrap(), 1, "exactly one non-retried GET");
    }
}
