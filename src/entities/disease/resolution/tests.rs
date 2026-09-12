use super::super::test_support::*;
use super::*;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn json_response(body: &str) -> Vec<u8> {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
    .into_bytes()
}

async fn exact_resolution_fixture(
    handler: impl Fn(&str) -> &'static str + Send + Sync + 'static,
) -> (
    MyDiseaseClient,
    Arc<Mutex<Vec<String>>>,
    tokio::task::JoinHandle<()>,
) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind exact-resolution fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let requests = Arc::new(Mutex::new(Vec::new()));
    let observed = Arc::clone(&requests);
    let handler = Arc::new(handler);
    let server = tokio::spawn(async move {
        while let Ok((mut stream, _)) = listener.accept().await {
            let mut bytes = vec![0_u8; 16 * 1024];
            let length = stream.read(&mut bytes).await.expect("read request");
            let request = String::from_utf8_lossy(&bytes[..length]).into_owned();
            observed
                .lock()
                .expect("lock exact-resolution requests")
                .push(request.clone());
            let response = json_response(handler(&request));
            stream.write_all(&response).await.expect("write response");
        }
    });
    (
        MyDiseaseClient::new_for_test(base).expect("test MyDisease client"),
        requests,
        server,
    )
}

#[test]
fn normalize_disease_id_basic() {
    assert_eq!(
        normalize_disease_id("MONDO:0005105"),
        Some("MONDO:0005105".into())
    );
    assert_eq!(
        normalize_disease_id("mondo:0005105"),
        Some("MONDO:0005105".into())
    );
    assert_eq!(
        normalize_disease_id(" DOID:1909 "),
        Some("DOID:1909".into())
    );
    assert_eq!(normalize_disease_id("lung cancer"), None);
    assert_eq!(normalize_disease_id("MONDO:"), None);
    assert_eq!(normalize_disease_id("HP:0002861"), None);
}

#[test]
fn parse_disease_lookup_input_distinguishes_canonical_crosswalk_and_text() {
    assert_eq!(
        parse_disease_lookup_input("MONDO:0005105"),
        DiseaseLookupInput::CanonicalOntologyId("MONDO:0005105".into())
    );
    assert_eq!(
        parse_disease_lookup_input("mesh:D008545"),
        DiseaseLookupInput::CrosswalkId(DiseaseXrefKind::Mesh, "D008545".into())
    );
    assert_eq!(
        parse_disease_lookup_input("OMIM:155600"),
        DiseaseLookupInput::CrosswalkId(DiseaseXrefKind::Omim, "155600".into())
    );
    assert_eq!(
        parse_disease_lookup_input("ICD10CM:Q07.0"),
        DiseaseLookupInput::CrosswalkId(DiseaseXrefKind::Icd10Cm, "Q07.0".into())
    );
    assert_eq!(
        parse_disease_lookup_input("Arnold Chiari syndrome"),
        DiseaseLookupInput::FreeText
    );
}

#[test]
fn preferred_crosswalk_hit_prefers_mondo_then_doid_then_lexicographic_id() {
    let best = preferred_crosswalk_hit(vec![
        test_disease_hit("DOID:1909", "melanoma", &[], &[]),
        test_disease_hit("MONDO:0005105", "melanoma", &[], &[]),
        test_disease_hit("MESH:D008545", "melanoma", &[], &[]),
    ])
    .expect("a best hit should be selected");
    assert_eq!(best.id, "MONDO:0005105");
}

#[test]
fn resolver_queries_adds_cml_fallback_variant() {
    let queries = resolver_queries("chronic myeloid leukemia");
    assert!(
        queries
            .iter()
            .any(|query| query == "chronic myelogenous leukemia")
    );
    assert!(
        queries
            .iter()
            .any(|query| query == "chronic myelogenous leukemia, bcr-abl1 positive")
    );
}

#[test]
fn resolver_queries_adds_hodgkin_alias_variants() {
    let queries = resolver_queries("Hodgkin lymphoma");
    assert!(queries.iter().any(|query| query == "hodgkins lymphoma"));
    assert!(queries.iter().any(|query| query == "hodgkin disease"));
}

#[test]
fn resolve_disease_hit_by_name_direct_rejects_weak_contains_only_match() {
    let queries = resolver_queries("Hodgkin lymphoma");
    let hit = test_disease_hit("MONDO:0015760", "T-cell non-Hodgkin lymphoma", &[], &[]);

    assert!(
        best_disease_candidate_score_for_queries(&queries, &hit) < MIN_DIRECT_DISEASE_MATCH_SCORE
    );
}

