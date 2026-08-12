#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;

#[test]
fn planner_routes_all_to_europepmc_for_strict_filters() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.open_access = true;

    let plan = plan_backends(&filters, ArticleSourceFilter::All).expect("planner should work");
    assert!(matches!(plan, BackendPlan::EuropeOnly));
}

#[test]
fn planner_routes_pubmed_to_pubmed_only() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    let plan = plan_backends(&filters, ArticleSourceFilter::PubMed).expect("planner should work");
    assert!(matches!(plan, BackendPlan::PubMedOnly));
}

#[test]
fn planner_routes_litsense2_to_litsense2_only() {
    let mut filters = empty_filters();
    filters.keyword = Some("Hirschsprung disease".into());

    let plan =
        plan_backends(&filters, ArticleSourceFilter::LitSense2).expect("planner should work");
    assert!(matches!(plan, BackendPlan::LitSense2Only));
}

#[test]
fn planner_routes_semantic_scholar_to_semantic_scholar_only() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF melanoma".into());

    let plan =
        plan_backends(&filters, ArticleSourceFilter::SemanticScholar).expect("planner should work");
    assert!(matches!(plan, BackendPlan::SemanticScholarOnly));
}

#[test]
fn planner_routes_all_with_author_to_author_capable_backends() {
    let mut filters = empty_filters();
    filters.author = Some("Williams LS".into());

    let plan = plan_backends(&filters, ArticleSourceFilter::All).expect("planner should work");
    assert_eq!(plan, BackendPlan::TypeCapable);
}

#[test]
fn planner_routes_all_with_author_and_no_preprints_to_europe_only() {
    let mut filters = empty_filters();
    filters.author = Some("Williams LS".into());
    filters.no_preprints = true;

    let plan = plan_backends(&filters, ArticleSourceFilter::All).expect("planner should work");
    assert_eq!(plan, BackendPlan::EuropeOnly);
}

#[test]
fn planner_rejects_author_on_incapable_explicit_sources() {
    for source in [
        ArticleSourceFilter::PubTator,
        ArticleSourceFilter::SemanticScholar,
        ArticleSourceFilter::LitSense2,
    ] {
        let mut filters = empty_filters();
        filters.author = Some("Williams LS".into());

        let err = plan_backends(&filters, source).expect_err("source should reject --author");
        let message = err.to_string();
        assert!(message.contains("--author"), "{source:?}: {message}");
        assert!(
            message.contains("--source europepmc"),
            "{source:?}: {message}"
        );
        assert!(message.contains("--source pubmed"), "{source:?}: {message}");
    }
}

#[test]
fn planner_routes_all_with_type_to_type_capable() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.article_type = Some("review".into());

    let plan = plan_backends(&filters, ArticleSourceFilter::All).expect("planner should work");
    assert!(matches!(plan, BackendPlan::TypeCapable));
}

#[test]
fn planner_routes_all_with_type_and_no_preprints_to_europe_only() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.article_type = Some("review".into());
    filters.no_preprints = true;

    let plan = plan_backends(&filters, ArticleSourceFilter::All).expect("planner should work");
    assert!(matches!(plan, BackendPlan::EuropeOnly));
}

#[test]
fn planner_rejects_pubtator_type_with_pubmed_compatible_filters_and_suggests_supported_routes() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.article_type = Some("review".into());

    let err = plan_backends(&filters, ArticleSourceFilter::PubTator)
        .expect_err("planner should reject strict-only filter on pubtator");
    let msg = err.to_string();
    assert!(msg.contains("--type"));
    assert!(msg.contains("--source europepmc"));
    assert!(msg.contains("--source pubmed"));
    assert!(msg.contains("remove --type"));
}

#[test]
fn planner_rejects_pubtator_open_access_without_suggesting_pubmed() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.open_access = true;

    let err = plan_backends(&filters, ArticleSourceFilter::PubTator)
        .expect_err("planner should reject open-access on pubtator");
    let msg = err.to_string();
    assert!(msg.contains("--open-access"));
    assert!(msg.contains("--source europepmc"));
    assert!(!msg.contains("--source pubmed"));
}

#[test]
fn planner_rejects_pubtator_type_and_no_preprints_without_suggesting_pubmed() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.article_type = Some("review".into());
    filters.no_preprints = true;

    let err = plan_backends(&filters, ArticleSourceFilter::PubTator)
        .expect_err("planner should reject incompatible type/no-preprints mix");
    let msg = err.to_string();
    assert!(msg.contains("--source europepmc"));
    assert!(msg.contains("--no-preprints"));
    assert!(!msg.contains("--source pubmed"));
}

#[test]
fn planner_rejects_litsense2_without_keyword() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    let err = plan_backends(&filters, ArticleSourceFilter::LitSense2)
        .expect_err("planner should reject keyword-less litsense2 source");
    let msg = err.to_string();
    assert!(msg.contains("--source litsense2"));
    assert!(msg.contains("keyword"));
}

