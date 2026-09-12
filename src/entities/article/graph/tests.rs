#[allow(unused_imports)]
use super::super::test_support::*;
use super::citation_evidence::citation_evidence;
use super::citation_evidence::{
    JatsEvidenceOutcome, install_jats_citation_seam, run_jats_extraction,
};
use super::*;
#[cfg(test)]
mod admission;
use crate::sources::semantic_scholar::{
    SemanticScholarCitationEdge, SemanticScholarClient, SemanticScholarExternalIds,
    SemanticScholarGraphResponse, SemanticScholarPaper, SemanticScholarRecommendationsResponse,
    SemanticScholarReferenceEdge,
};
use crate::transform::article::{JatsCitationExtraction, JatsCitationTargetIds};
use reqwest::StatusCode;

fn semantic_paper(
    paper_id: &str,
    pmid: &str,
    title: &str,
    venue: &str,
    year: u32,
) -> SemanticScholarPaper {
    SemanticScholarPaper {
        paper_id: Some(paper_id.to_string()),
        external_ids: Some(SemanticScholarExternalIds {
            pubmed: Some(pmid.to_string()),
            ..Default::default()
        }),
        title: Some(title.to_string()),
        venue: Some(venue.to_string()),
        year: Some(year),
        ..Default::default()
    }
}

#[test]
fn semantic_scholar_lookup_id_supports_arxiv_and_paper_ids() {
    assert_eq!(
        semantic_scholar_lookup_id("arXiv:2401.01234"),
        Some("ARXIV:2401.01234".to_string())
    );
    assert_eq!(
        semantic_scholar_lookup_id("0123456789abcdef0123456789abcdef01234567"),
        Some("0123456789abcdef0123456789abcdef01234567".to_string())
    );
}

#[test]
fn citations_map_semantic_scholar_edges() {
    let article = related_paper_from_semantic_scholar(&semantic_paper(
        "paper-1",
        "22663011",
        "Seed paper",
        "Science",
        2012,
    ));
    let result = article_graph_from_citations(
        article,
        SemanticScholarGraphResponse {
            offset: Some(0),
            next: Some(10),
            data: vec![SemanticScholarCitationEdge {
                contexts: vec!["Example context".into()],
                intents: vec!["Background".into()],
                is_influential: Some(false),
                citing_paper: semantic_paper(
                    "paper-2",
                    "24200969",
                    "Related paper",
                    "Nature",
                    2024,
                ),
            }],
        },
        0,
        10,
        "22663011",
    )
    .expect("valid graph page");

    assert_eq!(result.article.paper_id.as_deref(), Some("paper-1"));
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].paper.pmid.as_deref(), Some("24200969"));
    assert_eq!(result.edges[0].contexts, ["Example context"]);
    assert_eq!(result.edges[0].intents, ["Background"]);
    assert!(!result.edges[0].is_influential);
    assert_eq!(
        serde_json::to_value(&result.pagination).unwrap(),
        serde_json::json!({
            "offset": 0,
            "limit": 10,
            "returned": 1,
            "next_offset": 10,
            "coverage_status": "continuable"
        })
    );
    assert_eq!(
        result._meta.next_commands,
        ["biomcp article citations 22663011 --limit 10 --offset 10"]
    );
}

#[test]
fn receipted_provider_only_citation_survives_article_graph_mapping() {
    let response: SemanticScholarGraphResponse<SemanticScholarCitationEdge> =
        SemanticScholarClient::decode_json_response(
            StatusCode::OK,
            include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/semantic_scholar/pmid20516115-citations.json"
            )),
            false,
        )
        .unwrap();
    let result = article_graph_from_citations(
        related_paper_from_semantic_scholar(&semantic_paper(
            "059f780c07b87339c275192f1b82662747c28ccd",
            "20516115",
            "Seed paper",
            "Cancer Research",
            2010,
        )),
        response,
        0,
        100,
        "20516115",
    )
    .expect("valid capture");
    let paper = result
        .edges
        .iter()
        .map(|edge| &edge.paper)
        .find(|paper| paper.paper_id.as_deref() == Some("bdb7239fd58ab8fee22b211f96073a3c58dad53d"))
        .expect("captured provider-only citation");
    assert_eq!(paper.pmid, None);
    assert_eq!(paper.doi, None);
    assert_eq!(paper.arxiv_id, None);
}

