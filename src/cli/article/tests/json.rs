//! Article CLI JSON and session integration tests.
use chrono::NaiveDate;

use super::super::ArticleSearchDetail;
use super::super::dispatch::{
    ArticleSearchJsonPage, ArticleSuggestion, article_query_summary, article_search_json,
    article_search_json_with_detail, article_session_suggestions,
};
use crate::cli::PaginationMeta;
use crate::test_support::TempDirGuard;

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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            source_status: vec![crate::entities::article::ArticleSourceStatus {
                source: crate::entities::article::ArticleSource::SemanticScholar,
                enabled: true,
                auth_mode: Some(
                    crate::sources::semantic_scholar::SemanticScholarAuthMode::SharedPool,
                ),
                status: Some(crate::entities::article::ArticleSourceAvailability::Ok),
                message: None,
            }],
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            source_status: Vec::new(),
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
            source_status: Vec::new(),
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

fn session_article_result(
    pmid: &str,
    title: &str,
) -> crate::entities::article::ArticleSearchResult {
    crate::entities::article::ArticleSearchResult {
        pmid: pmid.into(),
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: title.into(),
        journal: Some("Journal".into()),
        date: Some("2025-01-01".into()),
        first_index_date: Some(NaiveDate::from_ymd_opt(2025, 1, 15).expect("valid date")),
        citation_count: Some(25),
        influential_citation_count: None,
        source: crate::entities::article::ArticleSource::EuropePmc,
        matched_sources: vec![crate::entities::article::ArticleSource::EuropePmc],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: None,
        ranking: None,
        normalized_title: title.to_ascii_lowercase(),
        normalized_abstract: String::new(),
        publication_type: Some("journal article".into()),
        source_local_position: 0,
    }
}

