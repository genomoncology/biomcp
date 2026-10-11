//! Tests for NCI CTS trial search helpers.

use super::super::validate_trial_search;
use super::*;
use crate::entities::trial::test_support::TrialSearchEnvRestore;
use crate::entities::trial::TrialSource;
use crate::sources::nci_cts::NciCtsClient;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn mydisease_hit(value: serde_json::Value) -> crate::sources::mydisease::MyDiseaseHit {
    serde_json::from_value(value).expect("valid MyDisease hit")
}

#[test]
fn nci_search_prefers_grounded_disease_concept_id() {
    let grounding = nci_disease_grounding_from_hit(
        "melanoma",
        mydisease_hit(serde_json::json!({
            "_id": "MONDO:0005105",
            "mondo": {
                "name": "Melanoma",
                "xrefs": {
                    "ncit": ["C3224"]
                }
            }
        })),
    );
    assert!(grounding.degrade_note.is_none());
    let plan = NciCtsClient::search_plan(
        "test-key",
        &NciSearchParams {
            disease: Some(grounding.filter),
            size: 1,
            from: 0,
            ..NciSearchParams::default()
        },
    );

    assert!(
        plan.query
            .contains(&("diseases.nci_thesaurus_concept_id".into(), "C3224".into()))
    );
    assert!(!plan.query.iter().any(|(key, _)| *key == "keyword"));
}

#[test]
fn nci_search_falls_back_to_keyword_when_grounding_is_unavailable() {
    let plan = NciCtsClient::search_plan(
        "test-key",
        &NciSearchParams {
            disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
            size: 1,
            from: 0,
            ..NciSearchParams::default()
        },
    );

    assert!(plan.query.contains(&("keyword".into(), "melanoma".into())));
    assert!(
        !plan
            .query
            .iter()
            .any(|(key, _)| *key == "diseases.nci_thesaurus_concept_id")
    );
}

#[test]
fn nci_search_falls_back_to_keyword_when_best_hit_lacks_nci_xref() {
    let grounding = nci_disease_grounding_from_hit(
        "melanoma",
        mydisease_hit(serde_json::json!({
            "_id": "MONDO:0005105",
            "mondo": {
                "name": "Melanoma"
            }
        })),
    );

    match grounding.filter {
        NciDiseaseFilter::Keyword(value) => assert_eq!(value, "melanoma"),
        other => panic!("expected keyword fallback, got {other:?}"),
    }
    let note = grounding
        .degrade_note
        .expect("missing xref degrades visibly");
    assert!(note.contains("keyword"), "{note}");
    assert!(note.contains("no NCI concept ID"), "{note}");
    assert!(note.contains("'melanoma'"), "{note}");
}

#[test]
fn nci_keyword_fallback_request_uses_keyword_not_concept_id() {
    let plan = NciCtsClient::search_plan(
        "test-key",
        &NciSearchParams {
            disease: Some(NciDiseaseFilter::Keyword("melanoma".into())),
            size: 1,
            from: 0,
            ..NciSearchParams::default()
        },
    );

    assert!(plan.query.contains(&("keyword".into(), "melanoma".into())));
    assert!(
        !plan
            .query
            .iter()
            .any(|(key, _)| *key == "diseases.nci_thesaurus_concept_id")
    );
}

#[test]
fn nci_status_mapping_uses_documented_single_value_filters() {
    let cases = [
        ("recruiting", "site", "ACTIVE"),
        ("not yet recruiting", "current", "Approved"),
        (
            "enrolling by invitation",
            "current",
            "Enrolling by Invitation",
        ),
        ("active, not recruiting", "site", "CLOSED_TO_ACCRUAL"),
        ("completed", "current", "Complete"),
        ("suspended", "current", "Temporarily Closed to Accrual"),
        ("terminated", "current", "Administratively Complete"),
        ("withdrawn", "current", "Withdrawn"),
    ];

    for &(input, expected_kind, expected_value) in &cases {
        let normalized = validate_trial_search(&TrialSearchFilters {
            source: TrialSource::NciCts,
            status: Some(input.into()),
            ..Default::default()
        })
        .expect("status should normalize");
        let filter = nci_status_filter(normalized.normalized_status.as_deref())
            .expect("status should map")
            .expect("status filter");
        match (expected_kind, filter) {
            ("current", NciStatusFilter::CurrentTrialStatus(value)) => {
                assert_eq!(value, expected_value);
            }
            ("site", NciStatusFilter::SiteRecruitmentStatus(value)) => {
                assert_eq!(value, expected_value);
            }
            (_, other) => panic!("unexpected status filter for {input}: {other:?}"),
        }
    }
}

