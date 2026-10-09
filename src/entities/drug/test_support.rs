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
/// and whose product-data-elements fallback answer the caller selects: a
/// transport failure status or a body past the read limit.
#[derive(Clone, Copy)]
enum ElementsFallbackAnswer {
    Error,
    Oversize,
}

async fn label_fallback_failure_server(
    answer: ElementsFallbackAnswer,
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
                    && request.contains("spl_product_data_elements")
                {
                    match answer {
                        ElementsFallbackAnswer::Error => (
                            "500 Internal Server Error",
                            r#"{"error":"private sentinel"}"#.as_bytes().to_vec(),
                        ),
                        // One byte past the 8 MiB source-body read limit.
                        ElementsFallbackAnswer::Oversize => {
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

async fn label_fallback_fixture_drug(base: &str, name: &str, sections: &[&str]) -> super::Drug {
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
    super::get(name, &sections)
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
async fn label_fallback_fetch_error_stays_an_error() {
    let (base, server) = label_fallback_failure_server(ElementsFallbackAnswer::Error).await;

    // Ticket 1300 review: the field-scoped miss must fall through to the
    // identity-field fallback, and a real fallback fetch error settles as an
    // unavailable outcome — never as a silent no-match empty.
    let drug = label_fallback_fixture_drug(&base, "fixture-drug", &["label"]).await;
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
async fn label_elements_oversize_response_settles_as_no_match_with_reason() {
    let (base, server) = label_fallback_failure_server(ElementsFallbackAnswer::Oversize).await;

    // Ticket 1300 second review: an identity-field fallback response past
    // the body read limit can never succeed on retry, so it settles as an
    // empty outcome that names the oversize reason instead of an unavailable
    // retry hint.
    let drug = label_fallback_fixture_drug(&base, "fixture-drug", &["label"]).await;
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
            "The openFDA label product-data-elements search response was too large to read, so no label record could be confirmed."
        )
    );
    assert!(drug.label.is_none());
    server.abort();
}

/// The withdrawn Propulsid record's product data elements, captured from
/// openFDA 2026-10-07 and shared with the identity-guard tests: three
/// per-strength entries whose inactive excipients differ, and no populated
/// `openfda` identity fields. Cisapride's only other label matches are other
/// drugs' mentions, so the unfielded full-text answer for cisapride is
/// 16.4 MB and unreadable.
const PROPULSID_ELEMENTS: &str = super::label::PROPULSID_ELEMENTS;

fn propulsid_label_body() -> String {
    format!(
        r#"{{"meta":{{"results":{{"skip":0,"limit":10,"total":1}}}},"results":[{{"set_id":"fdd8f491-28d6-49ae-9935-0224b8815e84","openfda":{{}},"spl_product_data_elements":["{PROPULSID_ELEMENTS}"],"indications_and_usage":["INDICATIONS AND USAGE PROPULSID (cisapride) is indicated for the symptomatic treatment of adult patients with nocturnal heartburn due to gastroesophageal reflux disease."]}},{{"set_id":"another-drug-mentioning-cisapride","openfda":{{"brand_name":["Otherdrug"],"generic_name":["otherdrug"]}},"spl_product_data_elements":["OTHERDRUG otherdrug OTHERDRUG OTHERDRUG STARCH"],"indications_and_usage":["OTHERDRUG interacts with cisapride."]}}]}}"#
    )
}

/// A fixture server whose drug resolves as cisapride while both label
/// lookups answer the caller's selection: the field-scoped search always
/// misses (the live Propulsid record carries no openfda identity fields) and
/// the product-data-elements fallback either returns the Propulsid record
/// alongside another drug's mention or answers with no match.
#[derive(Clone, Copy)]
enum ElementsFallbackMatch {
    Propulsid,
    NoMatch,
}

async fn label_elements_match_server(
    answer: ElementsFallbackMatch,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind label elements fixture");
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
                        r#"{"total":1,"hits":[{"_id":"fixture-cisapride","_score":10.0,"drugbank":{"id":"DBFIXTURE","name":"cisapride","synonyms":[],"drug_interactions":[]}}]}"#
                            .as_bytes()
                            .to_vec(),
                    )
                } else if request.starts_with("GET /drug/label.json?")
                    && request.contains("spl_product_data_elements")
                {
                    match answer {
                        ElementsFallbackMatch::Propulsid => {
                            ("200 OK", propulsid_label_body().into_bytes())
                        }
                        ElementsFallbackMatch::NoMatch => (
                            "404 Not Found",
                            br#"{"error":{"code":"NOT_FOUND","message":"No matches found!"}}"#
                                .to_vec(),
                        ),
                    }
                } else if request.starts_with("GET /drug/label.json?") {
                    (
                        "404 Not Found",
                        br#"{"error":{"code":"NOT_FOUND","message":"No matches found!"}}"#.to_vec(),
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
                    .expect("write fixture response body");
            });
        }
    });
    (base, task)
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn label_elements_fallback_returns_the_own_record_not_an_empty() {
    let (base, server) = label_elements_match_server(ElementsFallbackMatch::Propulsid).await;

    // Ticket 1300 second review: openFDA holds the withdrawn Propulsid
    // label, so the identity-field fallback must return it even though its
    // openfda fields are empty and another drug's mention ranks in the
    // answer — never settle as an empty outcome for a drug the source has.
    let drug = label_fallback_fixture_drug(&base, "cisapride", &["label"]).await;
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert_eq!(outcome.message(), None);
    let label = drug.label.as_ref().expect("Propulsid label present");
    assert!(
        !label.indication_summary.is_empty()
            || label
                .indications
                .as_deref()
                .is_some_and(|text| text.contains("cisapride")),
        "the Propulsid record's own indication text reached the card"
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn label_elements_fallback_no_match_settles_as_the_honest_empty() {
    let (base, server) = label_elements_match_server(ElementsFallbackMatch::NoMatch).await;

    // When the source truly holds no record — terfenadine has no openFDA
    // SPL record at all — both lookups answer with no match, so the empty
    // outcome must name the missing record rather than an oversize or
    // unavailable answer.
    let drug = label_fallback_fixture_drug(&base, "cisapride", &["label"]).await;
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
        Some("No openFDA SPL label record matched this drug.")
    );
    assert!(drug.label.is_none());
    server.abort();
}

/// Percent-decode one query-parameter value, so a fixture server can route on
/// the decoded MyChem `q` term (ticket 2031).
fn decoded_query_param(request_target: &str, key: &str) -> Option<String> {
    let query = request_target.split_once('?')?.1;
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=')?;
        if name != key {
            continue;
        }
        let mut out = String::with_capacity(value.len());
        let bytes = value.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b'%' if index + 2 < bytes.len() => {
                    let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).ok()?;
                    out.push(u8::from_str_radix(hex, 16).ok()? as char);
                    index += 3;
                }
                b'+' => {
                    out.push(' ');
                    index += 1;
                }
                byte => {
                    out.push(byte as char);
                    index += 1;
                }
            }
        }
        return Some(out);
    }
    None
}

