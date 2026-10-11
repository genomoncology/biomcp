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

fn holder_hits() -> Vec<MyDiseaseHit> {
    vec![
        // Human record holding the abbreviation `CAD` and the full word
        // `word` as exact synonyms.
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0005000",
            "disease_ontology": {"name": "human synonym holder", "synonyms": {"exact": ["CAD", "word"]}}
        }))
        .expect("human synonym holder"),
        // Non-human record holding the same tokens as exact synonyms:
        // descended from MONDO:0005583 (non-human animal disease).
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:1013329",
            "mondo": {
                "synonym": {"exact": ["CAD", "word"]},
                "ancestors": ["MONDO:0005583", "MONDO:0700102"]
            }
        }))
        .expect("non-human synonym holder"),
        // Exact NAME holder of the full word.
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0006000",
            "mondo": {"name": "word"}
        }))
        .expect("name holder"),
    ]
}

#[test]
fn holder_counting_leaves_nonhuman_records_out() {
    // `CAD` is an abbreviation-shaped token, so synonym holders count —
    // but the veterinary record never counts toward the refusal (ticket
    // 2017: `myeloma` refused because a venom-database record held it).
    let ids = exact_token_holder_ids("CAD", &holder_hits());
    assert_eq!(ids, ["MONDO:0005000".to_string()]);
}

#[test]
fn full_word_holders_count_only_exact_name_holds() {
    // `word` is held as an exact NAME by MONDO:0006000, as an exact
    // synonym by the human MONDO:0005000, and as an exact synonym by the
    // veterinary record; a full word is ambiguous only by that name, so
    // only the name holder counts.
    let ids = exact_token_holder_ids("word", &holder_hits());
    assert_eq!(ids, ["MONDO:0006000".to_string()]);
}

#[test]
fn abbreviation_holders_count_name_and_synonym_holds() {
    // An abbreviation-shaped token keeps the broader count: name and
    // synonym holders both count (MF, CAD, MDS), which is what the refusal
    // table pins.
    let hits = vec![
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0005233",
            "mondo": {"name": "NSCLC", "synonym": {"exact": ["non-small cell lung carcinoma"]}}
        }))
        .expect("name holder"),
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0005234",
            "mondo": {"synonym": {"exact": ["NSCLC"]}}
        }))
        .expect("synonym holder"),
    ];
    let ids = exact_token_holder_ids("NSCLC", &hits);
    assert_eq!(
        ids,
        ["MONDO:0005233".to_string(), "MONDO:0005234".to_string()]
    );
}

#[test]
fn holder_candidate_lines_fill_the_mondo_label() {
    // A record with no `name` in the search response still names itself
    // through the ontology `label` (ticket 2017: CAD listed
    // `MONDO:0018922 (no label in the search response)`).
    let hit: MyDiseaseHit = serde_json::from_value(serde_json::json!({
        "_id": "MONDO:0018922",
        "mondo": {
            "label": "cold agglutinin disease",
            "synonym": {"exact": ["CAD"]}
        }
    }))
    .expect("labelled record");
    let lines = abbreviation_candidate_lines(&[hit]);
    assert_eq!(lines, "- cold agglutinin disease (MONDO:0018922)\n");
}

/// Ticket 2032: a refusal names the clinical reading the source cannot
/// see as a pointer line, never as a source candidate, and only for the
/// abbreviations the table records.
#[test]
fn clinical_reading_pointer_is_a_pointer_not_a_candidate() {
    let holder: MyDiseaseHit = serde_json::from_value(serde_json::json!({
        "_id": "MONDO:0009685",
        "disease_ontology": {
            "name": "Miyoshi muscular dystrophy",
            "synonyms": {"exact": ["MM"]}
        }
    }))
    .expect("single MM holder");
    let message = ambiguous_abbreviation_error("MM", &[holder], true).to_string();
    assert!(
        message.contains("- Miyoshi muscular dystrophy (MONDO:0009685)\n"),
        "{message}"
    );
    assert!(
        message.contains("Clinical reading: 'MM' usually means multiple myeloma (MONDO:0009693)"),
        "{message}"
    );
    assert!(
        message.contains("get disease \"multiple myeloma\""),
        "{message}"
    );
    assert!(!message.contains("- multiple myeloma"), "{message}");

    let untouched = ambiguous_abbreviation_error("CAD", &holder_hits(), false).to_string();
    assert!(!untouched.contains("Clinical reading"), "{untouched}");
}

fn mf_holder_hits() -> Vec<MyDiseaseHit> {
    vec![
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0009691",
            "disease_ontology": {
                "name": "mycosis fungoides",
                "synonyms": {"exact": ["MF"]}
            }
        }))
        .expect("mycosis fungoides holder"),
        serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0020481",
            "mondo": {
                "label": "myotonia fluctuans",
                "synonym": {"exact": ["MF"]}
            }
        }))
        .expect("myotonia fluctuans holder"),
    ]
}

/// Ticket 2040: the MF refusal kept hiding myelofibrosis because no
/// indexed source record holds `MF` on a myelofibrosis record (checked
/// live 2026-10-11 against MyDisease, the Disease Ontology entry, and
/// the NCI Thesaurus synonym lists). The pointer names the reading the
/// registry corpus carries most, with the ontology ID `get disease
/// "myelofibrosis"` resolves to; it stays a pointer line, never a
/// candidate.
#[test]
fn mf_refusal_names_the_myelofibrosis_reading() {
    let message = ambiguous_abbreviation_error("MF", &mf_holder_hits(), false).to_string();
    assert!(
        message.contains("- mycosis fungoides (MONDO:0009691)\n"),
        "{message}"
    );
    assert!(
        message.contains("- myotonia fluctuans (MONDO:0020481)\n"),
        "{message}"
    );
    assert!(
        message.contains(
            "Clinical reading: 'MF' also names myelofibrosis (MONDO:0009692)"
        ),
        "{message}"
    );
    assert!(message.contains("get disease \"myelofibrosis\""), "{message}");
    // The pointer is not a candidate: no myelofibrosis line joins the
    // holder list.
    assert!(!message.contains("- myelofibrosis"), "{message}");
}