#[test]
fn disease_candidate_score_prefers_canonical_colorectal_match_over_subtype() {
    let broad = disease_candidate_score("colorectal cancer", "colorectal carcinoma");
    let subtype = disease_candidate_score(
        "colorectal cancer",
        "hereditary nonpolyposis colorectal cancer type 6",
    );
    assert!(broad > subtype);
}

#[test]
fn scored_best_candidate_for_queries_prefers_hodgkin_alias_over_non_hodgkin_contains_match() {
    let queries = resolver_queries("Hodgkin lymphoma");
    let best = scored_best_candidate_for_queries(
        &queries,
        vec![
            test_disease_hit("MONDO:0015760", "T-cell non-Hodgkin lymphoma", &[], &[]),
            test_disease_hit(
                "MONDO:0004952",
                "Hodgkins lymphoma",
                &["Hodgkin disease"],
                &[],
            ),
        ],
    )
    .expect("a best hit should be selected");

    assert_eq!(best.id, "MONDO:0004952");
}
#[test]
fn rerank_disease_search_hits_prefers_canonical_exact_candidate_across_query_variants() {
    let canonical = test_disease_hit(
        "MONDO:0024331",
        "colorectal carcinoma",
        &["colorectal cancer"],
        &["colorectal cancer"],
    );

    let ranked = rerank_disease_search_hits(
        "colorectal cancer",
        vec![
            (
                0,
                vec![test_disease_hit(
                    "MONDO:0101010",
                    "hereditary nonpolyposis colorectal cancer type 6",
                    &[],
                    &[],
                )],
            ),
            (
                1,
                vec![
                    canonical,
                    test_disease_hit(
                        "MONDO:0101010",
                        "hereditary nonpolyposis colorectal cancer type 6",
                        &[],
                        &[],
                    ),
                ],
            ),
        ],
    );

    let ids = ranked.iter().map(|hit| hit.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, vec!["MONDO:0024331", "MONDO:0101010"]);
}

#[test]
fn disease_exact_rank_prefers_exact_then_prefix_then_contains() {
    assert!(
        disease_exact_rank("colorectal cancer", "colorectal cancer")
            > disease_exact_rank("colorectal cancer syndrome", "colorectal cancer")
    );
    assert!(
        disease_exact_rank("colorectal cancer syndrome", "colorectal cancer")
            > disease_exact_rank("metastatic colorectal cancer", "colorectal cancer")
    );
}

#[test]
fn resolver_queries_adds_carcinoma_fallback_for_cancer_terms() {
    let queries = resolver_queries("breast cancer");
    assert!(queries.iter().any(|q| q == "breast cancer"));
    assert!(queries.iter().any(|q| q == "breast carcinoma"));
}

#[test]
fn exact_resolution_matches_only_primary_names_and_declared_synonyms() {
    let hit = test_disease_hit(
        "MONDO:0033642",
        "neurodevelopmental disorder with alopecia and brain abnormalities",
        &["Bachmann-Bupp syndrome"],
        &["BABS"],
    );
    assert!(exact_hit_matches(" bachmann-bupp\u{00a0}syndrome ", &hit));
    assert!(exact_hit_matches("BABS", &hit));
    assert!(!exact_hit_matches("alopecia", &hit));
}

#[test]
fn detail_terms_bounds_and_deduplicates_provider_synonyms() {
    let hit = test_disease_hit(
        "MONDO:0033642",
        "Bachmann-Bupp syndrome",
        &["BABS", "Bachmann-Bupp syndrome", "BABS", "Long synonym"],
        &["BACHMANN-BUPP SYNDROME"],
    );
    let terms = detail_terms("Bachmann-Bupp syndrome", "MONDO:0033642", hit)
        .expect("valid selected detail");
    assert_eq!(terms.canonical_id.as_deref(), Some("MONDO:0033642"));
    assert_eq!(
        terms.canonical_name.as_deref(),
        Some("Bachmann-Bupp syndrome")
    );
    assert_eq!(terms.synonyms, vec!["BABS", "Long synonym"]);
}

#[test]
fn detail_terms_rejects_id_disagreement_and_malformed_synonyms() {
    let mut hit = test_disease_hit("MONDO:0033642", "Bachmann-Bupp syndrome", &[], &[]);
    hit.mondo = Some(serde_json::json!({
        "name": "Bachmann-Bupp syndrome",
        "synonym": {"exact": {"nested": ["not a flat synonym"]}}
    }));
    assert!(detail_terms("Bachmann-Bupp syndrome", "MONDO:0033642", hit).is_err());
    let mismatch = test_disease_hit("DOID:123", "Bachmann-Bupp syndrome", &[], &[]);
    assert!(detail_terms("Bachmann-Bupp syndrome", "MONDO:0033642", mismatch).is_err());
}