/// A fixture server for the name-resolution flows (tickets 2031 and 2037):
/// MyChem answers by decoded query term, each openFDA label search answers
/// with the body the caller registered or a no-match 404, and the OLS4
/// discover search answers by decoded `q` term or a no-match 404.
async fn name_resolution_fixture_server(
    mychem: Vec<(String, String)>,
    label_searches: Vec<(String, String)>,
    ols_searches: Vec<(String, String)>,
) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind name-resolution fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mychem = mychem.clone();
            let label_searches = label_searches.clone();
            let ols_searches = ols_searches.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 64 * 1024];
                let len = stream
                    .read(&mut request)
                    .await
                    .expect("read fixture request");
                let request = String::from_utf8_lossy(&request[..len]);
                let target = request
                    .split_whitespace()
                    .nth(1)
                    .unwrap_or_default()
                    .to_string();
                let (status, body): (&str, Vec<u8>) = if target.starts_with("/v1/query?") {
                    let term = decoded_query_param(&target, "q").unwrap_or_default();
                    match mychem.iter().find(|(query, _)| *query == term) {
                        Some((_, body)) => ("200 OK", body.clone().into_bytes()),
                        None => (
                            "404 Not Found",
                            br#"{"error":{"code":"NOT_FOUND"}}"#.to_vec(),
                        ),
                    }
                } else if target.starts_with("/api/search?") {
                    let term = decoded_query_param(&target, "q").unwrap_or_default();
                    match ols_searches.iter().find(|(query, _)| *query == term) {
                        Some((_, body)) => ("200 OK", body.clone().into_bytes()),
                        None => (
                            "404 Not Found",
                            br#"{"error":{"code":"NOT_FOUND"}}"#.to_vec(),
                        ),
                    }
                } else if target.starts_with("/drug/label.json?") {
                    let term = decoded_query_param(&target, "search").unwrap_or_default();
                    let term = term
                        .replace("openfda.generic_name:", "")
                        .replace("openfda.brand_name:", "");
                    match label_searches
                        .iter()
                        .find(|(query, _)| term.contains(query))
                    {
                        Some((_, body)) => ("200 OK", body.clone().into_bytes()),
                        None => (
                            "404 Not Found",
                            br#"{"error":{"code":"NOT_FOUND","message":"No matches found!"}}"#
                                .to_vec(),
                        ),
                    }
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
                    .expect("write fixture response body");
            });
        }
    });
    (base, task)
}

