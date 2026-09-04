//! Article CLI filter, ranking, and debug tests.

use super::super::dispatch::{
    article_debug_filters, article_query_summary, article_search_request, build_article_debug_plan,
};
use super::super::truncate_article_annotations;
use crate::cli::PaginationMeta;

fn default_article_search_args() -> super::super::ArticleSearchArgs {
    super::super::ArticleSearchArgs {
        gene: None,
        disease: Vec::new(),
        drug: Vec::new(),
        author: Vec::new(),
        keyword: Vec::new(),
        positional_query: None,
        date_from: None,
        date_to: None,
        year_min: None,
        year_max: None,
        article_type: None,
        journal: Vec::new(),
        open_access: false,
        no_preprints: false,
        exclude_retracted: false,
        include_retracted: false,
        sort: "relevance".into(),
        ranking_mode: None,
        weight_semantic: None,
        weight_lexical: None,
        weight_citations: None,
        weight_position: None,
        source: "all".into(),
        max_per_source: None,
        session: None,
        limit: 10,
        offset: 0,
        debug_plan: false,
        full: false,
    }
}

#[test]
fn article_search_request_records_normalized_cli_intent_and_backend_plan() {
    let mut args = default_article_search_args();
    args.gene = Some("BRAF".into());
    args.disease = vec!["melanoma".into()];
    args.keyword = vec!["targeted therapy".into()];
    args.source = "pubmed".into();
    args.sort = "date".into();
    args.limit = 5;
    args.offset = 2;
    args.session = Some("caller-token".into());

    let request = article_search_request(args).expect("request");

    assert_eq!(request.filters.gene.as_deref(), Some("BRAF"));
    assert_eq!(request.filters.disease.as_deref(), Some("melanoma"));
    assert_eq!(request.filters.keyword.as_deref(), Some("targeted therapy"));
    assert_eq!(
        request.source_filter,
        crate::entities::article::ArticleSourceFilter::PubMed
    );
    assert_eq!(request.limit, 5);
    assert_eq!(request.offset, 2);
    assert_eq!(request.sort, crate::entities::article::ArticleSort::Date);
    assert_eq!(request.filters.sort, request.sort);
    assert_eq!(
        request.backend_plan,
        crate::entities::article::BackendPlan::PubMedOnly
    );
    assert!(request.exact_keyword_lookup.is_none());
}

#[test]
fn article_search_request_records_author_capable_plan() {
    let mut args = default_article_search_args();
    args.author = vec!["Williams".into(), "LS".into()];

    let request = article_search_request(args).expect("request");

    assert_eq!(request.filters.author.as_deref(), Some("Williams LS"));
    assert_eq!(
        request.backend_plan,
        crate::entities::article::BackendPlan::TypeCapable
    );
}

#[test]
fn article_search_request_rejects_native_keyword_field_syntax() {
    let mut args = default_article_search_args();
    args.keyword = vec!["Williams LS[Author]".into()];

    let err = article_search_request(args).expect_err("provider syntax should be rejected");

    assert!(err.to_string().contains("Use --author"));
}

#[test]
fn article_search_request_rejects_reserved_keyword_aliases_and_multiword_gene() {
    for (keyword, expected) in [
        (
            "gene:RB1",
            "Invalid argument: keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example \"gene\":\"RB1\".",
        ),
        (
            "disease:melanoma",
            "Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example \"disease\":\"melanoma\".",
        ),
        (
            "drug:vemurafenib",
            "Invalid argument: keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example \"drug\":\"vemurafenib\".",
        ),
    ] {
        for positional in [false, true] {
            let mut args = default_article_search_args();
            if positional {
                args.positional_query = Some(keyword.into());
            } else {
                args.keyword = vec![keyword.into()];
            }
            let err =
                article_search_request(args).expect_err("reserved syntax should fail planning");
            assert_eq!(err.to_string(), expected);
        }
    }

    let mut args = default_article_search_args();
    args.gene = Some("TPMT mercaptopurine".into());
    let err = article_search_request(args).expect_err("multiword gene should fail planning");
    assert_eq!(
        err.to_string(),
        "Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"]."
    );
}

