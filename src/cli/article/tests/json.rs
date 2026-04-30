//! Article CLI JSON and session integration tests.
use chrono::NaiveDate;
use clap::Parser;

use super::super::dispatch::{
    ArticleSearchJsonPage, ArticleSuggestion, article_query_summary, article_search_json,
};
use super::super::handle_search;
use super::help::parse_article_search;
use crate::cli::{Cli, Commands, PaginationMeta, SearchEntity};
use crate::test_support::{TempDirGuard, env_lock, set_env_var};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn article_search_json_includes_query_and_ranking_context() {
    let pagination = PaginationMeta::offset(0, 3, 1, Some(1));
    let mut filters = super::super::super::related_article_filters();
    filters.gene = Some("BRAF".into());
    let query = article_query_summary(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        false,
        3,
        0,
    );
    let results = vec![crate::entities::article::ArticleSearchResult {
        pmid: "22663011".into(),
        pmcid: Some("PMC9984800".into()),
        doi: Some("10.1056/NEJMoa1203421".into()),
        title: "BRAF melanoma review".into(),
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: Some(NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid date")),
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: crate::entities::article::ArticleSource::EuropePmc,
        matched_sources: vec![
            crate::entities::article::ArticleSource::EuropePmc,
            crate::entities::article::ArticleSource::SemanticScholar,
        ],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: Some(crate::entities::article::ArticleRankingMetadata {
            directness_tier: 3,
            anchor_count: 2,
            title_anchor_hits: 2,
            abstract_anchor_hits: 0,
            combined_anchor_hits: 2,
            all_anchors_in_title: true,
            all_anchors_in_text: true,
            study_or_review_cue: true,
            pubmed_rescue: false,
            pubmed_rescue_kind: None,
            pubmed_source_position: None,
            mode: Some(crate::entities::article::ArticleRankingMode::Lexical),
            semantic_score: None,
            lexical_score: None,
            citation_score: None,
            position_score: None,
            composite_score: None,
            avg_source_rank: None,
        }),
        normalized_title: "braf melanoma review".into(),
        normalized_abstract: "abstract".into(),
        publication_type: Some("Review".into()),
        source_local_position: 0,
    }];
    let next_commands = crate::render::markdown::search_next_commands_article(
        &results,
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );
    let related = crate::render::markdown::related_article_search_results(
        &results,
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );
    let json = article_search_json(
        &query,
        &filters,
        true,
        Some(
            "Note: --type restricts article search to Europe PMC and PubMed. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering.".into(),
        ),
        None,
        ArticleSearchJsonPage {
            results,
            pagination,
            next_commands,
            suggestions: Vec::new(),
        },
    )
    .expect("article search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert_eq!(value["query"], query);
    assert_eq!(value["sort"], "relevance");
    assert_eq!(value["semantic_scholar_enabled"], true);
    let source_status = value["_meta"]["source_status"]
        .as_array()
        .expect("article search JSON should expose source status metadata");
    assert!(source_status.iter().any(|status| {
        status.get("source") == Some(&serde_json::Value::String("semanticscholar".into()))
            && status.get("enabled") == Some(&serde_json::Value::Bool(true))
            && status.get("auth_mode") == Some(&serde_json::Value::String("shared_pool".into()))
            && matches!(
                status.get("status").and_then(serde_json::Value::as_str),
                Some("ok" | "degraded" | "unavailable")
            )
    }));
    assert_eq!(
        value["ranking_policy"],
        crate::entities::article::ARTICLE_RELEVANCE_RANKING_POLICY
    );
    assert_eq!(
        value["note"],
        "Note: --type restricts article search to Europe PMC and PubMed. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering."
    );
    assert_eq!(value["results"][0]["ranking"]["directness_tier"], 3);
    assert_eq!(value["results"][0]["ranking"]["pubmed_rescue"], false);
    assert!(value["results"][0]["ranking"]["pubmed_rescue_kind"].is_null());
    assert!(value["results"][0]["ranking"]["pubmed_source_position"].is_null());
    assert_eq!(
        value["results"][0]["matched_sources"][1],
        serde_json::Value::String("semanticscholar".into())
    );
    assert_eq!(value["results"][0]["first_index_date"], "2025-01-15");
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get article 22663011".into())
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands.contains(&serde_json::Value::String(
                "biomcp search article -g BRAF --year-min 2025 --year-max 2025 --limit 5".into()
            )))
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands
                .contains(&serde_json::Value::String("biomcp list article".into())))
    );
    assert!(
        value
            .get("_meta")
            .and_then(|meta| meta.get("suggestions"))
            .is_none()
    );
    assert!(!related.contains(&"biomcp list article".to_string()));
}

