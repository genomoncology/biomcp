//! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
//! method / path / query / headers / body that would be sent. Nothing is sent.

use super::super::*;
use crate::error::BioMcpError;
use crate::sources::{HttpMethod, RequestBody};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn client_with_api_key(api_key: Option<&str>) -> SemanticScholarClient {
    SemanticScholarClient {
        client: crate::sources::shared_client().expect("shared client"),
        base: std::borrow::Cow::Borrowed("http://127.0.0.1"),
        api_key: api_key.map(str::to_string),
    }
}

async fn author_fixture_client(
    responses: Vec<&'static str>,
) -> (SemanticScholarClient, tokio::task::JoinHandle<Vec<String>>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind author fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let server = tokio::spawn(async move {
        let mut requests = Vec::with_capacity(responses.len());
        for body in responses {
            let (mut stream, _) = listener.accept().await.expect("accept author request");
            let mut request = Vec::new();
            loop {
                let mut chunk = [0_u8; 4096];
                let read = stream.read(&mut chunk).await.expect("read author request");
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
                let Some(header_end) = request.windows(4).position(|bytes| bytes == b"\r\n\r\n")
                else {
                    continue;
                };
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length: ")
                            .and_then(|value| value.parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                if request.len() >= header_end + 4 + content_length {
                    break;
                }
            }
            requests.push(String::from_utf8_lossy(&request).into_owned());
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body,
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write author response");
        }
        requests
    });
    (
        SemanticScholarClient {
            client: crate::sources::shared_client().expect("shared client"),
            base: std::borrow::Cow::Owned(base),
            api_key: Some("fixture-key".into()),
        },
        server,
    )
}

#[tokio::test]
async fn transport_failures_keep_legacy_api_code_with_source_context() {
    let client = client_with_api_key(None);
    let error = client
        .send_json::<serde_json::Value>(client.client.get("http://127.0.0.1:0"))
        .await
        .expect_err("reserved port zero should be unreachable");

    assert_eq!(error.code(), "api");
    assert_eq!(error.public_projection().source, Some("Semantic Scholar"));
    assert_eq!(
        error.public_projection().recovery,
        Some(crate::error::RecoveryAction::RetryRemoteSource.message())
    );
}

#[test]
fn credential_attachment_requires_canonical_or_explicit_unsafe_fixture_origin() {
    let canonical = reqwest::Url::parse(SEMANTIC_SCHOLAR_BASE).unwrap();
    let canonical_policy = ProviderUrlPolicy::semantic_scholar_api(&canonical).unwrap();
    assert_eq!(
        effective_api_key(&canonical_policy, &canonical, Some("canonical-key".into())).as_deref(),
        Some("canonical-key")
    );

    let override_url = reqwest::Url::parse("https://s2-fixture.example.test").unwrap();
    let override_policy = ProviderUrlPolicy::semantic_scholar_api(&override_url).unwrap();
    assert_eq!(
        effective_api_key(
            &override_policy,
            &override_url,
            Some("must-not-leak".into())
        ),
        None
    );
}

#[test]
fn auth_mode_reports_keyed_or_shared_pool_without_exposing_key() {
    let keyed = client_with_api_key(Some("spec-secret-key-365"));
    assert_eq!(keyed.auth_mode(), SemanticScholarAuthMode::Authenticated);

    let keyless = client_with_api_key(None);
    assert_eq!(keyless.auth_mode(), SemanticScholarAuthMode::SharedPool);
}

#[test]
fn paper_detail_plan_sets_encoded_id_fields_and_auth_header() {
    let plan =
        SemanticScholarClient::paper_detail_plan("DOI:10.1056/NEJMoa1203421", Some(" test-key "))
            .unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "graph/v1/paper/DOI:10.1056%2FNEJMoa1203421");
    assert_eq!(plan.query_value("fields"), Some(GRAPH_PAPER_FIELDS));
    assert_eq!(plan.header_value("x-api-key"), Some("test-key"));
}

