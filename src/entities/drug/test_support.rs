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

#[cfg(test)]
mod brand_resolution;
mod label_choice;

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
async fn label_elements_oversize_response_settles_as_unavailable_with_reason() {
    let (base, server) = label_fallback_failure_server(ElementsFallbackAnswer::Oversize).await;

    // Ticket 2033 finding 8: an identity-field fallback response past
    // the body read limit can never succeed, and it is a provider-side
    // failure rather than a source-confirmed zero, so it settles as an
    // unavailable outcome that names the oversize reason.
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