#[test]
fn nci_source_rejects_status_lists() {
    let err = nci_status_filter(Some("RECRUITING,COMPLETED"))
        .expect_err("NCI should reject comma-separated status lists");
    assert!(err.to_string().contains("one mapped status at a time"));
    assert!(err.to_string().contains("--source nci"));
}

#[test]
fn nci_phase_mapping_uses_i_ii_for_combined_phase() {
    let cases = [
        ("1", vec!["I"]),
        ("2", vec!["II"]),
        ("3", vec!["III"]),
        ("4", vec!["IV"]),
        ("na", vec!["NA"]),
        ("1/2", vec!["I_II"]),
        ("2/3", vec!["II_III"]),
    ];

    for (input_phase, expected) in cases {
        let normalized = validate_trial_search(&TrialSearchFilters {
            source: TrialSource::NciCts,
            phase: Some(input_phase.into()),
            ..Default::default()
        })
        .expect("phase should normalize");
        assert_eq!(
            nci_phase_filters(normalized.normalized_phase.as_deref()).expect("phase should map"),
            expected
        );
    }
}

#[test]
fn recorded_provider_phase_output_round_trips_to_exact_nci_request() {
    let ctgov: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../testdata/sources/ctgov/search_keytruda_limit3_20260811.json"
    ))
    .expect("receipted CTGov response");
    let ctgov_study: crate::sources::clinicaltrials::CtGovStudy =
        serde_json::from_value(ctgov["studies"][0].clone()).expect("recorded CTGov study");
    let ctgov_phase = crate::transform::trial::from_ctgov_hit(&ctgov_study)
        .phase
        .expect("recorded CTGov phase");
    assert_eq!(ctgov_phase, "PHASE1/PHASE2");

    let nci: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../testdata/sources/nci_cts/search_melanoma_20260811.json"
    ))
    .expect("receipted NCI response");
    let nci_phase = crate::transform::trial::from_nci_hit(&nci["data"][0])
        .expect("recorded NCI hit")
        .phase
        .expect("recorded NCI phase");
    assert_eq!(nci_phase, "III");

    for (phase, expected) in [(ctgov_phase, "I_II"), (nci_phase, "III")] {
        let normalized = validate_trial_search(&TrialSearchFilters {
            source: TrialSource::NciCts,
            phase: Some(phase),
            ..Default::default()
        })
        .expect("emitted phase should pass public validation");
        let phases = nci_phase_filters(normalized.normalized_phase.as_deref())
            .expect("normalized phase should translate");
        let plan = NciCtsClient::search_plan(
            "test-key",
            &NciSearchParams {
                phases,
                size: 1,
                ..NciSearchParams::default()
            },
        );
        assert_eq!(plan.query_value("phase"), Some(expected));
    }
}

#[test]
fn nci_source_rejects_early_phase1() {
    let err = validate_trial_search(&TrialSearchFilters {
        source: TrialSource::NciCts,
        phase: Some("EARLY_PHASE1".into()),
        ..Default::default()
    })
    .err()
    .expect("NCI should reject early_phase1 during public validation");
    assert!(err.to_string().contains("early_phase1"));
    assert!(err.to_string().contains("--source nci"));
}