#[test]
fn references_map_semantic_scholar_edges() {
    let article = related_paper_from_semantic_scholar(&semantic_paper(
        "paper-1",
        "22663011",
        "Seed paper",
        "Science",
        2012,
    ));
    let result = article_graph_from_references(
        article,
        SemanticScholarGraphResponse {
            offset: Some(7),
            next: None,
            data: vec![SemanticScholarReferenceEdge {
                contexts: vec!["Example context".into()],
                intents: vec!["Background".into()],
                is_influential: Some(false),
                cited_paper: semantic_paper(
                    "paper-2",
                    "19424861",
                    "Referenced paper",
                    "Cell",
                    2009,
                ),
            }],
        },
        7,
        3,
        "22663011",
    )
    .expect("valid graph page");

    assert_eq!(result.article.paper_id.as_deref(), Some("paper-1"));
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].paper.pmid.as_deref(), Some("19424861"));
    assert_eq!(result.edges[0].paper.journal.as_deref(), Some("Cell"));
    assert_eq!(result.pagination.offset, 7);
    assert_eq!(result.pagination.next_offset, None);
    assert_eq!(
        result.pagination.coverage_status,
        GraphCoverageStatus::Exhausted
    );
    assert!(result._meta.next_commands.is_empty());
}

#[test]
fn graph_page_validation_fails_closed_for_bad_offsets_and_next_values() {
    let article = related_paper_from_semantic_scholar(&semantic_paper(
        "paper-1",
        "22663011",
        "Seed paper",
        "Science",
        2012,
    ));
    for (offset, next) in [
        (None, Some(2)),
        (Some(0), Some(2)),
        (Some(1), Some(1)),
        (Some(2), Some(1)),
    ] {
        let response = SemanticScholarGraphResponse::<SemanticScholarCitationEdge> {
            offset,
            next,
            data: Vec::new(),
        };
        assert!(
            article_graph_from_citations(article.clone(), response, 1, 10, "22663011",).is_err()
        );
    }
}

#[test]
fn graph_pages_preserve_provider_order_and_duplicate_edges() {
    let article = related_paper_from_semantic_scholar(&semantic_paper(
        "seed", "22663011", "Seed", "Science", 2012,
    ));
    let edge = SemanticScholarCitationEdge {
        contexts: vec!["first context".into(), "second context".into()],
        intents: vec!["Background".into(), "Methods".into()],
        is_influential: Some(true),
        citing_paper: semantic_paper("duplicate", "24200969", "Duplicate", "Nature", 2024),
    };
    let result = article_graph_from_citations(
        article,
        SemanticScholarGraphResponse {
            offset: Some(4),
            next: Some(8),
            data: vec![edge.clone(), edge],
        },
        4,
        4,
        "22663011",
    )
    .expect("valid duplicate page");

    assert_eq!(result.edges.len(), 2);
    assert_eq!(result.edges[0], result.edges[1]);
    assert_eq!(result.edges[0].intents, ["Background", "Methods"]);
    assert_eq!(
        result.edges[0].contexts,
        ["first context", "second context"]
    );
    assert!(result.edges[0].is_influential);
}