#[test]
fn planner_rejects_semantic_scholar_type_filter() {
    let mut filters = empty_filters();
    filters.keyword = Some("melanoma".into());
    filters.article_type = Some("review".into());

    let err = plan_backends(&filters, ArticleSourceFilter::SemanticScholar)
        .expect_err("planner should reject --type for semanticscholar");
    let msg = err.to_string();
    assert!(msg.contains("--source semanticscholar"));
    assert!(msg.contains("--type"));
}

#[test]
fn planner_rejects_semantic_scholar_open_access_filter() {
    let mut filters = empty_filters();
    filters.keyword = Some("melanoma".into());
    filters.open_access = true;

    let err = plan_backends(&filters, ArticleSourceFilter::SemanticScholar)
        .expect_err("planner should reject --open-access for semanticscholar");
    let msg = err.to_string();
    assert!(msg.contains("--source semanticscholar"));
    assert!(msg.contains("--open-access"));
}

#[test]
fn planner_rejects_litsense2_type_filter() {
    let mut filters = empty_filters();
    filters.keyword = Some("melanoma".into());
    filters.article_type = Some("review".into());

    let err = plan_backends(&filters, ArticleSourceFilter::LitSense2)
        .expect_err("planner should reject --type for litsense2");
    let msg = err.to_string();
    assert!(msg.contains("--source litsense2"));
    assert!(msg.contains("--type"));
}

#[test]
fn planner_rejects_litsense2_open_access_filter() {
    let mut filters = empty_filters();
    filters.keyword = Some("melanoma".into());
    filters.open_access = true;

    let err = plan_backends(&filters, ArticleSourceFilter::LitSense2)
        .expect_err("planner should reject --open-access for litsense2");
    let msg = err.to_string();
    assert!(msg.contains("--source litsense2"));
    assert!(msg.contains("--open-access"));
}

#[test]
fn article_type_limitation_note_tracks_compatible_source_sets() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.article_type = Some("review".into());

    assert_eq!(
        article_type_limitation_note(&filters, ArticleSourceFilter::All),
        Some(
            "Note: --type restricts article search to Europe PMC and PubMed. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering."
                .into()
        )
    );
    assert_eq!(
        article_type_limitation_note(&filters, ArticleSourceFilter::EuropePmc),
        None
    );
    assert_eq!(
        article_type_limitation_note(&filters, ArticleSourceFilter::PubTator),
        None
    );
    assert_eq!(
        article_type_limitation_note(&filters, ArticleSourceFilter::PubMed),
        None
    );

    filters.no_preprints = true;
    assert_eq!(
        article_type_limitation_note(&filters, ArticleSourceFilter::All),
        Some(
            "Note: --type restricts this article search to Europe PMC. PubTator3, LitSense2, and Semantic Scholar do not support publication-type filtering, and PubMed does not support the other selected filters."
                .into()
        )
    );
}

#[test]
fn semantic_scholar_search_is_enabled_for_federated_queries() {
    assert!(semantic_scholar_search_enabled(
        &empty_filters(),
        ArticleSourceFilter::All
    ));
}

#[test]
fn semantic_scholar_search_is_disabled_for_author_filter() {
    let mut filters = empty_filters();
    filters.author = Some("Williams LS".into());

    assert!(!semantic_scholar_search_enabled(
        &filters,
        ArticleSourceFilter::All
    ));
}

#[test]
fn semantic_scholar_search_is_enabled_for_explicit_semantic_scholar_source() {
    assert!(semantic_scholar_search_enabled(
        &empty_filters(),
        ArticleSourceFilter::SemanticScholar
    ));
}

#[test]
fn summarize_debug_plan_reports_only_author_capable_sources() {
    let mut filters = empty_filters();
    filters.author = Some("Williams LS".into());

    let summary = summarize_debug_plan(&filters, ArticleSourceFilter::All, &[]).expect("summary");

    assert_eq!(summary.routing, vec!["planner=type_capable".to_string()]);
    assert_eq!(
        summary.sources,
        vec!["Europe PMC".to_string(), "PubMed".to_string()]
    );
}

