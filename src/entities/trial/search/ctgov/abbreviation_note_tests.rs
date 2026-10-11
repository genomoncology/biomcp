//! Tests for the default source's ambiguous-abbreviation note (ticket 2040).

use super::super::super::test_support::*;
use super::*;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

async fn mydisease_json_fixture(
    respond: impl Fn(&str) -> Option<String> + Send + Sync + 'static,
) -> (String, Arc<Mutex<Vec<String>>>, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind synthetic MyDisease fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
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
                captured
                    .lock()
                    .expect("lock MyDisease fixture requests")
                    .push(request.lines().next().unwrap_or("").to_string());
                let body =
                    respond(&request).unwrap_or_else(|| r#"{"total":0,"hits":[]}"#.to_string());
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes()).await;
            });
        }
    });
    (base, requests, task)
}

/// Ticket 2040: the default source has no disease grounding, so
/// `search trial -c MF` keeps running the registry's keyword search —
/// but the page says so plainly, naming the holders `get disease` would
/// refuse with and the myelofibrosis reading the source cannot see. The
/// registry request itself stays the raw keyword search.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn default_source_names_the_ambiguous_abbreviation_keyword_search() {
    let mf_hits =
        include_str!("../../../../../testdata/sources/mydisease/query_mf.json").to_string();
    let (disease_base, disease_requests, disease_server) =
        mydisease_json_fixture(move |request| request.contains("/query?").then(|| mf_hits.clone()))
            .await;
    let ctgov_body = r#"{"totalCount":1,"studies":[{
        "protocolSection": {
            "identificationModule": {"nctId": "NCT00000010", "briefTitle": "Fixture MF trial"},
            "statusModule": {"overallStatus": "RECRUITING"}
        }
    }]}"#
        .to_string();
    let (ctgov_base, ctgov_requests, ctgov_server) = ctgov_json_fixture(ctgov_body).await;

    let mut env = ctgov_fixture_env(&ctgov_base);
    env.set("BIOMCP_MYDISEASE_BASE", &disease_base);

    let page = super::super::search_page(
        &TrialSearchFilters {
            condition: Some("MF".into()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect("the default-source keyword search still runs");
    tokio::task::yield_now().await;
    ctgov_server.abort();
    disease_server.abort();
    drop(env);

    let note = page
        .partial_note
        .expect("the ambiguity note reaches the page");
    assert!(
        note.contains("ClinicalTrials.gov search ran a plain keyword search for 'MF'"),
        "{note}"
    );
    assert!(
        note.contains("an ambiguous abbreviation: 2 diseases hold it as an exact name or synonym"),
        "{note}"
    );
    assert!(
        note.contains(
            "Holders: mycosis fungoides (MONDO:0009691), myotonia fluctuans (MONDO:0020481)"
        ),
        "{note}"
    );
    assert!(
        note.contains("Clinical reading: 'MF' also names myelofibrosis (MONDO:0009692)"),
        "{note}"
    );
    assert!(
        note.contains(
            "Results may mix several conditions' trials; retry with the full disease name"
        ),
        "{note}"
    );
    assert_eq!(page.results.len(), 1, "the keyword results stay returned");

    let disease_requests = disease_requests
        .lock()
        .expect("lock MyDisease fixture requests")
        .clone();
    assert_eq!(
        disease_requests.len(),
        1,
        "one grounding query for the abbreviation: {disease_requests:?}"
    );
    let ctgov_requests = ctgov_requests
        .lock()
        .expect("lock CTGov fixture requests")
        .clone();
    assert!(
        ctgov_requests
            .iter()
            .any(|line| line.contains("query.cond=MF")),
        "the registry request stays the raw keyword search: {ctgov_requests:?}"
    );
}

/// Ticket 2040: the note never fires for a full-word condition, which
/// adds no MyDisease request to ordinary searches.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn default_source_skips_the_ambiguity_note_for_full_words() {
    let ctgov_body = r#"{"totalCount":0,"studies":[]}"#.to_string();
    let (ctgov_base, _ctgov_requests, ctgov_server) = ctgov_json_fixture(ctgov_body).await;
    let (disease_base, disease_requests, disease_server) = mydisease_json_fixture(|request| {
        request
            .contains("/query?")
            .then(|| r#"{"total":0,"hits":[]}"#.to_string())
    })
    .await;

    let mut env = ctgov_fixture_env(&ctgov_base);
    env.set("BIOMCP_MYDISEASE_BASE", &disease_base);

    let page = super::super::search_page(
        &TrialSearchFilters {
            condition: Some("melanoma".into()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect("the full-word search runs as before");
    tokio::task::yield_now().await;
    ctgov_server.abort();
    disease_server.abort();
    drop(env);

    assert!(
        page.partial_note.is_none(),
        "a full word never grounds through the ambiguity check"
    );
    let disease_requests = disease_requests
        .lock()
        .expect("lock MyDisease fixture requests")
        .clone();
    assert!(
        disease_requests.is_empty(),
        "no MyDisease request for a full-word condition: {disease_requests:?}"
    );
}

/// Ticket 2050: a short abbreviation grounds in any letter case, so
/// `search trial -c mf` prints the same note `MF` prints instead of
/// silently returning the keyword search.
#[tokio::test]
#[serial_test::serial(source_env)]
async fn default_source_notes_the_lowercase_abbreviation_like_the_uppercase() {
    let mf_hits =
        include_str!("../../../../../testdata/sources/mydisease/query_mf.json").to_string();
    let (disease_base, disease_requests, disease_server) =
        mydisease_json_fixture(move |request| request.contains("/query?").then(|| mf_hits.clone()))
            .await;
    let (ctgov_base, ctgov_requests, ctgov_server) =
        ctgov_json_fixture(r#"{"totalCount":0,"studies":[]}"#.to_string()).await;

    let mut env = ctgov_fixture_env(&ctgov_base);
    env.set("BIOMCP_MYDISEASE_BASE", &disease_base);

    let page = super::super::search_page(
        &TrialSearchFilters {
            condition: Some("mf".into()),
            ..Default::default()
        },
        1,
        0,
        None,
    )
    .await
    .expect("the lowercase keyword search still runs");
    tokio::task::yield_now().await;
    ctgov_server.abort();
    disease_server.abort();
    drop(env);

    let note = page
        .partial_note
        .expect("the lowercase abbreviation grounds like the uppercase");
    assert!(
        note.contains("plain keyword search for 'mf'"),
        "{note}"
    );
    assert!(
        note.contains("an ambiguous abbreviation: 2 diseases hold it as an exact name or synonym"),
        "{note}"
    );
    assert!(
        note.contains("Clinical reading: 'mf' also names myelofibrosis (MONDO:0009692)"),
        "{note}"
    );
    let disease_requests = disease_requests
        .lock()
        .expect("lock MyDisease fixture requests")
        .clone();
    assert_eq!(
        disease_requests.len(),
        1,
        "one grounding query for the lowercase abbreviation: {disease_requests:?}"
    );
    let ctgov_requests = ctgov_requests
        .lock()
        .expect("lock CTGov fixture requests")
        .clone();
    assert!(
        ctgov_requests
            .iter()
            .any(|line| line.contains("query.cond=mf")),
        "the registry request stays the raw keyword search: {ctgov_requests:?}"
    );
}