async fn name_resolution_fixture_drug(base: &str, name: &str) -> super::Drug {
    let root = crate::test_support::TempDirGuard::new("name-resolution-ddinter");
    let missing_ddinter = root.path().join("missing-ddinter");
    // Point the shared HTTP client's cache walk at the fixture's own tree
    // (see the discover fixture's note on client construction).
    let cache_root = crate::test_support::TempDirGuard::new("name-resolution-cache");
    let _cache_mode = crate::sources::test_cache_mode::off();
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.path().to_string_lossy().as_ref(),
    );
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", base);
    // The guarded discover rescue reaches OLS4 and the HPO alias lookup it
    // carries; both stay on the fixture so no name-resolution test leaves
    // it (ticket 2037).
    env.set("BIOMCP_OLS4_BASE", base);
    env.set("BIOMCP_HPO_BASE", &format!("{base}/hp"));
    env.set("BIOMCP_UMLS_BASE", &format!("{base}/umls"));
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", base);
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );
    super::get(name, &["label".to_string()])
        .await
        .expect("name-resolution fixture settles a card")
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn terfenadine_returns_terfenadines_card_with_an_honest_empty_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "terfenadine".to_string(),
            crate::transform::drug::name_resolution_tests::TERFENADINE_CAPTURE.to_string(),
        )],
        Vec::new(),
        Vec::new(),
    )
    .await;

    // Ticket 2031: MyChem's text query also returns fexofenadine through its
    // "Terfenadine carboxylate" synonyms, but only terfenadine's own record
    // (DB00342) may resolve the card; openFDA holds no terfenadine SPL
    // record, so the label settles as the honest empty.
    let drug = name_resolution_fixture_drug(&base, "terfenadine").await;
    assert_eq!(drug.name, "terfenadine");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00342"));
    assert!(
        !serde_json::to_string(&drug)
            .expect("card serializes")
            .to_ascii_lowercase()
            .contains("fexofenadine")
    );
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
        Some("No openFDA SPL label record matched this drug.")
    );
    assert!(drug.label.is_none());
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_text_only_match_refuses_and_names_what_matched() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "ferric oxide".to_string(),
            crate::transform::drug::name_resolution_tests::FERRIC_OXIDE_TEXT_ONLY_CAPTURE
                .to_string(),
        )],
        Vec::new(),
        Vec::new(),
    )
    .await;

    let root = crate::test_support::TempDirGuard::new("refusal-ddinter");
    let missing_ddinter = root.path().join("missing-ddinter");
    let cache_root = crate::test_support::TempDirGuard::new("refusal-cache");
    let _cache_mode = crate::sources::test_cache_mode::off();
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.path().to_string_lossy().as_ref(),
    );
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", &base);
    env.set("BIOMCP_OLS4_BASE", &base);
    env.set("BIOMCP_HPO_BASE", &format!("{base}/hp"));
    env.set("BIOMCP_UMLS_BASE", &format!("{base}/umls"));
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );

    // Ticket 2031: when no record names the query and openFDA's identity
    // fields cannot resolve it either, the lookup refuses and says which
    // other drugs the text search matched.
    let err = super::get("ferric oxide", &["label".to_string()])
        .await
        .expect_err("a text-only match must refuse");
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"ferric oxide\""),
        "{message}"
    );
    assert!(
        message.contains("zinc oxide, titanium dioxide"),
        "{message}"
    );
    assert!(
        message.contains("calamine and pramoxine hydrochloride"),
        "{message}"
    );
    assert!(
        message.contains("biomcp search drug -q \"ferric oxide\""),
        "{message}"
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_brand_query_resolves_through_the_mychem_record_itself() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "TAGRISSO".to_string(),
            crate::transform::drug::name_resolution_tests::TAGRISSO_CAPTURE.to_string(),
        )],
        Vec::new(),
        Vec::new(),
    )
    .await;

    // Ticket 2031, fourth-review finding 1: live openFDA returns NOT_FOUND
    // for `openfda.brand_name:"TAGRISSO"` (the label has no openFDA block),
    // so the fixture answers every label search with no match — the brand
    // must resolve through the drugcentral synonyms and NDC proprietary
    // names on the osimertinib record itself and land on the generic
    // identity.
    let drug = name_resolution_fixture_drug(&base, "TAGRISSO").await;
    assert_eq!(drug.name, "osimertinib");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB09330"));
    server.abort();
}