#[test]
fn article_session_suggestions_flow_into_search_json_after_overlap() {
    let cache_root = TempDirGuard::new("article-session-loop-dispatch");
    let previous_results = vec![session_article_result("22663011", "Oncotype DX article")];
    let current_results = vec![session_article_result(
        "39073865",
        "Oncotype DX DCIS article",
    )];
    let mut previous_filters = super::super::super::related_article_filters();
    previous_filters.keyword = Some("Oncotype DX review paper".into());
    let mut current_filters = super::super::super::related_article_filters();
    current_filters.keyword = Some("Oncotype DX DCIS study".into());
    let next_commands = vec![
        "biomcp get article 39073865".to_string(),
        "biomcp search article -k \"Oncotype DX DCIS study\" --year-min 2025 --year-max 2025 --limit 5".to_string(),
        "biomcp list article".to_string(),
    ];

    let first = article_session_suggestions(
        cache_root.path(),
        "lit-review-1",
        &previous_filters,
        &previous_results,
        &[],
        1_000,
    );
    assert!(
        first.is_empty(),
        "first search should only record the session baseline"
    );

    let suggestions = article_session_suggestions(
        cache_root.path(),
        "lit-review-1",
        &current_filters,
        &current_results,
        &next_commands,
        1_030,
    );
    let query = article_query_summary(
        &current_filters,
        crate::entities::article::ArticleSourceFilter::EuropePmc,
        false,
        1,
        0,
    );
    let json = article_search_json(
        &query,
        &current_filters,
        false,
        None,
        None,
        ArticleSearchJsonPage {
            results: current_results,
            pagination: PaginationMeta::offset(0, 1, 1, Some(1)),
            next_commands,
            suggestions,
            source_status: Vec::new(),
        },
    )
    .expect("article search json should render");

    let value: serde_json::Value =
        serde_json::from_str(&json).expect("json should parse successfully");
    let commands = value["_meta"]["suggestions"]
        .as_array()
        .expect("overlap should emit suggestions")
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
    assert!(
        value["_meta"]["suggestions"]
            .as_array()
            .expect("suggestions")
            .iter()
            .all(|suggestion| suggestion
                .as_object()
                .is_some_and(|object| !object.contains_key("sections")))
    );
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
        arxiv_id: None,
        semantic_scholar_id: None,
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
            source_status: Vec::new(),
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

#[test]
fn compact_article_search_rows_preserve_triage_fields_and_date_warnings() {
    let mut filters = super::super::super::related_article_filters();
    filters.sort = crate::entities::article::ArticleSort::Date;
    let mut row = session_article_result("", "Provider-only article");
    row.pmcid = Some("PMC123".into());
    row.doi = Some("10.1000/example".into());
    row.arxiv_id = Some("2401.12345".into());
    row.semantic_scholar_id = Some("paper-1".into());
    row.abstract_snippet = Some("A deliberately expensive abstract snippet".into());
    row.matched_sources
        .push(crate::entities::article::ArticleSource::SemanticScholar);
    let page = || ArticleSearchJsonPage {
        results: vec![row.clone()],
        pagination: PaginationMeta::offset(0, 1, 1, Some(1)),
        next_commands: vec!["biomcp get article PMC123".into()],
        suggestions: Vec::new(),
        source_status: Vec::new(),
    };

    let compact = article_search_json_with_detail(
        "sort=date",
        &filters,
        ArticleSearchDetail::Compact,
        true,
        None,
        None,
        page(),
    )
    .expect("compact JSON");
    let full = article_search_json_with_detail(
        "sort=date",
        &filters,
        ArticleSearchDetail::Full,
        true,
        None,
        None,
        page(),
    )
    .expect("full JSON");
    let value: serde_json::Value = serde_json::from_str(&compact).expect("compact value");
    let result = &value["results"][0];

    assert!(result.get("pmid").is_none());
    assert_eq!(result["pmcid"], "PMC123");
    assert_eq!(result["doi"], "10.1000/example");
    assert_eq!(result["arxiv_id"], "2401.12345");
    assert_eq!(result["semantic_scholar_id"], "paper-1");
    assert!(result["is_retracted"].is_boolean());
    for omitted in [
        "first_index_date",
        "influential_citation_count",
        "matched_sources",
        "score",
        "abstract_snippet",
        "ranking",
    ] {
        assert!(
            result.get(omitted).is_none(),
            "compact row retained {omitted}"
        );
    }
    assert_eq!(
        value["_meta"]["warnings"][0]["code"],
        "date_sort_replaces_relevance"
    );
    assert!(
        value["_meta"]["warnings"][0]["message"]
            .as_str()
            .is_some_and(|message| message.contains("replaces relevance ranking"))
    );
    assert!(full.contains("abstract_snippet"));
    assert!(compact.len() < full.len());
}

#[test]
fn date_warning_alone_retains_article_search_metadata() {
    let mut filters = super::super::super::related_article_filters();
    filters.sort = crate::entities::article::ArticleSort::Date;
    let json = article_search_json_with_detail(
        "sort=date",
        &filters,
        ArticleSearchDetail::Compact,
        false,
        None,
        None,
        ArticleSearchJsonPage {
            results: Vec::new(),
            pagination: PaginationMeta::offset(0, 1, 0, Some(0)),
            next_commands: Vec::new(),
            suggestions: Vec::new(),
            source_status: Vec::new(),
        },
    )
    .expect("date warning JSON");
    let value: serde_json::Value = serde_json::from_str(&json).expect("date warning value");
    assert_eq!(
        value["_meta"]["warnings"][0]["code"],
        "date_sort_replaces_relevance"
    );
}

#[test]
fn ticket_377_article_renderer_envelope_contracts_json_meta() {
    let json = article_search_json(
        "BRAF melanoma",
        &super::super::super::related_article_filters(),
        true,
        None,
        None,
        ArticleSearchJsonPage {
            results: Vec::new(),
            pagination: PaginationMeta::offset(0, 1, 0, Some(0)),
            next_commands: vec!["biomcp get article 22663011".to_string()],
            suggestions: Vec::new(),
            source_status: vec![crate::entities::article::ArticleSourceStatus {
                source: crate::entities::article::ArticleSource::EuropePmc,
                enabled: true,
                auth_mode: None,
                status: Some(crate::entities::article::ArticleSourceAvailability::Degraded),
                message: Some("Europe PMC timed out after 12s".to_string()),
            }],
        },
    )
    .expect("article_search_json");
    let value: serde_json::Value = serde_json::from_str(&json).expect("valid article JSON");
    assert_eq!(
        value["_meta"]["next_commands"][0],
        "biomcp get article 22663011"
    );
    assert_eq!(value["_meta"]["source_status"][0]["source"], "europepmc");
    assert_eq!(value["_meta"]["source_status"][0]["status"], "degraded");
    assert_eq!(
        value["_meta"]["source_status"][0]["message"],
        "Europe PMC timed out after 12s"
    );
}