#[test]
fn article_search_json_emits_structured_exact_entity_suggestions() {
    let pagination = PaginationMeta::offset(0, 1, 1, Some(1));
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("BRAF".into());
    let query = article_query_summary(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        false,
        1,
        0,
    );
    let results = vec![crate::entities::article::ArticleSearchResult {
        pmid: "22663011".into(),
        pmcid: Some("PMC9984800".into()),
        doi: Some("10.1056/NEJMoa1203421".into()),
        title: "BRAF article".into(),
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: None,
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: crate::entities::article::ArticleSource::EuropePmc,
        matched_sources: vec![crate::entities::article::ArticleSource::EuropePmc],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: None,
        normalized_title: "braf article".into(),
        normalized_abstract: "abstract".into(),
        publication_type: None,
        source_local_position: 0,
    }];
    let suggestions = vec![ArticleSuggestion {
        command: "biomcp get gene BRAF".into(),
        reason: "Exact gene vocabulary match for article keyword \"BRAF\".".into(),
        sections: vec!["protein".into(), "diseases".into(), "expression".into()],
    }];
    let exact_commands = suggestions
        .iter()
        .map(|suggestion| suggestion.command.clone())
        .collect::<Vec<_>>();
    let next_commands = crate::render::markdown::search_next_commands_article(
        &results,
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        &exact_commands,
    );
    let related = crate::render::markdown::related_article_search_results(
        &results,
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        &exact_commands,
    );
    let json = article_search_json(
        &query,
        &filters,
        true,
        None,
        None,
        ArticleSearchJsonPage {
            results,
            pagination,
            next_commands,
            suggestions,
        },
    )
    .expect("article search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert!(value["results"][0].get("first_index_date").is_none());
    assert_eq!(
        value["_meta"]["next_commands"][0],
        serde_json::Value::String("biomcp get article 22663011".into())
    );
    assert_eq!(
        value["_meta"]["next_commands"][1],
        serde_json::Value::String("biomcp get gene BRAF".into())
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands.contains(&serde_json::Value::String(
                "biomcp search article -k BRAF --year-min 2025 --year-max 2025 --limit 5".into()
            )))
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands
                .contains(&serde_json::Value::String("biomcp list article".into())))
    );
    assert_eq!(
        value["_meta"]["suggestions"][0]["command"],
        serde_json::Value::String("biomcp get gene BRAF".into())
    );
    assert_eq!(
        value["_meta"]["suggestions"][0]["reason"],
        serde_json::Value::String(
            "Exact gene vocabulary match for article keyword \"BRAF\".".into()
        )
    );
    assert_eq!(
        value["_meta"]["suggestions"][0]["sections"],
        serde_json::json!(["protein", "diseases", "expression"])
    );
    assert!(related.contains(&"biomcp get gene BRAF".to_string()));
    assert!(
        !related
            .iter()
            .any(|command| command.contains("search article -g BRAF -k"))
    );
}

#[test]
fn article_search_json_allows_loop_suggestions_without_sections() {
    let pagination = PaginationMeta::offset(0, 1, 0, Some(0));
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("Oncotype DX DCIS study".into());
    let suggestions = vec![
        ArticleSuggestion {
            command: "biomcp get gene BRAF".into(),
            reason: "Exact gene vocabulary match for article keyword \"BRAF\".".into(),
            sections: vec!["protein".into(), "diseases".into(), "expression".into()],
        },
        ArticleSuggestion {
            command: "biomcp discover \"Oncotype DX DCIS study\"".into(),
            reason: "Map the topic to structured biomedical entities before searching again."
                .into(),
            sections: Vec::new(),
        },
    ];
    let json = article_search_json(
        "keyword=\"Oncotype DX DCIS study\"",
        &filters,
        false,
        None,
        None,
        ArticleSearchJsonPage {
            results: Vec::new(),
            pagination,
            next_commands: Vec::new(),
            suggestions,
        },
    )
    .expect("article search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert_eq!(
        value["_meta"]["suggestions"][0]["command"],
        serde_json::Value::String("biomcp get gene BRAF".into())
    );
    assert_eq!(
        value["_meta"]["suggestions"][0]["sections"],
        serde_json::json!(["protein", "diseases", "expression"])
    );
    assert_eq!(
        value["_meta"]["suggestions"][1]["command"],
        serde_json::Value::String("biomcp discover \"Oncotype DX DCIS study\"".into())
    );
    assert_eq!(
        value["_meta"]["suggestions"][1]["reason"],
        serde_json::Value::String(
            "Map the topic to structured biomedical entities before searching again.".into()
        )
    );
    assert!(
        value["_meta"]["suggestions"][1]
            .as_object()
            .is_some_and(|suggestion| !suggestion.contains_key("sections"))
    );
}

