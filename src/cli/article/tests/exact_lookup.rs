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

#[test]
fn graph_offset_accepts_u64_boundaries_and_rejects_out_of_range_values() {
    for value in ["0", "18446744073709551615"] {
        assert!(
            Cli::try_parse_from([
                "biomcp",
                "article",
                "citations",
                "22663011",
                "--offset",
                value
            ])
            .is_ok()
        );
    }
    for value in ["-1", "18446744073709551616"] {
        assert!(
            Cli::try_parse_from([
                "biomcp",
                "article",
                "references",
                "22663011",
                "--offset",
                value
            ])
            .is_err()
        );
    }
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

#[test]
fn degraded_article_sources_share_safe_direct_retries_across_zero_row_surfaces() {
    use crate::entities::article::{
        ArticleSource, ArticleSourceAvailability, ArticleSourceFilter, ArticleSourceStatus,
    };
    let status = |source, status, message: &str| ArticleSourceStatus {
        source,
        enabled: true,
        auth_mode: None,
        status: Some(status),
        message: (!message.is_empty()).then(|| message.into()),
    };
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("BRAF ` $(touch nope); & melanoma".into());
    filters.author = Some("Doe, Jane & Roe".into());
    filters.date_from = Some("2020-01".into());
    filters.date_to = Some("2025".into());
    filters.article_type = Some("review".into());
    filters.journal = Some("Cancer & Cell".into());
    filters.no_preprints = false;
    filters.max_per_source = Some(2);
    let statuses = vec![
        status(
            ArticleSource::EuropePmc,
            ArticleSourceAvailability::Degraded,
            "timed out",
        ),
        status(
            ArticleSource::PubMed,
            ArticleSourceAvailability::Unavailable,
            "unavailable",
        ),
        status(
            ArticleSource::SemanticScholar,
            ArticleSourceAvailability::Degraded,
            "incompatible",
        ),
        status(ArticleSource::LitSense2, ArticleSourceAvailability::Ok, ""),
    ];
    let retries = crate::render::markdown::article_source_retry_commands(&filters, &statuses, 5, 7);
    assert_eq!(retries.len(), 2, "{retries:?}");
    assert!(retries[0].contains("--source europepmc"));
    assert!(retries[1].contains("--source pubmed"));
    for command in &retries {
        for expected in [
            "--keyword \"BRAF \\` \\$(touch nope); & melanoma\"",
            "--author \"Doe, Jane & Roe\"",
            "--date-from 2020-01 --date-to 2025",
            "--type review --journal \"Cancer & Cell\"",
            "--limit 5 --offset 7",
        ] {
            assert!(command.contains(expected), "{command}");
        }
        assert!(!command.contains("max-per-source"));
        let parsed = crate::cli::try_parse_cli(shlex::split(command).unwrap()).unwrap();
        assert!(matches!(
            parsed.command,
            crate::cli::Commands::Search { .. }
        ));
    }
    let markdown = crate::render::markdown::article_search_markdown_with_footer_and_context(
        "adversarial zero row",
        &[],
        "",
        &filters,
        crate::render::markdown::ArticleSearchRenderContext {
            source_filter: ArticleSourceFilter::All,
            semantic_scholar_enabled: false,
            warning: None,
            note: None,
            debug_plan: None,
            exact_entity_commands: &[],
            source_status: &statuses,
            retry_page: Some((5, 7)),
        },
    )
    .expect("zero-row Markdown");
    for expected in [
        "No articles found matching the filters.",
        "Europe PMC source status: degraded",
        "PubMed source status: unavailable",
        "Semantic Scholar source status: degraded",
    ] {
        assert!(markdown.contains(expected), "{markdown}");
    }
    assert_eq!(markdown.matches("Retry:").count(), 2);
    assert!(retries.iter().all(|command| markdown.contains(command)));
    let json = super::super::dispatch::article_search_json(
        "adversarial zero row",
        &filters,
        false,
        None,
        None,
        super::super::dispatch::ArticleSearchJsonPage {
            results: Vec::new(),
            pagination: crate::cli::PaginationMeta::offset(7, 5, 0, Some(7)),
            next_commands: retries.clone(),
            suggestions: Vec::new(),
            source_status: statuses,
        },
    )
    .expect("zero-row JSON");
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["_meta"]["next_commands"], serde_json::json!(retries));
}

