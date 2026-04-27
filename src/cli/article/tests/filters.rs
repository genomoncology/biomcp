//! Article CLI filter, ranking, and debug tests.

use super::super::dispatch::{
    article_debug_filters, article_query_summary, build_article_debug_plan,
    truncate_article_annotations,
};
use crate::cli::PaginationMeta;

#[test]
fn build_article_debug_plan_includes_article_type_limitation_note() {
    let filters = crate::entities::article::ArticleSearchFilters {
        gene: Some("BRAF".into()),
        gene_anchored: false,
        disease: None,
        drug: None,
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