#[tokio::test]
async fn handle_search_json_appends_session_loop_suggestions_after_successful_overlap() {
    let _guard = env_lock().lock().await;
    let europepmc = MockServer::start().await;
    let cache_root = TempDirGuard::new("article-session-loop-dispatch");
    let cache_root_string = cache_root.path().to_string_lossy().into_owned();
    let _europepmc_base = set_env_var("BIOMCP_EUROPEPMC_BASE", Some(&europepmc.uri()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&cache_root_string));
    let _s2_key = set_env_var("S2_API_KEY", None);

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("page", "1"))
        .and(query_param("format", "json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "hitCount": 1,
            "resultList": {
                "result": [{
                    "id": "EP-1",
                    "pmid": "22663011",
                    "title": "Oncotype DX article",
                    "journalTitle": "Journal",
                    "firstPublicationDate": "2025-01-01",
                    "citedByCount": 25,
                    "pubType": "journal article"
                }]
            }
        })))
        .expect(2)
        .mount(&europepmc)
        .await;

    let first = Cli::try_parse_from([
        "biomcp",
        "--json",
        "search",
        "article",
        "-k",
        "Oncotype DX review paper",
        "--source",
        "europepmc",
        "--sort",
        "date",
        "--session",
        "lit-review-1",
        "--limit",
        "1",
    ])
    .expect("first article search should parse");
    let second = Cli::try_parse_from([
        "biomcp",
        "--json",
        "search",
        "article",
        "-k",
        "Oncotype DX DCIS study",
        "--source",
        "europepmc",
        "--sort",
        "date",
        "--session",
        "lit-review-1",
        "--limit",
        "1",
    ])
    .expect("second article search should parse");

    let Cli {
        command: Commands::Search {
            entity: SearchEntity::Article(first_args),
        },
        json,
        ..
    } = first
    else {
        panic!("expected first article search command");
    };
    let first_outcome = handle_search(first_args, json)
        .await
        .expect("first article search should succeed");
    let first_value: serde_json::Value =
        serde_json::from_str(&first_outcome.text).expect("first json should parse");
    assert!(
        first_value
            .get("_meta")
            .and_then(|meta| meta.get("suggestions"))
            .is_none()
    );

    let Cli {
        command:
            Commands::Search {
                entity: SearchEntity::Article(second_args),
            },
        json,
        ..
    } = second
    else {
        panic!("expected second article search command");
    };
    let second_outcome = handle_search(second_args, json)
        .await
        .expect("second article search should succeed");
    let second_value: serde_json::Value =
        serde_json::from_str(&second_outcome.text).expect("second json should parse");
    let suggestions = second_value["_meta"]["suggestions"]
        .as_array()
        .expect("overlap should emit suggestions");
    let commands = suggestions
        .iter()
        .filter_map(|suggestion| {
            suggestion
                .get("command")
                .and_then(serde_json::Value::as_str)
        })
        .collect::<Vec<_>>();

    assert_eq!(commands[0], "biomcp article batch 22663011");
    assert_eq!(commands[1], "biomcp discover \"Oncotype DX DCIS study\"");
    assert!(
        commands[2].contains("--year-min 2025 --year-max 2025"),
        "date retry should come from the current result page: {commands:#?}"
    );
    for command in commands.iter().copied() {
        let argv = shlex::split(command).unwrap_or_else(|| panic!("shlex failed on {command}"));
        Cli::try_parse_from(argv)
            .unwrap_or_else(|err| panic!("loop suggestion did not parse: {command}: {err}"));
    }
    assert!(suggestions.iter().all(|suggestion| {
        suggestion
            .as_object()
            .is_some_and(|object| !object.contains_key("sections"))
    }));
}