/// The live OLS4 answer for `q=Tarceva`, recorded 2026-10-09 (ticket 2037)
/// and trimmed to the fields the discover pipeline reads, with descriptions
/// shortened and each doc's synonym list cut to its Tarceva-bearing entries.
/// NCIT:C2693 "Erlotinib Hydrochloride" is the only exact synonym match and
/// names the drug the guarded rescue adopts.
const OLS_TARCEVA_BODY: &str = r#"{
 "response": {
  "docs": [
   {
    "iri": "http://purl.obolibrary.org/obo/DRON_00085777",
    "ontology_name": "dron",
    "ontology_prefix": "DRON",
    "short_form": "DRON_00085777",
    "obo_id": "DRON:00085777",
    "label": "erlotinib 100 MG Oral Tablet [Tarceva]",
    "description": [],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/DRON_00085778",
    "ontology_name": "dron",
    "ontology_prefix": "DRON",
    "short_form": "DRON_00085778",
    "obo_id": "DRON:00085778",
    "label": "erlotinib 150 MG Oral Tablet [Tarceva]",
    "description": [],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/DRON_00085779",
    "ontology_name": "dron",
    "ontology_prefix": "DRON",
    "short_form": "DRON_00085779",
    "obo_id": "DRON:00085779",
    "label": "erlotinib 25 MG Oral Tablet [Tarceva]",
    "description": [],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C37557",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C37557",
    "obo_id": "NCIT:C37557",
    "label": "Bevacizumab/Erlotinib Regimen",
    "description": [
     "A regimen consisting of bevacizumab and erlotinib that may be used in the treatment of epidermal growth factor receptor  \u2026"
    ],
    "exact_synonyms": [
     "Avastin-Tarceva",
     "Avastin/Tarceva",
     "Tarceva/Avastin"
    ],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C63503",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C63503",
    "obo_id": "NCIT:C63503",
    "label": "Erlotinib/Gemcitabine Regimen",
    "description": [
     "A regimen consisting of gemcitabine and erlotinib used for the treatment of pancreatic cancer."
    ],
    "exact_synonyms": [
     "Gemcitabine-Tarceva Regimen",
     "gemcitabine-Tarceva regimen"
    ],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/CHEBI_53509",
    "ontology_name": "chebi",
    "ontology_prefix": "CHEBI",
    "short_form": "CHEBI_53509",
    "obo_id": "CHEBI:53509",
    "label": "erlotinib hydrochloride",
    "description": [
     "The hydrochloride salt of erlotinib."
    ],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://id.nlm.nih.gov/mesh/D000069347",
    "ontology_name": "mesh",
    "ontology_prefix": "mesh",
    "short_form": "mesh_D000069347",
    "obo_id": "mesh:D000069347",
    "label": "Erlotinib Hydrochloride",
    "description": [
     "A quinazoline derivative and ANTINEOPLASTIC AGENT that functions as a PROTEIN KINASE INHIBITOR for EGFR associated tyros \u2026"
    ],
    "exact_synonyms": [],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C160031",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C160031",
    "obo_id": "NCIT:C160031",
    "label": "Erlotinib Regimen",
    "description": [
     "A regimen consisting of erlotinib that may be used in the treatment of soft tissue sarcoma, kidney, vulvar and bone canc \u2026"
    ],
    "exact_synonyms": [
     "Tarceva Regimen"
    ],
    "type": "class"
   },
   {
    "iri": "http://purl.obolibrary.org/obo/NCIT_C2693",
    "ontology_name": "ncit",
    "ontology_prefix": "NCIT",
    "short_form": "NCIT_C2693",
    "obo_id": "NCIT:C2693",
    "label": "Erlotinib Hydrochloride",
    "description": [
     "The hydrochloride salt of a quinazoline derivative with antineoplastic properties.  Competing with adenosine triphosphat \u2026"
    ],
    "exact_synonyms": [
     "Tarceva"
    ],
    "type": "class"
   }
  ],
  "numFound": 9,
  "start": 0,
  "maxScore": 1.0
 }
}"#;

