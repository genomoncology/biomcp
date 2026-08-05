//! Tier 3 — response parsing. Pure: feeds committed fixture bytes to response
//! decoders and response types. No network, no server.

use super::super::*;
use crate::error::BioMcpError;
use reqwest::StatusCode;

macro_rules! fixture {
    ($name:expr) => {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/semantic_scholar/",
            $name
        ))
    };
}

#[test]
fn parses_paper_detail_fixture() {
    let paper: SemanticScholarPaper = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("paper_detail.json"),
        false,
    )
    .unwrap();

    assert_eq!(paper.paper_id.as_deref(), Some("paper-1"));
    assert_eq!(
        paper
            .external_ids
            .as_ref()
            .and_then(|ids| ids.pubmed.as_deref()),
        Some("22663011")
    );
    assert_eq!(
        paper.tldr.as_ref().and_then(|tldr| tldr.text.as_deref()),
        Some("Compact summary")
    );
    assert_eq!(paper.citation_count, Some(12));
    assert_eq!(paper.influential_citation_count, Some(3));
}

#[test]
fn parses_batch_fixture_with_none_rows() {
    let rows: Vec<Option<SemanticScholarPaper>> = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("paper_batch.json"),
        false,
    )
    .unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0].as_ref().and_then(|row| row.paper_id.as_deref()),
        Some("paper-1")
    );
    assert!(rows[1].is_none());
    assert_eq!(
        rows[2].as_ref().and_then(|row| row.title.as_deref()),
        Some("Two")
    );
}

#[test]
fn parses_search_fixture_and_defaults_null_data() {
    let response: SemanticScholarSearchResponse = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("paper_search.json"),
        false,
    )
    .unwrap();

    assert_eq!(response.total, Some(1));
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].paper_id.as_deref(), Some("paper-1"));
    assert_eq!(
        response.data[0].abstract_text.as_deref(),
        Some("Direct answer abstract.")
    );

    let null_data: SemanticScholarSearchResponse =
        serde_json::from_value(serde_json::json!({ "total": 0, "data": null })).unwrap();
    assert!(null_data.data.is_empty());
}

#[test]
fn parses_graph_and_recommendation_fixtures() {
    let citations: SemanticScholarGraphResponse<SemanticScholarCitationEdge> =
        SemanticScholarClient::decode_json_response(
            StatusCode::OK,
            fixture!("citations.json"),
            false,
        )
        .unwrap();
    assert_eq!(citations.data.len(), 1);
    assert_eq!(
        citations.data[0].citing_paper.paper_id.as_deref(),
        Some("citing-paper")
    );

    let recommendations: SemanticScholarRecommendationsResponse =
        SemanticScholarClient::decode_json_response(
            StatusCode::OK,
            fixture!("recommendations.json"),
            false,
        )
        .unwrap();
    assert_eq!(recommendations.recommended_papers.len(), 1);
    assert_eq!(
        recommendations.recommended_papers[0].paper_id.as_deref(),
        Some("paper-3")
    );
}

#[test]
fn receipted_20516115_graph_captures_keep_provider_identity_and_all_recommendations() {
    let citations: SemanticScholarGraphResponse<SemanticScholarCitationEdge> =
        SemanticScholarClient::decode_json_response(
            StatusCode::OK,
            fixture!("pmid20516115-citations.json"),
            false,
        )
        .unwrap();
    let provider_only = citations
        .data
        .iter()
        .find(|edge| {
            edge.citing_paper.paper_id.as_deref()
                == Some("bdb7239fd58ab8fee22b211f96073a3c58dad53d")
        })
        .expect("captured provider-only citation");
    assert!(
        provider_only
            .citing_paper
            .external_ids
            .as_ref()
            .is_some_and(|ids| {
                ids.pubmed.is_none() && ids.doi.is_none() && ids.arxiv.is_none()
            })
    );

    let recommendations: SemanticScholarRecommendationsResponse =
        SemanticScholarClient::decode_json_response(
            StatusCode::OK,
            fixture!("pmid20516115-recommendations.json"),
            false,
        )
        .unwrap();
    assert_eq!(recommendations.recommended_papers.len(), 10);
    assert!(recommendations.recommended_papers.iter().all(|paper| {
        paper.paper_id.as_deref().is_some_and(|id| !id.is_empty())
            && paper
                .title
                .as_deref()
                .is_some_and(|title| !title.is_empty())
    }));
}

