//! Article CLI exact lookup and suggestion tests.
use clap::Parser;

use super::super::dispatch::{
    ArticleSearchJsonPage, article_entity_suggestion, article_search_json, article_search_request,
    is_exact_article_keyword_lookup_eligible,
};
use super::super::handle_command;
use crate::cli::{Cli, Commands, SearchEntity};
use crate::entities::discover::{DiscoverType, ExactArticleKeywordEntity};

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

fn article_result() -> crate::entities::article::ArticleSearchResult {
    crate::entities::article::ArticleSearchResult {
        pmid: "22663011".into(),
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: "BRAF melanoma review".into(),
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: crate::entities::article::ArticleSource::PubTator,
        matched_sources: vec![crate::entities::article::ArticleSource::PubTator],
        score: Some(42.0),
        is_retracted: Some(false),
        abstract_snippet: None,
        ranking: None,
        normalized_title: "braf melanoma review".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }
}

#[test]
fn article_search_json_fails_open_when_exact_entity_lookup_returns_none() {
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
    assert!(json);

    let request = article_search_request(args).expect("article search request");
    assert_eq!(request.exact_keyword_lookup.as_deref(), Some("BRAF"));
    let results = vec![article_result()];
    let pagination = crate::cli::PaginationMeta::offset(0, 1, results.len(), Some(1));
    let json = article_search_json(
        "keyword=BRAF, sort=date, source=pubtator",
        &request.filters,
        false,
        None,
        None,
        ArticleSearchJsonPage {
            results,
            pagination,
            next_commands: vec!["biomcp get article 22663011".into()],
            suggestions: Vec::new(),
            source_status: Vec::new(),
        },
    )
    .expect("article search json should render");
    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
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

#[test]
fn article_search_request_typed_filter_skips_exact_lookup() {
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
    assert!(json);

    let request = article_search_request(args).expect("article search request");
    assert_eq!(request.filters.keyword.as_deref(), Some("BRAF"));
    assert_eq!(request.filters.gene.as_deref(), Some("BRAF"));
    assert!(request.exact_keyword_lookup.is_none());
}