/// The live openFDA label answer for the erlotinib hydrochloride search,
/// recorded 2026-10-09 (ticket 2037): the newest erlotinib record, with the
/// section text trimmed for the fixture.
const ERLOTINIB_LABEL_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 5,
   "total": 1
  }
 },
 "results": [
  {
   "set_id": "ab6f3cb3-34a8-4492-a5d7-fb6b055a2d6b",
   "effective_time": "20240610",
   "openfda": {
    "brand_name": [
     "ERLOTINIB HYDROCHLORIDE"
    ],
    "generic_name": [
     "ERLOTINIB HYDROCHLORIDE"
    ],
    "route": [
     "ORAL"
    ],
    "substance_name": [
     "ERLOTINIB HYDROCHLORIDE"
    ]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE Erlotinib tablets are a kinase inhibitor indicated for: The treatment of patients with metastatic non-small cell lung cancer (NSCLC) whose tumors have epidermal growth factor receptor (EGFR) exon 19 deletions or exon 21 (L858R) substitution mutations as detected by an FDA-approved test receiving first-line, maintenance, or second or greater line treatment after progression following at least one prior chemotherapy regimen. (1.1) First-line treatment of patients with locally advanced, unresectable or metastatic pancreatic cancer, in combination with gemcitabine. (1.2) Limitations of Use: Safety and efficacy of erlotinib tablets has not been established in patients with \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Interstitial lung disease (ILD): Occurs in 1.1% of patients. Withhold erlotinib for acute onset of new or progressive unexplained pulmonary symptoms, such as dyspnea, cough and fever. Discontinue erlotinib if ILD is diagnosed. (5.1) Renal failure: Monitor renal function and electrolytes, particularly in patients at risk of dehydration. Withhold erlotinib for severe renal toxicity. (5.2) Hepatotoxicity: Occurs with or without hepatic impairment, including hepatic failure and hepatorenal syndrome: Monitor periodic liver testing. Withhold or discontinue erlotinib for severe or worsening liver tests. (5.3) Gastrointestinal perforations: Discontinue erlotinib. (5.4) Bul \u2026[recorded reply trimmed for the fixture]"
   ],
   "dosage_and_administration": [
    "2 DOSAGE AND ADMINISTRATION NSCLC: 150 mg orally, on an empty stomach, once daily. (2.2) Pancreatic cancer: 100 mg orally, on an empty stomach, once daily. (2.3) 2.1 Selection of Patients with Metastatic NSCLC Select patients for the treatment of metastatic NSCLC with erlotinib tablets based on the presence of EGFR exon 19 deletions or exon 21 (L858R) substitution mutations in tumor or plasma specimens [See Clinical Studies (14.1, 14.2)]. If these mutations are not detected in a plasma specimen, test tumor tissue if available. Information on FDA-approved tests for the detection of EGFR mutations in NSCLC is available at: http://www.fda.gov/CompanionDiagnostics . 2.2 Recommended Dose \u2013 NSCLC  \u2026[recorded reply trimmed for the fixture]"
   ],
   "drug_interactions": [
    "7 DRUG INTERACTIONS CYP3A4 Inhibitors Co-administration of erlotinib with a strong CYP3A4 inhibitor or a combined CYP3A4 and CYP1A2 inhibitor increased erlotinib exposure. Erlotinib is metabolized primarily by CYP3A4 and to a lesser extent by CYP1A2. Increased erlotinib exposure may increase the risk of exposure-related toxicity [see Clinical Pharmacology (12.3)] . Avoid co-administering erlotinib with strong CYP3A4 inhibitors (e.g., boceprevir, clarithromycin, conivaptan, indinavir, itraconazole, ketoconazole, lopinavir/ritonavir, nefazodone, nelfinavir, posaconazole, ritonavir, saquinavir, telithromycin, voriconazole, grapefruit or grapefruit juice) or a combined CYP3A4 and CYP1A2 inhibito \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

/// The live openFDA label answer for the niraparib search, recorded
/// 2026-10-09 (ticket 2037): Zejula's own record first and the Akeega
/// combination second, with the section text trimmed for the fixture.
const NIRAPARIB_LABEL_BODY: &str = r#"{
 "meta": {
  "results": {
   "skip": 0,
   "limit": 5,
   "total": 2
  }
 },
 "results": [
  {
   "set_id": "b7f675e2-159c-490c-b6f4-3f16d9492b7d",
   "effective_time": "20260728",
   "openfda": {
    "brand_name": [
     "ZEJULA"
    ],
    "generic_name": [
     "NIRAPARIB"
    ],
    "route": [
     "ORAL"
    ],
    "substance_name": [
     "NIRAPARIB TOSYLATE"
    ]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE ZEJULA is a poly (ADP-ribose) polymerase (PARP) inhibitor indicated: \u2022 for the maintenance treatment of adult patients with advanced epithelial ovarian, fallopian tube, or primary peritoneal cancer who are in a complete or partial response to first-line platinum-based chemotherapy and whose cancer is associated with homologous recombination deficiency (HRD)-positive status defined by either: o a deleterious or suspected deleterious BRCA mutation, and/or o genomic instability. Select patients for therapy based on an FDA\u2011authorized companion diagnostic for ZEJULA. ( 1.1 , 2.1 ) \u2022 for the maintenance treatment of adult patients with deleterious or suspected deleterious g \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS \u2022 Myelodysplastic Syndrome/Acute Myeloid Leukemia (MDS/AML): MDS/AML occurred in patients exposed to ZEJULA, and some cases were fatal. Monitor patients for hematological toxicity and discontinue if MDS/AML is confirmed. ( 5.1 ) \u2022 Bone Marrow Suppression: Test complete blood counts weekly for the first month, monthly for the next 11 months, and periodically thereafter for clinically significant changes. ( 5.2 ) \u2022 Hypertension and Cardiovascular Effects: Monitor blood pressure and heart rate at least weekly for the first 2 months, then monthly for the first year and periodically thereafter during treatment with ZEJULA. Manage with antihypertensive medications and ad \u2026[recorded reply trimmed for the fixture]"
   ],
   "dosage_and_administration": [
    "2 DOSAGE AND ADMINISTRATION \u2022 First \u2011 Line Maintenance Treatment of HRD \u2011 Positive Advanced Ovarian Cancer: o For patients weighing <77 kg (<170 lbs) OR with a platelet count <150,000/mcL, the recommended dosage is 200 mg taken orally once daily. ( 2.2 ) o For patients weighing \u226577 kg (\u2265170 lbs) AND a platelet count \u2265150,000/mcL, the recommended dosage is 300 mg taken orally once daily. ( 2.2 ) \u2022 Maintenance Treatment of Recurrent Germline BRCA \u2011 Mutated Ovarian Cancer: The recommended dosage is 300 mg taken orally once daily. ( 2.2 ) \u2022 Continue treatment until disease progression or unacceptable toxicity. ( 2.2 ) \u2022 ZEJULA may be taken with or without food. ( 2.2 ) \u2022 For adverse reactions, c \u2026[recorded reply trimmed for the fixture]"
   ]
  },
  {
   "set_id": "8245a990-3268-4613-b5c5-9537858a1eb9",
   "effective_time": "20251218",
   "openfda": {
    "brand_name": [
     "AKEEGA"
    ],
    "generic_name": [
     "NIRAPARIB TOSYLATE MONOHYDRATE AND ABIRATERONE ACETATE"
    ],
    "route": [
     "ORAL"
    ],
    "substance_name": [
     "NIRAPARIB TOSYLATE MONOHYDRATE",
     "ABIRATERONE ACETATE"
    ]
   },
   "indications_and_usage": [
    "1 INDICATIONS AND USAGE AKEEGA with prednisone is indicated for the treatment of adult patients with deleterious or suspected deleterious BRCA2 -mutated ( BRCA2 m) metastatic castration-sensitive prostate cancer (mCSPC). AKEEGA with prednisone is indicated for the treatment of adult patients with deleterious or suspected deleterious BRCA -mutated ( BRCA m) metastatic castration-resistant prostate cancer (mCRPC). Select patients for therapy based on an FDA-approved test for AKEEGA [see Dosage and Administration (2.1) ] . AKEEGA is a combination of niraparib, a poly (ADP-ribose) polymerase (PARP) inhibitor, and abiraterone acetate, a CYP17 inhibitor indicated with prednisone for the treatment  \u2026[recorded reply trimmed for the fixture]"
   ],
   "warnings_and_cautions": [
    "5 WARNINGS AND PRECAUTIONS Myelodysplastic Syndrome/Acute Myeloid Leukemia (MDS/AML) : MDS/AML, including a case with fatal outcome, has been observed in patients treated with AKEEGA. Monitor patients for hematological toxicity and discontinue if MDS/AML is confirmed. ( 5.1 ) Myelosuppression: Test complete blood counts weekly for the first month, every two weeks for the next two months, monthly for the remainder of the first year, then every other month, and as clinically indicated. ( 2.3 , 5.2 ) Hypokalemia, Fluid Retention, and Cardiovascular Adverse Reactions: Monitor patients for hypertension, hypokalemia, and fluid retention at least weekly for the first two months, then once a month.  \u2026[recorded reply trimmed for the fixture]"
   ],
   "dosage_and_administration": [
    "2 DOSAGE AND ADMINISTRATION BRCA2 m mCSPC: The recommended dosage of AKEEGA is 200 mg niraparib/1,000 mg abiraterone acetate orally once daily in combination with 5 mg prednisone daily until disease progression or unacceptable toxicity. ( 2.2 ) BRCA m mCRPC : The recommended dosage of AKEEGA is 200 mg niraparib/1,000 mg abiraterone acetate orally once daily in combination with 10 mg prednisone daily until disease progression or unacceptable toxicity. ( 2.2 ) Patients receiving AKEEGA should also receive a gonadotropin-releasing hormone (GnRH) analog concurrently or should have had bilateral orchiectomy. ( 2.2 ) Take AKEEGA on an empty stomach at least one hour before or two hours after food. \u2026[recorded reply trimmed for the fixture]"
   ],
   "drug_interactions": [
    "7 DRUG INTERACTIONS Strong CYP3A4 Inducers: Avoid coadministration. ( 7.1 ) CYP2D6 Substrates: Avoid coadministration of AKEEGA with CYP2D6 substrates for which minimal changes in concentration may lead to serious toxicities. If alternative treatments cannot be used, consider a dose reduction of the concomitant CYP2D6 substrate. ( 7.2 ) 7.1 Effect of Other Drugs on AKEEGA Effect of CYP3A4 Inducers Avoid coadministration with strong CYP3A4 inducers [see Clinical Pharmacology (12.3) ] . Abiraterone is a substrate of CYP3A4. Strong CYP3A4 inducers may decrease abiraterone concentrations [see Clinical Pharmacology (12.3) ], which may reduce the effectiveness of abiraterone. 7.2 Effects of AKEEGA \u2026[recorded reply trimmed for the fixture]"
   ]
  }
 ]
}"#;

#[tokio::test]
#[serial_test::serial(source_env)]
async fn tarceva_resolves_erlotinib_through_the_guarded_discover_rescue() {
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Tarceva".to_string(),
                crate::transform::drug::name_resolution_tests::TARCEVA_CAPTURE.to_string(),
            ),
            (
                "Erlotinib Hydrochloride".to_string(),
                crate::transform::drug::name_resolution_tests::ERLOTINIB_HYDROCHLORIDE_CAPTURE
                    .to_string(),
            ),
        ],
        vec![(
            "erlotinib hydrochloride".to_string(),
            ERLOTINIB_LABEL_BODY.to_string(),
        )],
        vec![("Tarceva".to_string(), OLS_TARCEVA_BODY.to_string())],
    )
    .await;

    // Ticket 2037: MyChem holds Tarceva only on a record with no name and
    // openFDA holds no Tarceva label, so the guarded discover rescue must
    // run before the name-miss refusal. The rescue resolves the brand to
    // erlotinib hydrochloride and the card carries the erlotinib label.
    let drug = name_resolution_fixture_drug(&base, "Tarceva").await;
    assert_eq!(drug.name, "erlotinib hydrochloride");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB00530"));
    assert_eq!(
        drug.label_set_id.as_deref(),
        Some("ab6f3cb3-34a8-4492-a5d7-fb6b055a2d6b")
    );
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert!(
        drug.label.as_ref().is_some_and(|label| {
            !label.indication_summary.is_empty()
                || label
                    .indications
                    .as_deref()
                    .is_some_and(|text| text.contains("erlotinib"))
        }),
        "the erlotinib label text reached the card"
    );
    server.abort();
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn zejula_returns_niraparib_with_the_zejula_label() {
    let (base, server) = name_resolution_fixture_server(
        vec![(
            "Zejula".to_string(),
            crate::transform::drug::name_resolution_tests::ZEJULA_CAPTURE.to_string(),
        )],
        vec![("niraparib".to_string(), NIRAPARIB_LABEL_BODY.to_string())],
        Vec::new(),
    )
    .await;

    // Ticket 2037: the Zejula card takes the name its own product row pairs
    // (niraparib), never the Akeega combination row that sits first, and
    // the label is Zejula's own record.
    let drug = name_resolution_fixture_drug(&base, "Zejula").await;
    assert_eq!(drug.name, "niraparib");
    assert_eq!(drug.drugbank_id.as_deref(), Some("DB11793"));
    assert_eq!(
        drug.label_set_id.as_deref(),
        Some("b7f675e2-159c-490c-b6f4-3f16d9492b7d")
    );
    assert!(
        !drug.name.contains("abiraterone"),
        "the card name never takes the Akeega combination row"
    );
    let outcome = drug
        .section_outcomes
        .get("label")
        .expect("label outcome completed");
    assert_eq!(
        outcome.outcome(),
        crate::entities::section_outcome::SectionOutcomeState::Data
    );
    assert!(
        drug.label.as_ref().is_some_and(|label| label
            .indication_summary
            .iter()
            .any(|row| row.name.contains("ovarian")))
            || drug.label.as_ref().is_some_and(|label| label
                .indications
                .as_deref()
                .is_some_and(|text| text.contains("ZEJULA"))),
        "the Zejula label text reached the card"
    );
    server.abort();
}