#[test]
fn paper_batch_plan_posts_ids_and_fields() {
    let ids = vec!["PMID:22663011".to_string(), "PMID:24200969".to_string()];
    let plan = SemanticScholarClient::paper_batch_plan(&ids, BATCH_PAPER_FIELDS, Some("test-key"))
        .unwrap();

    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "graph/v1/paper/batch");
    assert_eq!(plan.query_value("fields"), Some(BATCH_PAPER_FIELDS));
    assert_eq!(plan.header_value("x-api-key"), Some("test-key"));
    let RequestBody::Json(body) = &plan.body else {
        panic!("expected JSON body, got {:?}", plan.body);
    };
    assert_eq!(body["ids"], serde_json::json!(ids));
}

#[test]
fn paper_batch_plan_validates_id_count() {
    assert!(matches!(
        SemanticScholarClient::paper_batch_plan(&[], BATCH_PAPER_FIELDS, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    let too_many = (0..501)
        .map(|idx| format!("paper-{idx}"))
        .collect::<Vec<_>>();
    assert!(matches!(
        SemanticScholarClient::paper_batch_plan(&too_many, BATCH_PAPER_FIELDS, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn paper_search_plan_sets_query_limit_year_and_auth() {
    let plan = SemanticScholarClient::paper_search_plan(
        " braf melanoma ",
        3,
        Some("2000-2013"),
        Some("test-key"),
    )
    .unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "graph/v1/paper/search");
    assert_eq!(plan.query_value("query"), Some("braf melanoma"));
    assert_eq!(plan.query_value("fields"), Some(SEARCH_PAPER_FIELDS));
    assert_eq!(plan.query_value("limit"), Some("3"));
    assert_eq!(plan.query_value("year"), Some("2000-2013"));
    assert_eq!(plan.header_value("x-api-key"), Some("test-key"));
}

#[test]
fn paper_search_plan_validates_query_and_limit() {
    assert!(matches!(
        SemanticScholarClient::paper_search_plan("   ", 3, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::paper_search_plan("BRAF", 0, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::paper_search_plan("BRAF", 101, None, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[test]
fn author_search_plan_sets_query_fields_offset_limit_and_auth() {
    let plan =
        SemanticScholarClient::author_search_plan(" Atul Butte ", 25, 100, Some(" author-key "))
            .unwrap();

    assert_eq!(plan.method, HttpMethod::Get);
    assert_eq!(plan.path, "graph/v1/author/search");
    assert_eq!(plan.query_value("query"), Some("Atul Butte"));
    assert_eq!(plan.query_value("fields"), Some(AUTHOR_FIELDS));
    assert_eq!(plan.query_value("offset"), Some("25"));
    assert_eq!(plan.query_value("limit"), Some("100"));
    assert_eq!(plan.header_value("x-api-key"), Some("author-key"));
}

#[test]
fn author_detail_and_papers_plans_encode_ids_and_preserve_continuation() {
    let detail = SemanticScholarClient::author_detail_plan(" author/id ", None).unwrap();
    assert_eq!(detail.method, HttpMethod::Get);
    assert_eq!(detail.path, "graph/v1/author/author%2Fid");
    assert_eq!(detail.query_value("fields"), Some(AUTHOR_FIELDS));
    assert_eq!(detail.header_value("x-api-key"), None);

    let papers =
        SemanticScholarClient::author_papers_plan("author/id", 100, 1, Some("paper-key")).unwrap();
    assert_eq!(papers.method, HttpMethod::Get);
    assert_eq!(papers.path, "graph/v1/author/author%2Fid/papers");
    assert_eq!(papers.query_value("fields"), Some(AUTHOR_PAPER_FIELDS));
    assert_eq!(papers.query_value("offset"), Some("100"));
    assert_eq!(papers.query_value("limit"), Some("1"));
    assert_eq!(papers.header_value("x-api-key"), Some("paper-key"));
}

#[test]
fn author_id_dot_segments_are_rejected_before_request_construction() {
    for author_id in [".", ".."] {
        assert!(matches!(
            SemanticScholarClient::author_detail_plan(author_id, None),
            Err(BioMcpError::InvalidArgument(_))
        ));
        assert!(matches!(
            SemanticScholarClient::author_papers_plan(author_id, 0, 1, None),
            Err(BioMcpError::InvalidArgument(_))
        ));
        assert!(matches!(
            SemanticScholarClient::author_batch_plan(&[author_id.into()], None),
            Err(BioMcpError::InvalidArgument(_))
        ));
    }
}

#[test]
fn author_batch_plan_posts_ordered_ids_at_provider_ceiling() {
    let ids = (0..SEMANTIC_SCHOLAR_AUTHOR_BATCH_MAX)
        .map(|idx| format!("author-{idx}"))
        .collect::<Vec<_>>();
    let plan = SemanticScholarClient::author_batch_plan(&ids, Some("batch-key")).unwrap();

    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "graph/v1/author/batch");
    assert_eq!(plan.query_value("fields"), Some(AUTHOR_FIELDS));
    assert_eq!(plan.header_value("x-api-key"), Some("batch-key"));
    let RequestBody::Json(body) = &plan.body else {
        panic!("expected JSON body, got {:?}", plan.body);
    };
    assert_eq!(body["ids"], serde_json::json!(ids));
}

#[test]
fn author_plans_validate_required_input_and_endpoint_boundaries() {
    assert!(matches!(
        SemanticScholarClient::author_search_plan("   ", 0, 1, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(SemanticScholarClient::author_search_plan("Butte", 0, 1, None).is_ok());
    assert!(
        SemanticScholarClient::author_search_plan(
            "Butte",
            0,
            SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX,
            None,
        )
        .is_ok()
    );
    assert!(matches!(
        SemanticScholarClient::author_search_plan("Butte", 0, 0, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::author_search_plan(
            "Butte",
            0,
            SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX + 1,
            None,
        ),
        Err(BioMcpError::InvalidArgument(_))
    ));

    assert!(matches!(
        SemanticScholarClient::author_detail_plan(" ", None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    let too_long_id = "a".repeat(513);
    assert!(matches!(
        SemanticScholarClient::author_detail_plan(&too_long_id, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::author_papers_plan(" ", 0, 1, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::author_papers_plan(&too_long_id, 0, 1, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::author_papers_plan("author-1", 0, 0, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(
        SemanticScholarClient::author_papers_plan(
            "author-1",
            usize::MAX,
            SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX,
            None,
        )
        .is_ok()
    );
    assert!(matches!(
        SemanticScholarClient::author_papers_plan(
            "author-1",
            0,
            SEMANTIC_SCHOLAR_AUTHOR_PAGE_MAX + 1,
            None,
        ),
        Err(BioMcpError::InvalidArgument(_))
    ));

    assert!(matches!(
        SemanticScholarClient::author_batch_plan(&[], None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(SemanticScholarClient::author_batch_plan(&["author-1".into()], None).is_ok());
    let too_many = (0..=SEMANTIC_SCHOLAR_AUTHOR_BATCH_MAX)
        .map(|idx| format!("author-{idx}"))
        .collect::<Vec<_>>();
    assert!(matches!(
        SemanticScholarClient::author_batch_plan(&too_many, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::author_batch_plan(&[" ".into()], None),
        Err(BioMcpError::InvalidArgument(_))
    ));
    assert!(matches!(
        SemanticScholarClient::author_batch_plan(&[too_long_id], None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}

#[tokio::test]
async fn author_execution_methods_send_plans_and_decode_typed_responses() {
    let (client, server) = author_fixture_client(vec![
        r#"{"total":1,"offset":0,"next":null,"data":[{"authorId":"search-id"}]}"#,
        r#"{"authorId":"detail-id"}"#,
        r#"[{"authorId":"batch-id"}]"#,
        r#"{"offset":0,"next":null,"data":[{"paperId":"paper-id"}]}"#,
    ])
    .await;

    let search = client.author_search("Atul Butte", 0, 1).await.unwrap();
    assert_eq!(search.data[0].author_id.as_deref(), Some("search-id"));
    let detail = client.author_detail("detail-id").await.unwrap();
    assert_eq!(detail.author_id.as_deref(), Some("detail-id"));
    let batch = client.author_batch(&["batch-id".into()]).await.unwrap();
    assert_eq!(
        batch[0]
            .as_ref()
            .and_then(|author| author.author_id.as_deref()),
        Some("batch-id")
    );
    let papers = client.author_papers("detail-id", 0, 1).await.unwrap();
    assert_eq!(papers.data[0].paper_id.as_deref(), Some("paper-id"));

    let requests = tokio::time::timeout(std::time::Duration::from_secs(5), server)
        .await
        .expect("author methods reached fixture server")
        .expect("author fixture server");
    assert!(requests[0].starts_with("GET /graph/v1/author/search?"));
    assert!(requests[1].starts_with("GET /graph/v1/author/detail-id?"));
    assert!(requests[2].starts_with("POST /graph/v1/author/batch?"));
    assert!(requests[2].contains(r#"{"ids":["batch-id"]}"#));
    assert!(requests[3].starts_with("GET /graph/v1/author/detail-id/papers?"));
}

#[test]
fn legacy_search_request_plan_keeps_article_contract_shape() {
    let keyless = client_with_api_key(None);
    let keyless_plan: SemanticScholarPaperSearchRequestPlan = keyless
        .paper_search_request_plan("BRAF", 3, None)
        .expect("SemanticScholarPaperSearchRequestPlan");
    assert_eq!(keyless_plan.auth_mode, SemanticScholarAuthMode::SharedPool);
    assert!(keyless_plan.cache_mode.contains("shared_pool"));
    assert!(keyless_plan.status_expectation.contains("unavailable"));
    assert!(
        keyless_plan
            .query_params
            .contains(&("query", "BRAF".to_string()))
    );

    let authenticated = client_with_api_key(Some("s2-super-secret"));
    let auth_plan = authenticated
        .paper_search_request_plan("BRAF", 3, Some("2020-"))
        .expect("authenticated plan");
    assert_eq!(auth_plan.auth_mode, SemanticScholarAuthMode::Authenticated);
    assert!(
        auth_plan
            .query_params
            .contains(&("query", "BRAF".to_string()))
    );
    assert!(!format!("{:?}", auth_plan.query_params).contains("s2-super-secret"));
}

#[test]
fn citation_reference_and_recommendation_plans_set_paths() {
    let citation = SemanticScholarClient::paper_subresource_plan(
        "PMID:22663011",
        "citations",
        CITATION_EDGE_FIELDS,
        10,
        None,
    )
    .unwrap();
    assert_eq!(citation.path, "graph/v1/paper/PMID:22663011/citations");
    assert_eq!(citation.query_value("fields"), Some(CITATION_EDGE_FIELDS));
    assert_eq!(citation.query_value("limit"), Some("10"));
    assert_eq!(citation.header_value("x-api-key"), None);

    let reference = SemanticScholarClient::paper_subresource_plan(
        "PMID:22663011",
        "references",
        REFERENCE_EDGE_FIELDS,
        10,
        None,
    )
    .unwrap();
    assert_eq!(reference.path, "graph/v1/paper/PMID:22663011/references");
    assert_eq!(reference.header_value("x-api-key"), None);

    let for_paper =
        SemanticScholarClient::recommendations_for_paper_plan("paper-1", 2, Some("key")).unwrap();
    assert_eq!(for_paper.path, "recommendations/v1/papers/forpaper/paper-1");
    assert_eq!(for_paper.query_value("fields"), Some(RECOMMENDATION_FIELDS));
    assert_eq!(for_paper.header_value("x-api-key"), Some("key"));
}

#[test]
fn recommendations_plan_posts_positive_and_negative_ids() {
    let positives = vec!["paper-1".to_string()];
    let negatives = vec!["paper-2".to_string()];
    let plan =
        SemanticScholarClient::recommendations_plan(&positives, &negatives, 2, Some("test-key"))
            .unwrap();

    assert_eq!(plan.method, HttpMethod::Post);
    assert_eq!(plan.path, "recommendations/v1/papers/");
    assert_eq!(plan.query_value("fields"), Some(RECOMMENDATION_FIELDS));
    assert_eq!(plan.query_value("limit"), Some("2"));
    assert_eq!(plan.header_value("x-api-key"), Some("test-key"));
    let RequestBody::Json(body) = &plan.body else {
        panic!("expected JSON body, got {:?}", plan.body);
    };
    assert_eq!(body["positivePaperIds"], serde_json::json!(positives));
    assert_eq!(body["negativePaperIds"], serde_json::json!(negatives));

    assert!(matches!(
        SemanticScholarClient::recommendations_plan(&[], &negatives, 2, None),
        Err(BioMcpError::InvalidArgument(_))
    ));
}