#[tokio::test]
async fn handle_search_session_baseline_is_markdown_independent_and_no_session_does_not_reset() {
    let _guard = env_lock().lock().await;
    let europepmc = MockServer::start().await;
    let cache_root = TempDirGuard::new("article-session-markdown-baseline");
    let cache_root_string = cache_root.path().to_string_lossy().into_owned();
    let _europepmc_base = set_env_var("BIOMCP_EUROPEPMC_BASE", Some(&europepmc.uri()));
    let _cache_dir = set_env_var("BIOMCP_CACHE_DIR", Some(&cache_root_string));
    let _s2_key = set_env_var("S2_API_KEY", None);

    Mock::given(method("GET"))
        .and(path("/search"))
        .and(query_param("page", "1"))
        .and(query_param("format", "json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "hitCount": 1,
            "resultList": {
                "result": [{
                    "id": "EP-1",
                    "pmid": "22663011",
                    "title": "Oncotype DX article",
                    "journalTitle": "Journal",
                    "firstPublicationDate": "2025-01-01",
                    "citedByCount": 25,
                    "pubType": "journal article"
                }]
            }
        })))
        .expect(3)
        .mount(&europepmc)
        .await;

    let (markdown_args, markdown_json) = parse_article_search([
        "biomcp",
        "search",
        "article",
        "-k",
        "Oncotype DX review paper",
        "--source",
        "europepmc",
        "--sort",
        "date",
        "--session",
        "lit-review-1",
        "--limit",
        "1",
    ]);
    assert!(!markdown_json);
    let markdown_outcome = handle_search(markdown_args, markdown_json)
        .await
        .expect("markdown article search should succeed");
    assert!(!markdown_outcome.text.contains("_meta"));
    assert!(
        !markdown_outcome
            .text
            .contains("Use the prior search's article set")
    );

    let (no_session_args, no_session_json) = parse_article_search([
        "biomcp",
        "--json",
        "search",
        "article",
        "-k",
        "Oncotype DX DCIS study",
        "--source",
        "europepmc",
        "--sort",
        "date",
        "--limit",
        "1",
    ]);
    let no_session_outcome = handle_search(no_session_args, no_session_json)
        .await
        .expect("no-session article search should succeed");
    let no_session_value: serde_json::Value =
        serde_json::from_str(&no_session_outcome.text).expect("no-session json should parse");
    assert!(
        no_session_value
            .get("_meta")
            .and_then(|meta| meta.get("suggestions"))
            .is_none(),
        "searches without --session should not emit loop-breaker suggestions"
    );

    let (session_args, session_json) = parse_article_search([
        "biomcp",
        "--json",
        "search",
        "article",
        "-k",
        "Oncotype DX DCIS study",
        "--source",
        "europepmc",
        "--sort",
        "date",
        "--session",
        "lit-review-1",
        "--limit",
        "1",
    ]);
    let session_outcome = handle_search(session_args, session_json)
        .await
        .expect("session article search should succeed");
    let session_value: serde_json::Value =
        serde_json::from_str(&session_outcome.text).expect("session json should parse");
    let commands = session_value["_meta"]["suggestions"]
        .as_array()
        .expect("session overlap should emit suggestions")
        .iter()
        .filter_map(|suggestion| {
            suggestion
                .get("command")
                .and_then(serde_json::Value::as_str)
        })
        .collect::<Vec<_>>();
    assert_eq!(commands[0], "biomcp article batch 22663011");
    assert_eq!(commands[1], "biomcp discover \"Oncotype DX DCIS study\"");
}

#[test]
fn article_search_json_next_commands_preserve_source_filter() {
    let pagination = PaginationMeta::offset(0, 1, 1, Some(1));
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("BRAF melanoma".into());
    let query = article_query_summary(
        &filters,
        crate::entities::article::ArticleSourceFilter::PubMed,
        false,
        1,
        0,
    );
    let results = vec![crate::entities::article::ArticleSearchResult {
        pmid: "22663011".into(),
        pmcid: None,
        doi: None,
        title: "BRAF melanoma review".into(),
        journal: Some("Journal".into()),
        date: Some("2013-05-12".into()),
        first_index_date: None,
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: crate::entities::article::ArticleSource::PubMed,
        matched_sources: vec![crate::entities::article::ArticleSource::PubMed],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".into()),
        ranking: None,
        normalized_title: "braf melanoma review".into(),
        normalized_abstract: "abstract".into(),
        publication_type: None,
        source_local_position: 0,
    }];
    let next_commands = crate::render::markdown::search_next_commands_article(
        &results,
        &filters,
        crate::entities::article::ArticleSourceFilter::PubMed,
        &[],
    );
    let json = article_search_json(
        &query,
        &filters,
        true,
        None,
        None,
        ArticleSearchJsonPage {
            results,
            pagination,
            next_commands,
            suggestions: Vec::new(),
        },
    )
    .expect("article search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|commands| commands.contains(&serde_json::Value::String(
                "biomcp search article -k \"BRAF melanoma\" --source pubmed --year-min 2013 --year-max 2013 --limit 5".into()
            )))
    );
}
