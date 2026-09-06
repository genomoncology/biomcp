#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;
use crate::sources::semantic_scholar::{
    SemanticScholarCitationEdge, SemanticScholarClient, SemanticScholarExternalIds,
    SemanticScholarGraphResponse, SemanticScholarPaper, SemanticScholarRecommendationsResponse,
    SemanticScholarReferenceEdge,
};
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