#[test]
fn article_search_request_trims_valid_gene_before_planning() {
    let mut args = default_article_search_args();
    args.gene = Some("  BRAF  ".into());
    let request = article_search_request(args).expect("trimmed gene should plan");
    assert_eq!(request.filters.gene.as_deref(), Some("BRAF"));
}

#[test]
fn article_search_request_records_exact_keyword_lookup_intent() {
    let mut args = default_article_search_args();
    args.keyword = vec!["Gleevec".into()];

    let request = article_search_request(args).expect("request");

    assert_eq!(request.exact_keyword_lookup.as_deref(), Some("Gleevec"));
    assert_eq!(
        request.backend_plan,
        crate::entities::article::BackendPlan::Both
    );
}

#[test]
fn article_search_request_accepts_semantic_scholar_source() {
    let mut args = default_article_search_args();
    args.keyword = vec!["BRAF melanoma".into()];
    args.source = "semanticscholar".into();

    let request = article_search_request(args).expect("request");

    assert_eq!(
        request.source_filter,
        crate::entities::article::ArticleSourceFilter::SemanticScholar
    );
    assert_eq!(
        request.backend_plan,
        crate::entities::article::BackendPlan::SemanticScholarOnly
    );
}

#[test]
fn ticket_400_request_command_article_fields_drive_execution_boundaries() {
    let mut args = default_article_search_args();
    args.keyword = vec!["Gleevec".into()];
    args.source = "all".into();
    args.sort = "relevance".into();
    args.ranking_mode = Some("hybrid".into());
    args.weight_semantic = Some(0.4);
    args.weight_lexical = Some(0.3);
    args.weight_citations = Some(0.2);
    args.weight_position = Some(0.1);
    args.limit = 7;
    args.offset = 3;

    let request = article_search_request(args).expect("request");
    let summary = article_query_summary(
        &request.filters,
        request.source_filter,
        false,
        request.limit,
        request.offset,
    );
    let debug_filters =
        article_debug_filters(&request.filters, request.source_filter, request.limit);

    assert_eq!(request.exact_keyword_lookup.as_deref(), Some("Gleevec"));
    assert_eq!(
        request.backend_plan,
        crate::entities::article::BackendPlan::Both
    );
    assert_eq!(
        request.sort,
        crate::entities::article::ArticleSort::Relevance
    );
    assert_eq!(request.filters.sort, request.sort);
    assert!(summary.contains("keyword=Gleevec"));
    assert!(summary.contains("sort=relevance"));
    assert!(summary.contains("ranking_mode=hybrid"));
    assert!(debug_filters.iter().any(|entry| entry == "source=all"));
    assert!(
        debug_filters
            .iter()
            .any(|entry| entry == "ranking_mode=hybrid")
    );
}

#[test]
fn build_article_debug_plan_includes_article_type_limitation_note() {
    let filters = crate::entities::article::ArticleSearchFilters {
        gene: Some("BRAF".into()),
        gene_anchored: false,
        disease: None,
        drug: None,
        variant: None,
        author: None,
        keyword: None,
        date_from: None,
        date_to: None,
        article_type: Some("review".into()),
        journal: None,
        open_access: false,
        no_preprints: false,
        exclude_retracted: false,
        max_per_source: None,
        sort: crate::entities::article::ArticleSort::Relevance,
        ranking: crate::entities::article::ArticleRankingOptions::default(),
    };
    let pagination = PaginationMeta::offset(0, 3, 0, Some(0));

    let plan = build_article_debug_plan(
        "gene=BRAF, type=review",
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        3,
        &[],
        &pagination,
        &[],
    )
    .expect("debug plan should build");

    assert_eq!(plan.legs.len(), 1);
    assert!(
        plan.legs[0]
            .note
            .as_deref()
            .is_some_and(|value: &str| value.contains("Europe PMC and PubMed"))
    );
}