#[test]
fn nci_public_filter_table_is_explicit() {
    let mut cases = Vec::new();
    macro_rules! case {
        ($name:literal, $field:ident, $value:expr, $mapped:literal) => {{
            let mut filters = TrialSearchFilters {
                source: TrialSource::NciCts,
                ..Default::default()
            };
            filters.$field = $value;
            cases.push(($name, filters, $mapped));
        }};
    }
    case!("condition", condition, Some("melanoma".into()), true);
    case!("intervention", intervention, Some("drug".into()), true);
    case!("facility", facility, Some("clinic".into()), true);
    case!("status", status, Some("recruiting".into()), true);
    case!("phase", phase, Some("2".into()), true);
    case!("biomarker", biomarker, Some("BRAF".into()), true);
    case!("mutation", mutation, Some("V600E".into()), true);
    case!("criteria", criteria, Some("ECOG 0".into()), true);
    case!(
        "study type",
        study_type,
        Some("interventional".into()),
        false
    );
    case!("age", age, Some(67.0), false);
    case!("sex", sex, Some("female".into()), false);
    case!("sponsor", sponsor, Some("NCI".into()), false);
    case!("sponsor type", sponsor_type, Some("nih".into()), false);
    case!("date from", date_from, Some("2026-01-01".into()), false);
    case!("date to", date_to, Some("2026-01-01".into()), false);
    case!(
        "prior therapies",
        prior_therapies,
        Some("platinum".into()),
        false
    );
    case!("progression on", progression_on, Some("drug".into()), false);
    case!("line of therapy", line_of_therapy, Some("2L".into()), false);
    case!("results", results_available, true, false);
    let mut geo = TrialSearchFilters {
        source: TrialSource::NciCts,
        lat: Some(42.0),
        lon: Some(-71.0),
        distance: Some(50),
        ..Default::default()
    };
    cases.push(("complete geo", geo.clone(), true));
    geo.no_alias_expand = true;
    cases.push(("no alias expansion", geo, false));

    for (name, filters, mapped) in cases {
        assert_eq!(validate_trial_search(&filters).is_ok(), mapped, "{name}");
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_age_rejection_precedes_both_provider_requests() {
    async fn counter_server() -> (String, Arc<AtomicUsize>, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let count = Arc::new(AtomicUsize::new(0));
        let observed = count.clone();
        let task = tokio::spawn(async move {
            while listener.accept().await.is_ok() {
                observed.fetch_add(1, Ordering::SeqCst);
            }
        });
        (base, count, task)
    }
    struct Restore(Vec<(&'static str, Option<std::ffi::OsString>)>);
    impl Restore {
        fn set(&mut self, key: &'static str, value: &str) {
            self.0.push((key, std::env::var_os(key)));
            // SAFETY: this test holds the serial-test process-wide environment lock.
            unsafe { std::env::set_var(key, value) };
        }
    }
    impl Drop for Restore {
        fn drop(&mut self) {
            for (key, value) in self.0.drain(..).rev() {
                // SAFETY: this test holds the serial-test process-wide environment lock.
                unsafe {
                    if let Some(value) = value {
                        std::env::set_var(key, value)
                    } else {
                        std::env::remove_var(key)
                    }
                }
            }
        }
    }

    let (nci_base, nci_count, nci_server) = counter_server().await;
    let (disease_base, disease_count, disease_server) = counter_server().await;
    let mut restore = Restore(Vec::new());
    restore.set("BIOMCP_NCI_CTS_BASE", &nci_base);
    restore.set("BIOMCP_MYDISEASE_BASE", &disease_base);
    let error = super::super::search_page(
        &TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("melanoma".into()),
            age: Some(67.0),
            ..Default::default()
        },
        5,
        0,
        None,
    )
    .await
    .expect_err("NCI age must be rejected");
    tokio::task::yield_now().await;
    nci_server.abort();
    disease_server.abort();
    assert!(
        error
            .to_string()
            .contains("--age is only supported for --source ctgov")
    );
    assert_eq!(nci_count.load(Ordering::SeqCst), 0);
    assert_eq!(disease_count.load(Ordering::SeqCst), 0);
}

/// Ticket 2017: when MyDisease grounding fails, the NCI search still runs
/// as a keyword search, and the degrade reaches the response as a page
/// note (the 2021 pattern), not only a log line.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_keyword_degrade_note_reaches_the_search_page() {
    let (nci_base, nci_requests, nci_server) = json_server(|request| {
        request
            .contains("keyword=melanoma")
            .then(|| (200, r#"{"total":0,"data":[]}"#.to_string()))
    })
    .await;
    let (disease_base, _disease_requests, disease_server) = json_server(|_| {
        // 404 avoids the 5xx retry backoff; the query endpoint has no
        // route for a synthetic grounding failure.
        Some((
            404,
            r#"{"error":"synthetic grounding failure"}"#.to_string(),
        ))
    })
    .await;

    let mut restore = TrialSearchEnvRestore::new();
    // The NCI client constructor demands NCI_API_KEY even against a
    // fixture base (CI runs keyless), so pin a fixture key like the
    // trial get tests do; the fixture ignores it.
    restore.set("NCI_API_KEY", "fixture-key");
    restore.set("BIOMCP_NCI_CTS_BASE", &nci_base);
    restore.set("BIOMCP_MYDISEASE_BASE", &disease_base);

    let page = super::super::search_page(
        &TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("melanoma".into()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect("the degraded search still runs");
    nci_server.abort();
    disease_server.abort();
    drop(restore);

    let note = page
        .partial_note
        .expect("the degrade note reaches the page");
    assert!(note.contains("plain keyword search"), "{note}");
    assert!(note.contains("'melanoma'"), "{note}");
    assert!(note.contains("grounding failed"), "{note}");
    let requests = nci_requests
        .lock()
        .expect("lock degrade-note requests")
        .clone();
    assert!(
        requests
            .iter()
            .any(|line| line.contains("keyword=melanoma")),
        "the NCI request must stay a keyword search: {requests:?}"
    );
}

async fn json_server(
    respond: impl Fn(&str) -> Option<(u16, String)> + Send + Sync + 'static,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let respond = Arc::new(respond);
    let task = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let captured = captured.clone();
            let respond = respond.clone();
            tokio::spawn(async move {
                let mut request = vec![0_u8; 16 * 1024];
                let Ok(len) = stream.read(&mut request).await else {
                    return;
                };
                let request = String::from_utf8_lossy(&request[..len]).into_owned();
                let first_line = request.lines().next().unwrap_or("").to_string();
                captured
                    .lock()
                    .expect("lock fixture server requests")
                    .push(first_line.clone());
                let (status, body) = respond(&request)
                    .unwrap_or((404, r#"{"error":"fixture route not found"}"#.to_string()));
                let response = format!(
                    "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    (base, requests, task)
}

/// Ticket 2032: `--condition MF` grounds through the same resolver as
/// `get disease`. When the resolver refuses the abbreviation, the NCI
/// search returns the refusal with its named candidates and never sends
/// the keyword request a degrade would send.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_ambiguous_condition_refuses_instead_of_keyword_search() {
    let mf_hits =
        include_str!("../../../../../testdata/sources/mydisease/query_mf.json").to_string();
    let (disease_base, _disease_requests, disease_server) =
        json_server(move |request| request.contains("/query?").then(|| (200, mf_hits.clone())))
            .await;
    let (nci_base, nci_requests, nci_server) =
        json_server(|_| Some((200, r#"{"total":0,"data":[]}"#.to_string()))).await;

    let mut restore = TrialSearchEnvRestore::new();
    // The NCI client constructor demands NCI_API_KEY even against a
    // fixture base (CI runs keyless); the fixture ignores it.
    restore.set("NCI_API_KEY", "fixture-key");
    restore.set("BIOMCP_NCI_CTS_BASE", &nci_base);
    restore.set("BIOMCP_MYDISEASE_BASE", &disease_base);

    let error = super::super::search_page(
        &TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("MF".into()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect_err("the refusal must reach the caller");
    tokio::task::yield_now().await;
    nci_server.abort();
    disease_server.abort();
    drop(restore);

    let message = error.to_string();
    assert!(
        message.contains("Ambiguous disease abbreviation 'MF'"),
        "{message}"
    );
    assert!(
        message.contains("mycosis fungoides (MONDO:0009691)"),
        "{message}"
    );
    assert!(
        message.contains("myotonia fluctuans (MONDO:0020481)"),
        "{message}"
    );
    // Ticket 2040: the refusal also names the myelofibrosis reading the
    // source cannot see, while keeping the refusal itself intact.
    assert!(
        message.contains("'MF' also names myelofibrosis (MONDO:0009692)"),
        "{message}"
    );
    let requests = nci_requests
        .lock()
        .expect("lock NCI fixture requests")
        .clone();
    assert!(
        requests.is_empty(),
        "the refusal must send no NCI request: {requests:?}"
    );
}

/// Ticket 2032: a condition that fails to ground keeps the visible
/// keyword degrade (the 2021 pattern); only the resolver's refusal of
/// the input itself propagates.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_ungroundable_condition_keeps_the_keyword_degrade() {
    let (disease_base, _disease_requests, disease_server) = json_server(|request| {
        request
            .contains("/query?")
            .then(|| (200, r#"{"total":0,"hits":[]}"#.to_string()))
    })
    .await;
    let (nci_base, nci_requests, nci_server) =
        json_server(|_| Some((200, r#"{"total":0,"data":[]}"#.to_string()))).await;

    let mut restore = TrialSearchEnvRestore::new();
    restore.set("NCI_API_KEY", "fixture-key");
    restore.set("BIOMCP_NCI_CTS_BASE", &nci_base);
    restore.set("BIOMCP_MYDISEASE_BASE", &disease_base);
    // The discover fallback must stay off the network when the direct
    // resolution has nothing to hold.
    restore.set("BIOMCP_OLS4_BASE", "://unavailable-discover-fixture");
    restore.set("BIOMCP_UMLS_BASE", "://unavailable-discover-fixture");
    restore.set("BIOMCP_MEDLINEPLUS_BASE", "://unavailable-discover-fixture");

    let page = super::super::search_page(
        &TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some("ungroundable fixture condition".into()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect("the degraded search still runs");
    tokio::task::yield_now().await;
    nci_server.abort();
    disease_server.abort();
    drop(restore);

    let note = page
        .partial_note
        .expect("the degrade note reaches the page");
    assert!(note.contains("plain keyword search"), "{note}");
    assert!(
        note.contains("does not ground to a disease concept in MyDisease"),
        "{note}"
    );
    assert!(note.contains("'ungroundable fixture condition'"), "{note}");
    let requests = nci_requests
        .lock()
        .expect("lock NCI fixture requests")
        .clone();
    assert!(
        requests
            .iter()
            .any(|line| line.contains("keyword=ungroundable")),
        "the NCI request stays a keyword search: {requests:?}"
    );
}

/// Ticket 2040: a condition over 512 bytes used to fail the whole NCI
/// search with "Query is too long" before any provider request. It now
/// truncates to the longest prefix the disease lookup accepts, keeps
/// searching, and says so in the page note.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_overlong_condition_truncates_instead_of_failing() {
    let (disease_base, disease_requests, disease_server) = json_server(|request| {
        request
            .contains("/query?")
            .then(|| (200, r#"{"total":0,"hits":[]}"#.to_string()))
    })
    .await;
    let (nci_base, nci_requests, nci_server) =
        json_server(|_| Some((200, r#"{"total":0,"data":[]}"#.to_string()))).await;

    let mut restore = TrialSearchEnvRestore::new();
    restore.set("NCI_API_KEY", "fixture-key");
    restore.set("BIOMCP_NCI_CTS_BASE", &nci_base);
    restore.set("BIOMCP_MYDISEASE_BASE", &disease_base);
    restore.set("BIOMCP_OLS4_BASE", "://unavailable-discover-fixture");
    restore.set("BIOMCP_UMLS_BASE", "://unavailable-discover-fixture");
    restore.set("BIOMCP_MEDLINEPLUS_BASE", "://unavailable-discover-fixture");

    let condition = "m".repeat(600);
    let page = super::super::search_page(
        &TrialSearchFilters {
            source: TrialSource::NciCts,
            condition: Some(condition.clone()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect("the truncated search still runs");
    tokio::task::yield_now().await;
    nci_server.abort();
    disease_server.abort();
    drop(restore);

    let note = page
        .partial_note
        .expect("the truncation note reaches the page");
    assert!(
        note.contains("used at most the first 512 bytes of the condition"),
        "{note}"
    );
    assert!(note.contains("plain keyword search"), "{note}");
    let disease_requests = disease_requests
        .lock()
        .expect("lock MyDisease fixture requests")
        .clone();
    assert_eq!(
        disease_requests.len(),
        1,
        "the truncated condition grounds once: {disease_requests:?}"
    );
    assert!(
        disease_requests[0].contains(&"m".repeat(512)),
        "the grounding query carries the 512-byte prefix: {}",
        &disease_requests[0][..140]
    );
    let nci_requests = nci_requests
        .lock()
        .expect("lock NCI fixture requests")
        .clone();
    assert_eq!(nci_requests.len(), 1, "the NCI keyword search runs: {nci_requests:?}");
    assert!(
        nci_requests[0].contains(&format!("keyword={}", "m".repeat(512))),
        "the NCI keyword uses the truncated condition: {}",
        &nci_requests[0][..140]
    );
}

#[test]
fn nci_biomarker_like_fields_never_choose_or_duplicate_a_value() {
    for field in ["biomarker", "mutation", "criteria"] {
        let mut filters = TrialSearchFilters::default();
        match field {
            "biomarker" => filters.biomarker = Some("BRAF".into()),
            "mutation" => filters.mutation = Some("BRAF".into()),
            _ => filters.criteria = Some("BRAF".into()),
        }
        let value = nci_biomarker_value(&filters).unwrap();
        let plan = NciCtsClient::search_plan(
            "key",
            &NciSearchParams {
                biomarkers: value,
                ..Default::default()
            },
        );
        assert_eq!(
            plan.query
                .iter()
                .filter(|(key, _)| key == "biomarkers")
                .count(),
            1
        );
    }
    let filters = TrialSearchFilters {
        biomarker: Some("BRAF".into()),
        mutation: Some("V600E".into()),
        ..Default::default()
    };
    assert!(nci_biomarker_value(&filters).is_err());
}
