//! Shared test-only helpers and re-exports for nested drug module tests.

use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[allow(unused_imports)]
pub(super) use crate::entities::SearchPage;
#[allow(unused_imports)]
pub(super) use crate::error::BioMcpError;
#[allow(unused_imports)]
pub(super) use crate::sources::mychem::MyChemHit;

#[allow(unused_imports)]
pub(super) use super::{
    DrugRegion, DrugSearchFilters, DrugSearchResult, WhoPrequalificationEntry,
    WhoPrequalificationKind,
};

pub(super) fn mychem_row(name: &str) -> DrugSearchResult {
    DrugSearchResult {
        name: name.to_string(),
        drugbank_id: None,
        drug_type: None,
        mechanism: None,
        target: None,
    }
}

pub(super) fn who_row(reference: &str, inn: &str) -> WhoPrequalificationEntry {
    WhoPrequalificationEntry {
        kind: WhoPrequalificationKind::FinishedPharma,
        who_reference_number: Some(reference.to_string()),
        inn: inn.to_string(),
        presentation: Some(format!("{inn} Tablet 100mg")),
        dosage_form: Some("Tablet".to_string()),
        product_type: "Finished Pharmaceutical Product".to_string(),
        therapeutic_area: "Malaria".to_string(),
        applicant: "Example Applicant".to_string(),
        listing_basis: Some("Prequalification - Abridged".to_string()),
        alternative_listing_basis: None,
        prequalification_date: Some("2024-01-01".to_string()),
        who_product_id: None,
        grade: None,
        confirmation_document_date: None,
        vaccine_type: None,
        commercial_name: None,
        dose_count: None,
        manufacturer: None,
        responsible_nra: None,
    }
}

pub(super) fn who_api_row(product_id: &str, inn: &str) -> WhoPrequalificationEntry {
    WhoPrequalificationEntry {
        kind: WhoPrequalificationKind::Api,
        who_reference_number: None,
        inn: inn.to_string(),
        presentation: None,
        dosage_form: None,
        product_type: "Active Pharmaceutical Ingredient".to_string(),
        therapeutic_area: "Malaria".to_string(),
        applicant: "Example API Applicant".to_string(),
        listing_basis: None,
        alternative_listing_basis: None,
        prequalification_date: Some("2024-01-01".to_string()),
        who_product_id: Some(product_id.to_string()),
        grade: Some("Standard".to_string()),
        confirmation_document_date: Some("2024-02-01".to_string()),
        vaccine_type: None,
        commercial_name: None,
        dose_count: None,
        manufacturer: None,
        responsible_nra: None,
    }
}

struct RequiredLabelFixtureEnv(Vec<(&'static str, Option<std::ffi::OsString>)>);

impl RequiredLabelFixtureEnv {
    fn set(&mut self, name: &'static str, value: &str) {
        self.0.push((name, std::env::var_os(name)));
        // SAFETY: the test holds serial_test's process-wide environment lock.
        unsafe { std::env::set_var(name, value) };
    }
}

impl Drop for RequiredLabelFixtureEnv {
    fn drop(&mut self) {
        for (name, prior) in self.0.drain(..).rev() {
            // SAFETY: the test holds serial_test's process-wide environment lock.
            unsafe {
                if let Some(value) = prior {
                    std::env::set_var(name, value);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

async fn required_label_failure_server() -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind required-label fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]);
                let (status, body) = if request.starts_with("GET /v1/query?") {
                    (
                        "200 OK",
                        r#"{"total":1,"hits":[{"_id":"fixture-drug","_score":10.0,"drugbank":{"id":"DBFIXTURE","name":"fixture-drug","synonyms":[],"drug_interactions":[]}}]}"#,
                    )
                } else if request.starts_with("GET /drug/label.json?") {
                    ("400 Bad Request", r#"{"error":"private sentinel"}"#)
                } else {
                    ("404 Not Found", r#"{"error":"unplanned"}"#)
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response");
            });
        }
    });
    (base, task)
}

/// A fixture server whose field-scoped label lookup answers with no match
/// and whose full-text fallback answer the caller selects: a transport
/// failure status or a body past the read limit.
#[derive(Clone, Copy)]
enum FulltextFallbackAnswer {
    Error,
    Oversize,
}

async fn label_fulltext_failure_server(
    answer: FulltextFallbackAnswer,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind label fallback fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut request = vec![0_u8; 32 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]);
                let (status, body): (&str, Vec<u8>) = if request.starts_with("GET /v1/query?") {
                    (
                        "200 OK",
                        r#"{"total":1,"hits":[{"_id":"fixture-drug","_score":10.0,"drugbank":{"id":"DBFIXTURE","name":"fixture-drug","synonyms":[],"drug_interactions":[]}}]}"#.as_bytes().to_vec(),
                    )
                } else if request.starts_with("GET /drug/label.json?")
                    && request.contains("limit=100")
                {
                    match answer {
                        FulltextFallbackAnswer::Error => (
                            "500 Internal Server Error",
                            r#"{"error":"private sentinel"}"#.as_bytes().to_vec(),
                        ),
                        // One byte past the 8 MiB source-body read limit.
                        FulltextFallbackAnswer::Oversize => {
                            ("200 OK", vec![b'x'; 8 * 1024 * 1024 + 1])
                        }
                    }
                } else if request.starts_with("GET /drug/label.json?") {
                    (
                        "404 Not Found",
                        r#"{"error":{"code":"NOT_FOUND"}}"#.as_bytes().to_vec(),
                    )
                } else {
                    (
                        "404 Not Found",
                        r#"{"error":"unplanned"}"#.as_bytes().to_vec(),
                    )
                };
                let response = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .await
                    .expect("write fixture response head");
                stream
                    .write_all(&body)
                    .await
                    .or_else(|error| {
                        // The oversize answer ends with the client closing the
                        // connection once the read limit trips.
                        if error.kind() == std::io::ErrorKind::UnexpectedEof
                            || error.kind() == std::io::ErrorKind::BrokenPipe
                            || error.kind() == std::io::ErrorKind::ConnectionReset
                        {
                            Ok(())
                        } else {
                            Err(error)
                        }
                    })
                    .expect("write fixture response body");
            });
        }
    });
    (base, task)
}