/// Ticket 2040: the default trial source asks for the refusal's data,
/// not its error. An abbreviation-shaped condition that would refuse in
/// `get disease` yields the holders, the reason, and the clinical
/// reading; anything else yields nothing.
#[tokio::test]
async fn abbreviated_ambiguity_exposes_the_refusal_data_for_the_trial_note() {
    let (client, requests, server) = exact_resolution_fixture(|request| {
        if request.contains("/query?") {
            include_str!("../../../../testdata/sources/mydisease/query_mf.json")
        } else {
            r#"{"total":0,"hits":[]}"#
        }
    })
    .await;

    let ambiguity = resolve_abbreviated_disease_ambiguity(&client, "MF")
        .await
        .expect("the MF lookup should not error")
        .expect("the recorded MF response must read as ambiguous");
    assert_eq!(ambiguity.requested, "MF");
    assert_eq!(
        ambiguity.reason,
        "2 diseases hold it as an exact name or synonym"
    );
    assert_eq!(
        ambiguity.holders,
        vec![
            "mycosis fungoides (MONDO:0009691)".to_string(),
            "myotonia fluctuans (MONDO:0020481)".to_string(),
        ]
    );
    let reading = ambiguity
        .clinical_reading
        .expect("the pointer table carries MF");
    assert_eq!(reading.label, "myelofibrosis");
    assert_eq!(reading.ontology_id, "MONDO:0009692");
    assert_eq!(reading.relation, "also names");

    // A full-word condition never grounds through this surface, and a
    // token the resolver would resolve yields no ambiguity.
    let plain_word = resolve_abbreviated_disease_ambiguity(&client, "melanoma")
        .await
        .expect("full words never ground here");
    assert!(plain_word.is_none());

    server.abort();
    let requests = requests.lock().expect("lock fixture requests").clone();
    assert_eq!(
        requests.len(),
        1,
        "only the abbreviation-shaped condition reaches MyDisease: {requests:?}"
    );
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
fn rerank_disease_search_hits_ranks_exact_abbreviation_holder_first() {
    // Ticket 1295: the source's exact synonym lists hold "NSCLC" for the
    // parent only; a subtype holds the token inside a longer exact synonym.
    let ranked = rerank_disease_search_hits(
        "NSCLC",
        vec![(
            0,
            vec![
                test_disease_hit(
                    "MONDO:0056806",
                    "lung non-squamous non-small cell carcinoma",
                    &["squamous non-small cell lung carcinoma"],
                    &["non- squamous NSCLC"],
                ),
                test_disease_hit(
                    "MONDO:0005233",
                    "lung non-small cell carcinoma",
                    &[
                        "non-small cell lung carcinoma",
                        "NSCLC",
                        "NSCLC - non-small cell lung cancer",
                    ],
                    &["NSCLC"],
                ),
            ],
        )],
    );

    let ids = ranked.iter().map(|hit| hit.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, vec!["MONDO:0005233", "MONDO:0056806"]);
}

#[test]
fn rerank_disease_search_hits_keeps_every_exact_abbreviation_holder_above_loose_matches() {
    // "CAD" is ambiguous in the source's own exact synonym lists: coronary
    // artery disease, cold agglutinin disease, and alveolar capillary
    // dysplasia each hold it. Search resolves the ambiguity to all exact
    // holders, ranked above token-only matches, in provider order.
    let ranked = rerank_disease_search_hits(
        "CAD",
        vec![(
            0,
            vec![
                test_disease_hit(
                    "MONDO:0014647",
                    "developmental and epileptic encephalopathy 50",
                    &["developmental and epileptic encephalopathy 50"],
                    &[],
                ),
                test_disease_hit("MONDO:0018922", "cold agglutinin disease", &["CAD"], &[]),
                test_disease_hit(
                    "MONDO:0005010",
                    "coronary artery disease",
                    &["CAD"],
                    &["coronary arteriosclerosis"],
                ),
            ],
        )],
    );

    let ids = ranked.iter().map(|hit| hit.id.as_str()).collect::<Vec<_>>();
    assert_eq!(ids, vec!["MONDO:0018922", "MONDO:0005010", "MONDO:0014647"]);
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
async fn exact_resolution_refuses_ambiguous_abbreviation_held_by_two_diseases() {
    // Ticket 1295: "CAD" is an exact synonym of both coronary artery disease
    // and cold agglutinin disease in the source's own lists. Exact resolution
    // must refuse the abbreviation instead of picking one identity.
    let (client, requests, server) = exact_resolution_fixture(|_| {
        r#"{"total":2,"hits":[
            {"_id":"MONDO:0005010","disease_ontology":{"name":"coronary artery disease"},"mondo":{"synonym":{"exact":["CAD"]}}},
            {"_id":"MONDO:0018922","mondo":{"synonym":{"exact":["CAD","cold agglutinin disease"]}}}
        ]}"#
    })
    .await;
    let terms = resolve_exact_disease_terms(&client, "CAD")
        .await
        .expect("ambiguity safely retains the literal term");
    server.abort();
    assert_eq!(terms.requested, "CAD");
    assert!(terms.canonical_id.is_none());
    assert!(terms.canonical_name.is_none());
    assert!(terms.synonyms.is_empty());
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