#[test]
fn parses_author_detail_and_search_fixtures_without_inventing_identity() {
    let detail: SemanticScholarAuthor = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("author_detail.json"),
        false,
    )
    .unwrap();

    assert_eq!(detail.author_id.as_deref(), Some("1716151"));
    assert_eq!(detail.name.as_deref(), Some("A. Butte"));
    assert_eq!(
        detail
            .affiliations
            .as_ref()
            .and_then(|values| values.first())
            .map(String::as_str),
        Some("UCSF")
    );
    assert_eq!(detail.paper_count, Some(548));
    assert_eq!(detail.citation_count, Some(50_686));
    assert_eq!(detail.h_index, Some(99));
    assert_eq!(
        detail
            .external_ids
            .as_ref()
            .and_then(|ids| ids.get("DBLP"))
            .and_then(|value| value.as_array())
            .and_then(|values| values.first())
            .and_then(|value| value.as_str()),
        Some("Atul J. Butte")
    );
    assert_eq!(
        detail
            .external_ids
            .as_ref()
            .and_then(|ids| ids.get("ORCID"))
            .and_then(|value| value.as_str()),
        Some("0000-0002-7433-2740")
    );

    let search: SemanticScholarAuthorSearchResponse = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("author_search.json"),
        false,
    )
    .unwrap();
    assert_eq!(search.total, Some(2));
    assert_eq!(search.offset, Some(20));
    assert_eq!(search.next, Some(22));
    assert_eq!(search.data[0].author_id.as_deref(), Some("1716151"));
    assert_eq!(search.data[1].author_id.as_deref(), Some("2269573451"));
    assert!(search.data[1].affiliations.is_none());
    assert!(search.data[1].external_ids.is_none());
}

#[test]
fn parses_author_batch_fixture_with_positional_unavailable_row() {
    let rows: Vec<Option<SemanticScholarAuthor>> = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("author_batch.json"),
        false,
    )
    .unwrap();

    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0]
            .as_ref()
            .and_then(|author| author.author_id.as_deref()),
        Some("1716151")
    );
    assert!(rows[1].is_none());
    assert_eq!(
        rows[2]
            .as_ref()
            .and_then(|author| author.author_id.as_deref()),
        Some("2269573451")
    );
}

#[test]
fn parses_author_papers_identifiers_byline_and_continuation() {
    let page: SemanticScholarAuthorPapersResponse = SemanticScholarClient::decode_json_response(
        StatusCode::OK,
        fixture!("author_papers.json"),
        false,
    )
    .unwrap();

    assert_eq!(page.offset, Some(100));
    assert_eq!(page.next, Some(101));
    let paper = &page.data[0];
    assert_eq!(paper.paper_id.as_deref(), Some("paper-identity-1"));
    assert_eq!(paper.corpus_id, Some(277_710_284));
    assert_eq!(
        paper
            .external_ids
            .as_ref()
            .and_then(|ids| ids.get("PubMed"))
            .and_then(|value| value.as_str()),
        Some("40215974")
    );
    let byline = paper.authors.as_ref().expect("requested authors field");
    assert_eq!(byline[0].author_id.as_deref(), Some("2059910739"));
    assert_eq!(byline[1].author_id.as_deref(), Some("1716151"));
    assert_eq!(byline[1].name.as_deref(), Some("A. Butte"));
}

#[test]
fn author_response_types_keep_null_data_explicit_and_map_bad_responses() {
    let null_data: SemanticScholarAuthorSearchResponse = serde_json::from_value(
        serde_json::json!({ "total": 0, "offset": 0, "next": null, "data": null }),
    )
    .unwrap();
    assert!(null_data.data.is_empty());
    assert_eq!(null_data.next, None);

    let malformed = SemanticScholarClient::decode_json_response::<
        SemanticScholarAuthorPapersResponse,
    >(StatusCode::OK, b"{not-json", false)
    .unwrap_err();
    assert_eq!(malformed.code(), "api_json");

    let unavailable = SemanticScholarClient::decode_json_response::<SemanticScholarAuthor>(
        StatusCode::SERVICE_UNAVAILABLE,
        b"temporarily unavailable",
        false,
    )
    .unwrap_err();
    let message = format!("{unavailable:?}");
    assert_eq!(unavailable.code(), "api");
    assert!(message.contains("503"), "got: {message}");
    assert!(message.contains("source unavailable"), "got: {message}");
    assert!(
        !message.contains("temporarily unavailable"),
        "got: {message}"
    );

    let rate_limited = SemanticScholarClient::decode_json_response::<
        SemanticScholarAuthorSearchResponse,
    >(StatusCode::TOO_MANY_REQUESTS, b"shared rate limit", true)
    .unwrap_err();
    assert!(format!("{rate_limited:?}").contains("Set S2_API_KEY"));
}

#[test]
fn shared_pool_429_returns_dedicated_guidance() {
    let err = SemanticScholarClient::decode_json_response::<SemanticScholarPaper>(
        StatusCode::TOO_MANY_REQUESTS,
        b"shared rate limit",
        true,
    )
    .unwrap_err();

    match err {
        BioMcpError::Api { api, message } => {
            assert_eq!(api, SEMANTIC_SCHOLAR_API);
            assert!(message.contains("Set S2_API_KEY"), "got: {message}");
            assert!(
                message.contains(SEMANTIC_SCHOLAR_DOCS_URL),
                "got: {message}"
            );
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn authenticated_http_error_keeps_status_and_sanitizes_payload() {
    let err = SemanticScholarClient::decode_json_response::<SemanticScholarPaper>(
        StatusCode::FORBIDDEN,
        b"forbidden",
        false,
    )
    .unwrap_err();
    let msg = err.to_string();
    assert_eq!(err.code(), "api");
    assert!(msg.contains("Semantic Scholar"), "got: {msg}");
    assert!(msg.to_ascii_lowercase().contains("retry"), "got: {msg}");
    assert!(!msg.contains("403"), "got: {msg}");
    assert!(!msg.contains("forbidden"), "got: {msg}");
}