#[test]
fn related_article_filters_default_to_relevance_and_safety_flags() {
    let filters = super::super::super::related_article_filters();

    assert_eq!(
        filters.sort,
        crate::entities::article::ArticleSort::Relevance
    );
    assert!(!filters.open_access);
    assert!(filters.no_preprints);
    assert!(filters.exclude_retracted);
    assert_eq!(filters.max_per_source, None);
}

#[test]
fn article_query_and_debug_filters_include_effective_ranking_context() {
    let mut filters = super::super::super::related_article_filters();
    filters.keyword = Some("melanoma".into());
    filters.max_per_source = Some(10);

    let summary = article_query_summary(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        false,
        25,
        0,
    );
    assert!(summary.contains("ranking_mode=hybrid"));
    assert!(summary.contains("max_per_source=10"));
    assert!(summary.contains(
        "ranking_policy=hybrid relevance (score = 0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position)"
    ));

    let debug_filters = article_debug_filters(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        25,
    );
    assert!(
        debug_filters
            .iter()
            .any(|entry| entry == "ranking_mode=hybrid")
    );
    assert!(
        debug_filters
            .iter()
            .any(|entry| entry == "max_per_source=10")
    );
    assert!(debug_filters.iter().any(|entry| {
        entry
            == "ranking_policy=hybrid relevance (score = 0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position)"
    }));
}

#[test]
fn article_query_and_debug_filters_render_default_and_disabled_max_per_source_modes() {
    let mut filters = super::super::super::related_article_filters();
    filters.gene = Some("BRAF".into());
    filters.max_per_source = Some(0);

    let summary = article_query_summary(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        false,
        25,
        0,
    );
    assert!(summary.contains("max_per_source=default"));

    let debug_filters = article_debug_filters(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        25,
    );
    assert!(
        debug_filters
            .iter()
            .any(|entry| entry == "max_per_source=default")
    );

    filters.max_per_source = Some(25);
    let disabled_summary = article_query_summary(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        false,
        25,
        0,
    );
    assert!(disabled_summary.contains("max_per_source=disabled"));

    let disabled_debug_filters = article_debug_filters(
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        25,
    );
    assert!(
        disabled_debug_filters
            .iter()
            .any(|entry| entry == "max_per_source=disabled")
    );
}

#[test]
fn truncate_article_annotations_applies_limit_per_bucket() {
    let annotations = crate::entities::article::ArticleAnnotations {
        genes: vec![
            crate::entities::article::AnnotationCount {
                text: "BRAF".into(),
                count: 2,
            },
            crate::entities::article::AnnotationCount {
                text: "TP53".into(),
                count: 1,
            },
        ],
        diseases: vec![
            crate::entities::article::AnnotationCount {
                text: "melanoma".into(),
                count: 2,
            },
            crate::entities::article::AnnotationCount {
                text: "glioma".into(),
                count: 1,
            },
        ],
        chemicals: vec![
            crate::entities::article::AnnotationCount {
                text: "vemurafenib".into(),
                count: 1,
            },
            crate::entities::article::AnnotationCount {
                text: "dabrafenib".into(),
                count: 1,
            },
        ],
        mutations: vec![
            crate::entities::article::AnnotationCount {
                text: "V600E".into(),
                count: 1,
            },
            crate::entities::article::AnnotationCount {
                text: "L858R".into(),
                count: 1,
            },
        ],
    };
    let truncated = truncate_article_annotations(annotations, 1);
    assert_eq!(truncated.genes.len(), 1);
    assert_eq!(truncated.diseases.len(), 1);
    assert_eq!(truncated.chemicals.len(), 1);
    assert_eq!(truncated.mutations.len(), 1);
}