#[test]
fn empty_pages_follow_provider_next_for_both_directions() {
    let article = related_paper_from_semantic_scholar(&semantic_paper(
        "seed", "22663011", "Seed", "Science", 2012,
    ));
    let citation = article_graph_from_citations(
        article.clone(),
        SemanticScholarGraphResponse {
            offset: Some(9),
            next: None,
            data: Vec::new(),
        },
        9,
        2,
        "22663011",
    )
    .expect("empty exhausted citation page");
    let reference = article_graph_from_references(
        article,
        SemanticScholarGraphResponse {
            offset: Some(9),
            next: Some(20),
            data: Vec::new(),
        },
        9,
        2,
        "22663011",
    )
    .expect("empty continuable reference page");

    assert_eq!(citation.pagination.returned, 0);
    assert_eq!(
        citation.pagination.coverage_status,
        GraphCoverageStatus::Exhausted
    );
    assert!(citation._meta.next_commands.is_empty());
    assert_eq!(reference.pagination.returned, 0);
    assert_eq!(
        reference.pagination.coverage_status,
        GraphCoverageStatus::Continuable
    );
    assert_eq!(reference.pagination.next_offset, Some(20));
}

#[test]
fn graph_continuation_quotes_the_trimmed_caller_id() {
    assert_eq!(
        graph_continuation_command(
            "  10.1/example; echo owned  ",
            GraphDirection::References,
            7,
            9,
        ),
        "biomcp article references \"10.1/example; echo owned\" --limit 7 --offset 9"
    );
}

#[test]
fn recommendations_map_semantic_scholar_papers() {
    let seed = related_paper_from_semantic_scholar(&semantic_paper(
        "paper-1",
        "22663011",
        "Seed paper",
        "Science",
        2012,
    ));
    let result = article_recommendations_from_response(
        vec![seed],
        Vec::new(),
        SemanticScholarRecommendationsResponse {
            recommended_papers: vec![semantic_paper(
                "paper-3",
                "28052061",
                "Recommended paper",
                "Nature Medicine",
                2017,
            )],
        },
    );

    assert_eq!(result.positive_seeds.len(), 1);
    assert_eq!(result.recommendations.len(), 1);
    assert_eq!(result.recommendations[0].pmid.as_deref(), Some("28052061"));
}

#[tokio::test]
async fn jats_extraction_seam_controls_the_blocking_parse_outcome() {
    struct SeamGuard;
    impl Drop for SeamGuard {
        fn drop(&mut self) {
            install_jats_citation_seam(None);
        }
    }
    let _guard = SeamGuard;

    // A seam parse failure bounds into the public fulltext_unavailable state.
    install_jats_citation_seam(Some(|_xml: &str, _target: &JatsCitationTargetIds| Err(())));
    let unavailable = run_jats_extraction(
        "PMC1",
        "<article/>",
        &JatsCitationTargetIds {
            doi: None,
            pmid: None,
            pmcid: None,
        },
        tokio::time::Instant::now() + std::time::Duration::from_secs(5),
    )
    .await
    .unwrap();
    assert!(matches!(
        unavailable,
        JatsEvidenceOutcome::FulltextUnavailable
    ));

    // A seam success settles through the same single-permit path.
    install_jats_citation_seam(Some(|_xml: &str, _target: &JatsCitationTargetIds| {
        Ok(JatsCitationExtraction::Linked {
            ref_id: "bib7".to_string(),
            passages: Vec::new(),
        })
    }));
    let parsed = run_jats_extraction(
        "PMC1",
        "<article/>",
        &JatsCitationTargetIds {
            doi: None,
            pmid: None,
            pmcid: None,
        },
        tokio::time::Instant::now() + std::time::Duration::from_secs(5),
    )
    .await
    .unwrap();
    match parsed {
        JatsEvidenceOutcome::Parsed { pmcid, extraction } => {
            assert_eq!(pmcid, "PMC1");
            assert!(matches!(extraction, JatsCitationExtraction::Linked { .. }));
        }
        other => panic!("expected parsed seam outcome, got {other:?}"),
    }
}

// --- Ticket 1145: directed citation-evidence traversal ---