async fn label_fallback_fixture_drug(base: &str, sections: &[&str]) -> super::Drug {
    let root = crate::test_support::TempDirGuard::new("label-fallback-ddinter-counter");
    let missing_ddinter = root.path().join("missing-ddinter");
    // Point the shared HTTP client's cache walk at the fixture's own tree:
    // walking the machine cache root can outlast the test (see the discover
    // fixture's note on client construction).
    let cache_root = crate::test_support::TempDirGuard::new("label-fallback-cache");
    let _cache_mode = crate::sources::test_cache_mode::off();
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.path().to_string_lossy().as_ref(),
    );
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", base);
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );
    let sections: Vec<String> = sections.iter().map(|value| value.to_string()).collect();
    super::get("fixture-drug", &sections)
        .await
        .expect("label fallback fixture settles a card")
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn label_fetch_failures_settle_as_unavailable_outcomes() {
    let (base, server) = required_label_failure_server().await;
    let root = crate::test_support::TempDirGuard::new("required-label-ddinter-counter");
    let missing_ddinter = root.path().join("missing-ddinter");
    // The guard bypasses every cache for the test without latching the
    // process mode the way the former `BIOMCP_CACHE_MODE=off` set did
    // (ticket 1261).
    let _cache_mode = crate::sources::test_cache_mode::off();
    // Point the shared HTTP client's cache walk at the fixture's own tree:
    // walking the machine cache root can outlast the test.
    let cache_root = crate::test_support::TempDirGuard::new("required-label-cache");
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.path().to_string_lossy().as_ref(),
    );
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", &base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );

    // Ticket 1300: a requested label section whose fetch failed settles as
    // an unavailable outcome instead of aborting the card, and the provider's
    // error detail never leaks into the settled result.
    for sections in [
        vec!["label".to_string(), "interactions".to_string()],
        vec!["all".to_string()],
    ] {
        let drug = super::get("fixture-drug", &sections)
            .await
            .unwrap_or_else(|error| panic!("label fetch failure must settle: {error}"));
        let outcome = drug
            .section_outcomes
            .get("label")
            .expect("label outcome completed");
        assert_eq!(
            outcome.outcome(),
            crate::entities::section_outcome::SectionOutcomeState::Unavailable
        );
        assert_eq!(
            outcome.message(),
            Some("OpenFDA label evidence is temporarily unavailable.")
        );
        assert!(drug.label.is_none(), "{sections:?}");
    }
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn label_fulltext_fetch_error_stays_an_error() {
    let (base, server) = label_fulltext_failure_server(FulltextFallbackAnswer::Error).await;

    // Ticket 1300 review: the field-scoped miss must fall through to the
    // full-text fallback, and a real full-text fetch error settles as an
    // unavailable outcome — never as a silent no-match empty.
    let drug = label_fallback_fixture_drug(&base, &["label"]).await;
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Unavailable
    );
    assert_eq!(
        outcome.message(),
        Some("OpenFDA label evidence is temporarily unavailable.")
    );
    assert!(drug.label.is_none());
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn label_fulltext_oversize_response_settles_as_no_match_with_reason() {
    let (base, server) = label_fulltext_failure_server(FulltextFallbackAnswer::Oversize).await;

    // Ticket 1300 review: a full-text fallback response past the body read
    // limit can never succeed on retry, so it settles as an empty outcome
    // that names the oversize reason instead of an unavailable retry hint.
    let drug = label_fallback_fixture_drug(&base, &["label"]).await;
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Empty
    );
    assert_eq!(
        outcome.message(),
        Some(
            "The openFDA full-text label search response was too large to read, so no label record could be confirmed."
        )
    );
    assert!(drug.label.is_none());
    server.abort();
}