#[test]
fn summarize_debug_plan_reports_federated_sources_and_matches() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    let results = vec![ArticleSearchResult {
        pmid: "22663011".into(),
        pmcid: None,
        doi: None,
        arxiv_id: None,
        semantic_scholar_id: None,
        title: "BRAF melanoma".into(),
        journal: None,
        date: None,
        first_index_date: None,
        citation_count: None,
        influential_citation_count: None,
        source: ArticleSource::PubMed,
        matched_sources: vec![
            ArticleSource::PubTator,
            ArticleSource::PubMed,
            ArticleSource::SemanticScholar,
        ],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: None,
        ranking: None,
        normalized_title: "braf melanoma".into(),
        normalized_abstract: String::new(),
        publication_type: None,
        source_local_position: 0,
    }];

    let summary =
        summarize_debug_plan(&filters, ArticleSourceFilter::All, &results).expect("summary");

    assert_eq!(summary.routing, vec!["planner=federated".to_string()]);
    assert!(summary.sources.contains(&"PubTator3".to_string()));
    assert!(summary.sources.contains(&"Europe PMC".to_string()));
    assert!(summary.sources.contains(&"PubMed".to_string()));
    assert_eq!(
        summary.matched_sources,
        vec![
            "PubTator3".to_string(),
            "PubMed".to_string(),
            "Semantic Scholar".to_string()
        ]
    );
}

#[test]
fn summarize_debug_plan_strict_filter_emits_europe_only_strict() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.open_access = true;

    let summary = summarize_debug_plan(&filters, ArticleSourceFilter::All, &[]).expect("summary");

    assert_eq!(
        summary.routing,
        vec!["planner=europe_only_strict_filters".to_string()]
    );
    assert_eq!(summary.sources, vec!["Europe PMC".to_string()]);
    assert!(summary.matched_sources.is_empty());
}

#[test]
fn summarize_debug_plan_explicit_pubtator_emits_pubtator_only() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    let summary =
        summarize_debug_plan(&filters, ArticleSourceFilter::PubTator, &[]).expect("summary");

    assert_eq!(summary.routing, vec!["planner=pubtator_only".to_string()]);
    assert_eq!(summary.sources, vec!["PubTator3".to_string()]);
    assert!(summary.matched_sources.is_empty());
}

#[test]
fn summarize_debug_plan_no_preprints_omits_pubmed() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.no_preprints = true;

    let summary = summarize_debug_plan(&filters, ArticleSourceFilter::All, &[]).expect("summary");

    assert_eq!(summary.routing, vec!["planner=federated".to_string()]);
    assert!(summary.sources.contains(&"PubTator3".to_string()));
    assert!(summary.sources.contains(&"Europe PMC".to_string()));
    assert!(
        !summary.sources.contains(&"PubMed".to_string()),
        "PubMed should be excluded when no_preprints is set"
    );
}

#[test]
fn summarize_debug_plan_keyword_all_excludes_litsense2() {
    let mut filters = empty_filters();
    filters.keyword = Some("Hirschsprung disease".into());

    let summary = summarize_debug_plan(&filters, ArticleSourceFilter::All, &[]).expect("summary");

    assert_eq!(summary.routing, vec!["planner=federated".to_string()]);
    assert_eq!(
        summary.sources,
        vec![
            "PubTator3".to_string(),
            "Europe PMC".to_string(),
            "PubMed".to_string(),
            "Semantic Scholar".to_string(),
        ]
    );
}

#[test]
fn summarize_debug_plan_explicit_litsense2_remains_selectable() {
    let mut filters = empty_filters();
    filters.keyword = Some("Hirschsprung disease".into());

    let summary =
        summarize_debug_plan(&filters, ArticleSourceFilter::LitSense2, &[]).expect("summary");

    assert_eq!(summary.routing, vec!["planner=litsense2_only".to_string()]);
    assert_eq!(summary.sources, vec!["LitSense2".to_string()]);
}

#[test]
fn summarize_debug_plan_explicit_semantic_scholar_remains_selectable() {
    let mut filters = empty_filters();
    filters.keyword = Some("Hirschsprung disease".into());

    let summary =
        summarize_debug_plan(&filters, ArticleSourceFilter::SemanticScholar, &[]).expect("summary");

    assert_eq!(
        summary.routing,
        vec!["planner=semanticscholar_only".to_string()]
    );
    assert_eq!(summary.sources, vec!["Semantic Scholar".to_string()]);
}

#[test]
fn litsense2_search_enabled_only_for_explicit_litsense2_source() {
    let mut keyword_filters = empty_filters();
    keyword_filters.keyword = Some("Hirschsprung disease".into());
    assert!(!litsense2_search_enabled(
        &keyword_filters,
        ArticleSourceFilter::All
    ));
    assert!(litsense2_search_enabled(
        &keyword_filters,
        ArticleSourceFilter::LitSense2
    ));
}

#[test]
fn explicit_article_sources_have_no_cross_provider_enrichment() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF melanoma".into());
    for source in [
        ArticleSourceFilter::PubMed,
        ArticleSourceFilter::PubTator,
        ArticleSourceFilter::EuropePmc,
        ArticleSourceFilter::SemanticScholar,
        ArticleSourceFilter::LitSense2,
    ] {
        let plan = article_source_plan(&filters, source).unwrap();
        assert_eq!(plan.candidate_sources.len(), 1, "source={source:?}");
        assert!(plan.enrichment_sources.is_empty(), "source={source:?}");
    }
}