#[tokio::test]
async fn exact_resolution_zero_and_multiple_identities_each_stop_after_query() {
    for body in [
        r#"{"total":0,"hits":[]}"#,
        r#"{"total":2,"hits":[
            {"_id":"MONDO:1","mondo":{"name":"Exact syndrome"}},
            {"_id":"DOID:2","disease_ontology":{"name":"Exact syndrome"}}
        ]}"#,
    ] {
        let (client, requests, server) = exact_resolution_fixture(move |_| body).await;
        let terms = resolve_exact_disease_terms(&client, "Exact syndrome")
            .await
            .expect("absence and ambiguity safely retain the literal term");
        server.abort();
        assert_eq!(terms.requested, "Exact syndrome");
        assert!(terms.canonical_id.is_none());
        let requests = requests.lock().expect("lock requests");
        assert_eq!(requests.len(), 1);
        assert!(requests[0].starts_with("GET /query?"));
        assert!(requests[0].contains("size=50&from=0"));
    }
}

#[tokio::test]
async fn exact_resolution_one_identity_fetches_consistent_detail_once() {
    let (client, requests, server) = exact_resolution_fixture(|request| {
        if request.starts_with("GET /query?") {
            r#"{"total":1,"hits":[{"_id":"MONDO:33642","disease_ontology":{"name":"Long disorder","synonyms":{"exact":"Bachmann-Bupp syndrome"}}}]}"#
        } else {
            r#"{"_id":"MONDO:33642","disease_ontology":{"name":"Long disorder","synonyms":{"exact":["Bachmann-Bupp syndrome","BABS"]}}}"#
        }
    })
    .await;
    let terms = resolve_exact_disease_terms(&client, "Bachmann-Bupp syndrome")
        .await
        .expect("one exact identity");
    server.abort();
    assert_eq!(terms.canonical_id.as_deref(), Some("MONDO:33642"));
    assert_eq!(terms.canonical_name.as_deref(), Some("Long disorder"));
    assert_eq!(terms.synonyms, vec!["BABS"]);
    let requests = requests.lock().expect("lock requests");
    assert_eq!(requests.len(), 2);
    assert!(requests[0].starts_with("GET /query?"));
    assert!(requests[1].starts_with("GET /disease/MONDO:33642?"));
}

#[tokio::test]
async fn exact_resolution_rejects_incomplete_query_without_fetching_detail() {
    let (client, requests, server) = exact_resolution_fixture(
        |_| r#"{"total":2,"hits":[{"_id":"MONDO:33642","mondo":{"name":"Exact syndrome"}}]}"#,
    )
    .await;
    let error = resolve_exact_disease_terms(&client, "Exact syndrome")
        .await
        .expect_err("an incomplete candidate page cannot prove uniqueness");
    server.abort();
    assert!(matches!(error, BioMcpError::SourceUnavailable { .. }));
    assert_eq!(requests.lock().expect("lock requests").len(), 1);
}

#[tokio::test]
async fn exact_resolution_direct_mondo_and_doid_ids_each_use_one_request() {
    for id in ["MONDO:33642", "DOID:123"] {
        let response_id = id;
        let (client, requests, server) = exact_resolution_fixture(move |_| {
            if response_id.starts_with("MONDO") {
                r#"{"_id":"MONDO:33642","mondo":{"name":"Exact syndrome","synonym":"Alias"}}"#
            } else {
                r#"{"_id":"DOID:123","disease_ontology":{"name":"Exact syndrome","synonyms":["Alias"]}}"#
            }
        })
        .await;
        let terms = resolve_exact_disease_terms(&client, id)
            .await
            .expect("direct ontology detail");
        server.abort();
        assert_eq!(terms.canonical_id.as_deref(), Some(id));
        let requests = requests.lock().expect("lock requests");
        assert_eq!(requests.len(), 1);
        assert!(requests[0].starts_with(&format!("GET /disease/{id}?")));
    }
}

#[tokio::test]
async fn exact_resolution_maps_selected_detail_inconsistency_to_safe_source_error() {
    let (client, requests, server) = exact_resolution_fixture(
        |_| r#"{"_id":"DOID:999","disease_ontology":{"name":"Wrong identity"}}"#,
    )
    .await;
    let error = resolve_exact_disease_terms(&client, "MONDO:33642")
        .await
        .expect_err("detail ID disagreement must fail closed");
    server.abort();
    assert!(matches!(error, BioMcpError::SourceUnavailable { .. }));
    assert_eq!(requests.lock().expect("lock requests").len(), 1);
}
