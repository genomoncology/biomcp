//! Article CLI exact lookup and suggestion tests.
use clap::Parser;

use super::super::dispatch::{article_entity_suggestion, is_exact_article_keyword_lookup_eligible};
use super::super::{handle_command, handle_search};
use crate::cli::{Cli, Commands, SearchEntity};
use crate::entities::discover::{DiscoverType, ExactArticleKeywordEntity};
use crate::test_support::{env_lock, set_env_var};
use wiremock::matchers::{body_string_contains, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn exact_article_keyword_lookup_eligibility_is_keyword_only_and_short() {
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("BRAF".into());
    assert!(is_exact_article_keyword_lookup_eligible(&filters));

    filters.keyword = Some("non-small cell lung cancer".into());
    assert!(!is_exact_article_keyword_lookup_eligible(&filters));

    filters.keyword = Some("BRAF".into());
    filters.gene = Some("BRAF".into());
    assert!(!is_exact_article_keyword_lookup_eligible(&filters));

    filters.gene = None;
    filters.disease = Some("melanoma".into());
    assert!(!is_exact_article_keyword_lookup_eligible(&filters));

    filters.disease = None;
    filters.drug = Some("imatinib".into());
    assert!(!is_exact_article_keyword_lookup_eligible(&filters));
}

#[test]
fn article_entity_suggestion_uses_alias_reason_and_valid_sections() {
    let suggestion = article_entity_suggestion(&ExactArticleKeywordEntity {
        entity_type: DiscoverType::Drug,
        label: "imatinib mesylate".into(),
        primary_id: Some("CHEBI:45783".into()),
        matched_query: "Gleevec".into(),
        matched_alias: true,
    });

    assert_eq!(suggestion.command, "biomcp get drug \"imatinib mesylate\"");
    assert_eq!(
        suggestion.reason,
        "Exact drug alias match for article keyword \"Gleevec\"; suggested canonical drug \"imatinib mesylate\"."
    );
    assert_eq!(suggestion.sections, vec!["label", "targets", "indications"]);
}

#[tokio::test]
async fn handle_command_rejects_zero_limit_before_backend_lookup() {
    let cli = Cli::try_parse_from(["biomcp", "article", "citations", "22663011", "--limit", "0"])
        .expect("article citations should parse");

    let Cli {
        command: Commands::Article { cmd },
        json,
        ..
    } = cli
    else {
        panic!("expected article command");
    };

    let err = handle_command(cmd, json)
        .await
        .expect_err("zero article citations limit should fail fast");
    assert!(
        err.to_string()
            .contains("--limit must be between 1 and 100")
    );
}

#[tokio::test]
async fn handle_search_json_fails_open_when_exact_entity_lookup_errors() {
    let _guard = env_lock().lock().await;
    let pubtator = MockServer::start().await;
    let semantic_scholar = MockServer::start().await;
    let ols4 = MockServer::start().await;
    let _pubtator_base = set_env_var("BIOMCP_PUBTATOR_BASE", Some(&pubtator.uri()));
    let _s2_base = set_env_var("BIOMCP_S2_BASE", Some(&semantic_scholar.uri()));
    let _s2_key = set_env_var("S2_API_KEY", None);
    let _ols4_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols4.uri()));

    Mock::given(method("GET"))
        .and(path("/search/"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "results": [{
                "_id": "pt-1",
                "pmid": 22663011,
                "title": "BRAF melanoma review",
                "journal": "Journal",
                "date": "2025-01-01",
                "score": 42.0
            }],
            "count": 1,
            "total_pages": 1,
            "current": 1,
            "page_size": 25,
            "facets": {}
        })))
        .expect(1)
        .mount(&pubtator)
        .await;

    Mock::given(method("POST"))
        .and(path("/graph/v1/paper/batch"))
        .and(query_param(
            "fields",
            "paperId,externalIds,citationCount,influentialCitationCount,abstract",
        ))
        .and(body_string_contains("\"PMID:22663011\""))
        .respond_with(ResponseTemplate::new(429).set_body_string("shared rate limit"))
        .expect(1)
        .mount(&semantic_scholar)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/search"))
        .and(query_param("q", "BRAF"))
        .respond_with(ResponseTemplate::new(500).set_body_string("ols unavailable"))
        .mount(&ols4)
        .await;

    let cli = Cli::try_parse_from([
        "biomcp", "--json", "search", "article", "-k", "BRAF", "--source", "pubtator", "--sort",
        "date", "--limit", "1",
    ])
    .expect("article search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Article(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected article search command");
    };

    let outcome = handle_search(args, json)
        .await
        .expect("article search should fail open on OLS4 errors");
    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("json should parse successfully");
    let ols_requests = ols4
        .received_requests()
        .await
        .expect("OLS4 mock should record requests");
    assert!(
        !ols_requests.is_empty(),
        "article keyword lookup should call the failing OLS4 mock"
    );
    assert_eq!(value["count"], 1);
    assert!(
        value
            .get("_meta")
            .and_then(|meta| meta.get("suggestions"))
            .is_none()
    );
    assert!(
        !value["_meta"]["next_commands"]
            .as_array()
            .expect("next commands should be present")
            .iter()
            .any(|command| command.as_str() == Some("biomcp get gene BRAF"))
    );
}

#[tokio::test]
async fn handle_search_json_typed_filter_skips_exact_lookup_and_suggestions() {
    let _guard = env_lock().lock().await;
    let europepmc = MockServer::start().await;
    let ols4 = MockServer::start().await;
    let _europepmc_base = set_env_var("BIOMCP_EUROPEPMC_BASE", Some(&europepmc.uri()));
    let _ols4_base = set_env_var("BIOMCP_OLS4_BASE", Some(&ols4.uri()));

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("page", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "hitCount": 0,
            "resultList": {
                "result": []
            }
        })))
        .expect(1)
        .mount(&europepmc)
        .await;

    Mock::given(method("GET"))
        .and(path("/api/search"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": {
                "docs": []
            }
        })))
        .expect(0)
        .mount(&ols4)
        .await;

    let cli = Cli::try_parse_from([
        "biomcp",
        "--json",
        "search",
        "article",
        "-k",
        "BRAF",
        "-g",
        "BRAF",
        "--source",
        "europepmc",
        "--sort",
        "date",
        "--limit",
        "1",
    ])
    .expect("article search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Article(args),
        },
        json,
        ..
    } = cli
    else {
        panic!("expected article search command");
    };

    let outcome = handle_search(args, json)
        .await
        .expect("typed-filter article search should succeed");
    let value: serde_json::Value =
        serde_json::from_str(&outcome.text).expect("json should parse successfully");
    assert_eq!(value["count"], 0);
    assert!(
        value
            .get("_meta")
            .and_then(|meta| meta.get("suggestions"))
            .is_none()
    );
    assert!(!outcome.text.contains("biomcp get gene BRAF"));
}