use std::sync::Arc;
use std::sync::Mutex;

const CITING_PID: &str = "0123456789abcdef0123456789abcdef01234567";
const CITED_PID: &str = "fedcba9876543210fedcba9876543210fedcba98";

#[derive(Clone, Default)]
struct GraphPage {
    offset: u64,
    next: Option<u64>,
    /// (cited paper ID, contexts) per edge.
    edges: Vec<(String, Vec<&'static str>)>,
}

impl GraphPage {
    fn body(&self) -> String {
        let edges: Vec<String> = self
            .edges
            .iter()
            .map(|(pid, contexts)| {
                let contexts = contexts
                    .iter()
                    .map(|value| format!("\"{value}\""))
                    .collect::<Vec<_>>()
                    .join(",");
                format!(
                    "{{\"contexts\":[{contexts}],\"intents\":[],\"citedPaper\":{{\"paperId\":\"{pid}\"}}}}"
                )
            })
            .collect();
        let next = match self.next {
            Some(value) => value.to_string(),
            None => "null".to_string(),
        };
        format!(
            "{{\"offset\":{},\"next\":{},\"data\":[{}]}}",
            self.offset,
            next,
            edges.join(",")
        )
    }
}

async fn spawn_citation_fixture(
    pages: Vec<GraphPage>,
) -> (
    super::super::test_support::TestHttpFixture,
    Arc<Mutex<Vec<String>>>,
) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let logged = requests.clone();
    let pages_arc = Arc::new(Mutex::new(pages));
    let fixture = super::super::test_support::TestHttpFixture::spawn(move |request| {
        let mut parts = request.splitn(2, ' ');
        let method = parts.next().unwrap_or_default();
        let target = parts
            .next()
            .unwrap_or_default()
            .split_whitespace()
            .next()
            .unwrap_or_default();
        requests.lock().unwrap().push(format!("{method} {target}"));
        let reply = if method == "POST" {
            // The singleton seed batch must answer exactly one paper row.
            let requested = if request.contains(CITING_PID) {
                CITING_PID
            } else {
                CITED_PID
            };
            format!("[{{\"paperId\":\"{requested}\",\"title\":\"T\"}}]")
        } else if target.contains("/references") {
            let mut queue = pages_arc.lock().unwrap();
            if queue.is_empty() {
                "{\"offset\":0,\"next\":null,\"data\":[]}".to_string()
            } else {
                queue.remove(0).body()
            }
        } else {
            "{\"offset\":0,\"next\":null,\"data\":[]}".to_string()
        };
        super::super::test_support::TestHttpReply::Bytes(
            super::super::test_support::test_http_response(
                "200 OK",
                "application/json",
                reply.as_bytes(),
            ),
        )
    })
    .await;
    (fixture, logged)
}

