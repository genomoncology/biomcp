fn article_search_result(pmid: &str) -> ArticleSearchResult {
    ArticleSearchResult {
        pmid: pmid.to_string(),
        pmcid: None,
        doi: None,
        title: "Entity-aware article".to_string(),
        journal: Some("Journal".to_string()),
        date: Some("2025-01-01".to_string()),
        first_index_date: None,
        citation_count: Some(12),
        influential_citation_count: Some(4),
        source: ArticleSource::EuropePmc,
        matched_sources: vec![ArticleSource::EuropePmc],
        score: None,
        is_retracted: Some(false),
        abstract_snippet: Some("Abstract".to_string()),
        ranking: None,
        normalized_title: "entity-aware article".to_string(),
        normalized_abstract: "abstract".to_string(),
        publication_type: None,
        source_local_position: 0,
    }
}

fn article_filters(
    keyword: Option<&str>,
    gene: Option<&str>,
    drug: Option<&str>,
) -> crate::entities::article::ArticleSearchFilters {
    crate::entities::article::ArticleSearchFilters {
        gene: gene.map(str::to_string),
        gene_anchored: false,
        disease: None,
        drug: drug.map(str::to_string),
        variant: None,
        author: None,
        keyword: keyword.map(str::to_string),
        date_from: None,
        date_to: None,
        article_type: None,
        journal: None,
        open_access: false,
        no_preprints: true,
        exclude_retracted: true,
        max_per_source: None,
        sort: crate::entities::article::ArticleSort::Relevance,
        ranking: crate::entities::article::ArticleRankingOptions::default(),
    }
}

#[test]
fn article_search_related_results_include_primary_article_and_exact_commands() {
    let exact_commands = vec!["biomcp get gene BRAF".to_string()];
    let related = related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("BRAF"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &exact_commands,
    );

    assert_eq!(related[0], "biomcp get article 22663011");
    assert_eq!(related[1], "biomcp get gene BRAF");
}

