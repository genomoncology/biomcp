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
    );

    assert_eq!(result.article.paper_id.as_deref(), Some("paper-1"));
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].paper.pmid.as_deref(), Some("24200969"));
    assert_eq!(result.edges[0].contexts, ["Example context"]);
    assert_eq!(result.edges[0].intents, ["Background"]);
    assert!(!result.edges[0].is_influential);
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
    );
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
    );

    assert_eq!(result.article.paper_id.as_deref(), Some("paper-1"));
    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].paper.pmid.as_deref(), Some("19424861"));
    assert_eq!(result.edges[0].paper.journal.as_deref(), Some("Cell"));
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