fn graph_page(
    offset: u64,
    next: Option<u64>,
    edges: Vec<(String, Vec<&'static str>)>,
) -> GraphPage {
    GraphPage {
        offset,
        next,
        edges,
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_provider_context_needs_no_fulltext_request() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-context");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, requests) = spawn_citation_fixture(vec![graph_page(
        0,
        Some(100),
        vec![(CITED_PID.to_ascii_uppercase(), vec![" Context one "])],
    )])
    .await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let result = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap();
    assert_eq!(result.provider_contexts, vec!["Context one".to_string()]);
    let requests = requests.lock().unwrap().join("\n");
    assert_eq!(requests.matches("/references").count(), 1, "{requests}");
    assert!(!requests.contains("fullTextXML"), "{requests}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_deduplicates_duplicate_edges_and_contexts() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-dedup");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, requests) = spawn_citation_fixture(vec![graph_page(
        0,
        None,
        vec![
            (CITED_PID.to_string(), vec!["Shared", " Alpha ", "Shared"]),
            (CITED_PID.to_string(), vec!["Alpha", "Beta"]),
        ],
    )])
    .await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let result = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap();
    assert_eq!(
        result.provider_contexts,
        vec![
            "Shared".to_string(),
            "Alpha".to_string(),
            "Beta".to_string()
        ]
    );
    let requests = requests.lock().unwrap().join("\n");
    assert_eq!(requests.matches("/references").count(), 1, "{requests}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_follows_the_advertised_next_offset() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-next");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, requests) = spawn_citation_fixture(vec![
        graph_page(
            0,
            Some(100),
            vec![(
                "9999999999999999999999999999999999999999".to_string(),
                vec![],
            )],
        ),
        graph_page(
            100,
            None,
            vec![(CITED_PID.to_string(), vec!["Second page"])],
        ),
    ])
    .await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let result = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap();
    assert_eq!(result.provider_contexts, vec!["Second page".to_string()]);
    let requests = requests.lock().unwrap().join("\n");
    assert!(requests.contains("offset=100"), "{requests}");
    assert_eq!(requests.matches("/references").count(), 2, "{requests}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_stops_after_the_matching_page() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-stop");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, requests) = spawn_citation_fixture(vec![
        graph_page(0, Some(100), vec![(CITED_PID.to_string(), vec!["Hit"])]),
        graph_page(100, None, vec![(CITED_PID.to_string(), vec!["Never"])]),
    ])
    .await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let result = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap();
    assert_eq!(result.provider_contexts, vec!["Hit".to_string()]);
    let requests = requests.lock().unwrap().join("\n");
    assert_eq!(requests.matches("/references").count(), 1, "{requests}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_exhausted_pages_return_the_directed_not_found() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-notfound");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, _requests) = spawn_citation_fixture(vec![graph_page(0, None, vec![])]).await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let error = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap_err();
    match error {
        BioMcpError::NotFound {
            entity,
            id,
            suggestion,
        } => {
            assert_eq!(entity, "directed citation");
            assert_eq!(id, format!("{CITING_PID} -> {CITED_PID}"));
            assert_eq!(
                suggestion,
                "Semantic Scholar exhausted the directed reference pages without finding this pair."
            );
        }
        other => panic!("expected not found, got {other:?}"),
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_rejects_a_mismatched_page_offset() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-offset");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, _requests) = spawn_citation_fixture(vec![graph_page(7, None, vec![])]).await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let error = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{error:?}").contains("offset"),
        "expected offset decode error, got {error:?}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_bounds_the_traversal_at_three_pages() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-cap");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let (fixture, requests) = spawn_citation_fixture(vec![
        graph_page(0, Some(100), vec![]),
        graph_page(100, Some(200), vec![]),
        graph_page(200, Some(300), vec![]),
        graph_page(300, None, vec![]),
    ])
    .await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let error = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{error:?}").contains("three-page bound"),
        "expected bounded unavailable error, got {error:?}"
    );
    let requests = requests.lock().unwrap().join("\n");
    assert_eq!(requests.matches("/references").count(), 3, "{requests}");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_compares_paper_ids_case_insensitively() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-case");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let upper = CITED_PID.to_ascii_uppercase();
    let (fixture, _requests) =
        spawn_citation_fixture(vec![graph_page(0, None, vec![(upper, vec!["Cased"])])]).await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let result = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap();
    assert_eq!(result.provider_contexts, vec!["Cased".to_string()]);
}