#[test]
fn article_search_related_results_do_not_derive_entity_get_from_keyword_tokens() {
    let related = related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("SRY Sox9 miRNA"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert_eq!(related[0], "biomcp get article 22663011");
    assert!(!related.iter().any(|command| command == "biomcp get gene SRY"));
    assert!(
        !related
            .iter()
            .any(|command| command == "biomcp search article -g SRY -k \"Sox9 miRNA\"")
    );
}

#[test]
fn article_search_related_results_do_not_emit_braf_v600e_gene_get_without_exact_command() {
    let related = related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("BRAF V600E"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert_eq!(related[0], "biomcp get article 22663011");
    assert!(!related.iter().any(|command| command == "biomcp get gene BRAF"));
    assert!(
        !related
            .iter()
            .any(|command| command.starts_with("biomcp search article -g BRAF"))
    );
}

#[test]
fn article_search_related_results_include_exact_commands_without_result_rows() {
    let exact_commands = vec!["biomcp get disease melanoma".to_string()];
    let related = related_article_search_results(
        &[],
        &article_filters(Some("melanoma"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &exact_commands,
    );

    assert_eq!(related, exact_commands);
}

#[test]
fn article_search_next_commands_adds_list_for_exact_commands_without_result_rows() {
    let exact_commands = vec!["biomcp get gene BRAF".to_string()];
    let next = search_next_commands_article(
        &[],
        &article_filters(Some("BRAF"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &exact_commands,
    );

    assert_eq!(
        next,
        vec![
            "biomcp get gene BRAF".to_string(),
            "biomcp list article".to_string()
        ]
    );
}

#[test]
fn article_search_related_results_skip_non_entity_keyword_hints() {
    let related = related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("cell cycle checkpoint"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert_eq!(related[0], "biomcp get article 22663011");
    assert_eq!(
        related[1],
        "biomcp search article -k \"cell cycle checkpoint\" --year-min 2025 --year-max 2025 --limit 5"
    );
}

#[test]
fn markdown_article_search_related_results_include_discover_for_short_vaccine_phrase() {
    let related = markdown_related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("live attenuated vaccines"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(related.contains(&"biomcp discover \"live attenuated vaccines\"".to_string()));
}

#[test]
fn markdown_article_search_related_results_include_discover_for_psoralen_vaccine_phrase() {
    let related = markdown_related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("psoralen vaccines"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(related.contains(&"biomcp discover \"psoralen vaccines\"".to_string()));
}

#[test]
fn markdown_article_search_related_results_include_discover_but_not_heuristic_disease_get(
) {
    let related = markdown_related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("Emanuel syndrome"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(related.contains(&"biomcp discover \"Emanuel syndrome\"".to_string()));
    assert!(!related.contains(&"biomcp get disease \"Emanuel syndrome\"".to_string()));
}

#[test]
fn markdown_article_search_related_results_do_not_emit_lung_cancer_immunotherapy_disease_get() {
    let related = markdown_related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("lung cancer immunotherapy"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(
        !related
            .iter()
            .any(|command| command.starts_with("biomcp get disease "))
    );
}

#[test]
fn markdown_article_search_related_results_include_exact_disease_command_when_precomputed() {
    let exact_commands = vec!["biomcp get disease melanoma".to_string()];
    let related = markdown_related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("melanoma"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &exact_commands,
    );

    assert!(related.contains(&"biomcp get disease melanoma".to_string()));
}

#[test]
fn markdown_article_search_related_results_skip_cross_entity_hints_for_long_phrase() {
    let related = markdown_related_article_search_results(
        &[article_search_result("22663011")],
        &article_filters(Some("live attenuated vaccines still in use review"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(!related.iter().any(|command| command.starts_with("biomcp discover ")));
    assert!(!related.iter().any(|command| command.starts_with("biomcp get disease ")));
}

#[test]
fn article_search_related_results_include_year_refinement_hint_when_unbounded() {
    let related = related_article_search_results(
        &[
            article_search_result("22663011"),
            ArticleSearchResult {
                pmid: "24200969".to_string(),
                date: Some("2013-05-12".to_string()),
                ..article_search_result("24200969")
            },
        ],
        &article_filters(Some("BRAF melanoma"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(related.contains(
        &"biomcp search article -k \"BRAF melanoma\" --year-min 2013 --year-max 2025 --limit 5"
            .to_string()
    ));
    assert_eq!(
        related_command_description(
            "biomcp search article -k \"BRAF melanoma\" --year-min 2013 --year-max 2025 --limit 5"
        ),
        Some("refine this search to the visible publication-year range")
    );
}

#[test]
fn article_search_related_results_skip_year_refinement_when_already_bounded() {
    let mut filters = article_filters(Some("BRAF melanoma"), None, None);
    filters.date_from = Some("2000-01-01".to_string());
    let related = related_article_search_results(
        &[article_search_result("22663011")],
        &filters,
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(!related.iter().any(|command| command.contains("--year-min")));
}

#[test]
fn article_search_related_results_skip_year_refinement_without_visible_years() {
    let related = related_article_search_results(
        &[ArticleSearchResult {
            date: None,
            ..article_search_result("22663011")
        }],
        &article_filters(Some("BRAF melanoma"), None, None),
        crate::entities::article::ArticleSourceFilter::All,
        &[],
    );

    assert!(!related.iter().any(|command| command.contains("--year-min")));
}

#[test]
fn article_search_related_results_preserve_source_filter_in_year_refinement() {
    let related = related_article_search_results(
        &[ArticleSearchResult {
            date: Some("2013-05-12".to_string()),
            ..article_search_result("22663011")
        }],
        &article_filters(Some("BRAF melanoma"), None, None),
        crate::entities::article::ArticleSourceFilter::PubMed,
        &[],
    );

    assert!(related.contains(
        &"biomcp search article -k \"BRAF melanoma\" --source pubmed --year-min 2013 --year-max 2013 --limit 5".to_string()
    ));
}