/// Two exact canonical drugs answering one brand: the discover rescue must
/// refuse to pick between them, so `get` keeps the honest no-match refusal.
/// This pin bites when the rescue guard's competing-exact check is removed
/// (ticket 2037).
#[tokio::test]
#[serial_test::serial(source_env)]
async fn a_brand_two_exact_drugs_answer_refuses_instead_of_picking_one() {
    let ambiguous_ols = r#"{"response":{"docs":[
        {"iri":"http://purl.obolibrary.org/obo/CHEBI_999991","ontology_prefix":"CHEBI","obo_id":"CHEBI:999991","label":"Fixture Drug Aaa","exact_synonyms":["Ambifton"]},
        {"iri":"http://purl.obolibrary.org/obo/CHEBI_999992","ontology_prefix":"CHEBI","obo_id":"CHEBI:999992","label":"Fixture Drug Bbb","exact_synonyms":["Ambifton"]}
    ]}}"#;
    let (base, server) = name_resolution_fixture_server(
        vec![
            (
                "Ambifton".to_string(),
                r#"{"total":1,"hits":[{"_id":"C9999999","_score":17.668518}]}"#.to_string(),
            ),
            (
                "Fixture Drug Aaa".to_string(),
                r#"{"total":1,"hits":[{"_id":"aaa","_score":10.0,"drugbank":{"id":"DBAAA","name":"Fixture Drug Aaa"}}]}"#
                    .to_string(),
            ),
        ],
        Vec::new(),
        vec![("Ambifton".to_string(), ambiguous_ols.to_string())],
    )
    .await;

    let root = crate::test_support::TempDirGuard::new("ambifton-ddinter");
    let missing_ddinter = root.path().join("missing-ddinter");
    let cache_root = crate::test_support::TempDirGuard::new("ambifton-cache");
    let _cache_mode = crate::sources::test_cache_mode::off();
    let mut env = RequiredLabelFixtureEnv(Vec::new());
    env.set(
        "BIOMCP_CACHE_DIR",
        cache_root.path().to_string_lossy().as_ref(),
    );
    env.set("BIOMCP_MYCHEM_BASE", &format!("{base}/v1"));
    env.set("BIOMCP_OPENFDA_BASE", &base);
    env.set("BIOMCP_OLS4_BASE", &base);
    env.set("BIOMCP_HPO_BASE", &format!("{base}/hp"));
    env.set("BIOMCP_UMLS_BASE", &format!("{base}/umls"));
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &base);
    env.set(
        "BIOMCP_DDINTER_DIR",
        missing_ddinter.to_str().expect("UTF-8 fixture path"),
    );

    // With the guard intact the two exact canonical drugs compete, the
    // rescue declines, and the refusal names the miss. Unguarded, the
    // rescue would adopt Fixture Drug Aaa and return its card instead.
    let err = super::get("Ambifton", &["label".to_string()])
        .await
        .expect_err("two competing exact drugs must refuse");
    let message = err.to_string();
    assert!(
        message.contains("No drug card matches \"Ambifton\""),
        "{message}"
    );
    server.abort();
}