#[test]
fn graph_edge_evidence_meta_orders_arguments_per_direction() {
    let edge_with = |contexts: Vec<&str>, pmid: Option<&str>| super::ArticleGraphEdge {
        paper: super::ArticleRelatedPaper {
            paper_id: Some(CITED_PID.to_string()),
            pmid: pmid.map(str::to_string),
            doi: None,
            arxiv_id: None,
            title: "T".into(),
            journal: None,
            year: None,
        },
        intents: vec![],
        contexts: contexts.into_iter().map(str::to_string).collect(),
        is_influential: false,
        _meta: None,
    };
    let caller = CITING_PID;
    // Contextless citations edge: the edge paper is the citing ID, so its
    // executable identifier comes first.
    let meta = graph_edge_evidence_meta(
        &edge_with(vec![], Some("31666226")),
        GraphDirection::Citations,
        caller,
    )
    .expect("contextless citations edge carries the evidence command");
    assert_eq!(
        meta.next_commands,
        vec![format!(
            "biomcp article citation-evidence 31666226 {caller}"
        )]
    );
    // References direction keeps the caller first: the caller cites the edge.
    let meta = graph_edge_evidence_meta(
        &edge_with(vec![], Some("31666226")),
        GraphDirection::References,
        caller,
    )
    .expect("contextless references edge carries the evidence command");
    assert_eq!(
        meta.next_commands,
        vec![format!(
            "biomcp article citation-evidence {caller} 31666226"
        )]
    );
    // A contextual edge carries no edge-local command.
    assert!(
        graph_edge_evidence_meta(
            &edge_with(vec!["context"], None),
            GraphDirection::Citations,
            caller
        )
        .is_none()
    );
    // Whitespace-only contexts still count as contextless.
    assert!(
        graph_edge_evidence_meta(
            &edge_with(vec!["  "], None),
            GraphDirection::Citations,
            caller
        )
        .is_some()
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_rejects_equal_and_decreasing_next_values() {
    for pages in [
        vec![graph_page(0, Some(0), vec![])],
        vec![
            graph_page(0, Some(100), vec![]),
            graph_page(100, Some(50), vec![]),
        ],
    ] {
        let mut env = TestEnv::new();
        let cache = crate::test_support::TempDirGuard::new("citation-bad-next");
        env.set("BIOMCP_CACHE_DIR", cache.path());
        let (fixture, _requests) = spawn_citation_fixture(pages).await;
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
        let client =
            crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
                &fixture.base,
                |_, _| {},
                |_, _| {},
            )
            .unwrap();
        let error = crate::sources::semantic_scholar::with_test_client(
            client,
            citation_evidence(CITING_PID, CITED_PID, false),
        )
        .await
        .unwrap_err();
        assert!(
            format!("{error:?}").contains("did not advance"),
            "expected continuation error, got {error:?}"
        );
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_rejects_a_page_with_more_rows_than_the_page_size() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-oversize-page");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let oversize: Vec<(String, Vec<&'static str>)> = (0..101)
        .map(|index| (format!("{index:040}"), vec![] as Vec<&'static str>))
        .collect();
    let (fixture, _requests) = spawn_citation_fixture(vec![graph_page(0, None, oversize)]).await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let error = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{error:?}").contains("more rows than the page size"),
        "expected page-size error, got {error:?}"
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_returns_the_bounded_error_when_the_graph_deadline_expires() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-graph-deadline");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    // A zero-millisecond graph budget makes the pre-page deadline check fire
    // before any request is issued, pinning the bounded-unavailable outcome
    // of the ten-second graph deadline deterministically.
    env.set("BIOMCP_TEST_CITATION_GRAPH_DEADLINE_MS", "0");
    let (fixture, requests) = spawn_citation_fixture(vec![graph_page(0, None, vec![])]).await;
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    let client = crate::sources::semantic_scholar::SemanticScholarClient::new_with_cache_observers(
        &fixture.base,
        |_, _| {},
        |_, _| {},
    )
    .unwrap();
    let error = crate::sources::semantic_scholar::with_test_client(
        client,
        citation_evidence(CITING_PID, CITED_PID, false),
    )
    .await
    .unwrap_err();
    assert!(
        format!("{error:?}").contains("bounded deadline"),
        "expected bounded deadline error, got {error:?}"
    );
    let requests = requests.lock().unwrap().join("\n");
    assert_eq!(requests.matches("/references").count(), 0, "{requests}");
}