#[test]
fn graph_markdown_is_exact_for_every_page_shape_and_direction() {
    use crate::entities::article::{
        ArticleGraphEdge, ArticleGraphMeta, ArticleGraphPagination, ArticleGraphResult,
        ArticleRelatedPaper, GraphCoverageStatus,
    };
    const ROW: &str =
        "| PMID 24200969 | Related paper | Background | yes | Important supporting context |\n";
    const EMPTY: &str = "| - | - | - | - | No related papers returned |\n";
    let paper = |pmid: &str, title: &str| ArticleRelatedPaper {
        paper_id: Some(format!("paper-{pmid}")),
        pmid: Some(pmid.into()),
        doi: None,
        arxiv_id: None,
        title: title.into(),
        journal: None,
        year: None,
    };
    let article = paper("22663011", "Seed");
    let edge = ArticleGraphEdge {
        paper: paper("24200969", "Related paper"),
        intents: vec!["Background".into()],
        contexts: vec!["Important supporting context".into()],
        is_influential: true,
    };
    let cases = [
        (0, 1, Some(1), GraphCoverageStatus::Continuable, false),
        (1, 1, Some(2), GraphCoverageStatus::Continuable, false),
        (58, 1, None, GraphCoverageStatus::Exhausted, false),
        (1000, 0, Some(1001), GraphCoverageStatus::Continuable, true),
        (1001, 0, None, GraphCoverageStatus::Exhausted, true),
    ];
    for (kind, direction) in [("Citations", "citations"), ("References", "references")] {
        for (offset, returned, next, coverage_status, empty) in cases {
            let command = next.map(|next| {
                format!("biomcp article {direction} 22663011 --limit 1 --offset {next}")
            });
            let result = ArticleGraphResult {
                article: article.clone(),
                edges: (!empty).then(|| edge.clone()).into_iter().collect(),
                pagination: ArticleGraphPagination {
                    offset,
                    limit: 1,
                    returned,
                    next_offset: next,
                    coverage_status,
                },
                _meta: ArticleGraphMeta {
                    next_commands: command.clone().into_iter().collect(),
                },
            };
            let status = coverage_status.as_str();
            let expected = format!(
                "# {kind} for PMID 22663011\n\n| Identifier | Title | Intents | Influential | Context |\n| --- | --- | --- | --- | --- |\n{}\nPage offset: {offset}; page size: 1; returned: {returned}; coverage: {status}.\nSemantic Scholar does not provide an exact total.\n{}",
                if empty { EMPTY } else { ROW },
                command.map_or_else(String::new, |value| format!("Next: `{value}`\n")),
            );
            assert_eq!(
                crate::render::markdown::article_graph_markdown(kind, &result).unwrap(),
                expected
            );
        }
    }
}

#[test]
fn graph_markdown_uses_a_safe_code_span_for_the_shared_command() {
    use crate::entities::article::{
        ArticleGraphMeta, ArticleGraphPagination, ArticleGraphResult, ArticleRelatedPaper,
        GraphCoverageStatus,
    };
    let mut result = ArticleGraphResult {
        article: ArticleRelatedPaper {
            paper_id: Some("paper-1".into()),
            pmid: Some("22663011".into()),
            doi: None,
            arxiv_id: None,
            title: "Seed".into(),
            journal: None,
            year: None,
        },
        edges: Vec::new(),
        pagination: ArticleGraphPagination {
            offset: 0,
            limit: 1,
            returned: 0,
            next_offset: Some(1),
            coverage_status: GraphCoverageStatus::Continuable,
        },
        _meta: ArticleGraphMeta {
            next_commands: vec![
                "biomcp article citations \"10.1/a`b\" --limit 1 --offset 1".into(),
            ],
        },
    };
    let command = result._meta.next_commands[0].clone();
    let markdown = crate::render::markdown::article_graph_markdown("Citations", &result).unwrap();
    assert!(
        markdown.contains(&format!("Next: ``{command}``")),
        "{markdown}"
    );
    result._meta.next_commands.clear();
    assert!(
        !crate::render::markdown::article_graph_markdown("Citations", &result)
            .unwrap()
            .contains("Next:")
    );
}
