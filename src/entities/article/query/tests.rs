#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;
use crate::entities::article::ArticleVariantIntent;

#[test]
fn pubtator_sort_omits_param_for_relevance() {
    assert_eq!(pubtator_sort(ArticleSort::Relevance), None);
}

#[test]
fn pubtator_sort_sends_param_for_date() {
    assert_eq!(pubtator_sort(ArticleSort::Date), Some("date desc"));
}

#[test]
fn strict_variant_templates_quote_and_escape_provider_inputs() {
    assert_eq!(
        build_pubmed_variant_strict_query("TP53", "c.356C>A"),
        "(\"TP53\"[Title/Abstract] AND \"c.356C>A\"[Title/Abstract])"
    );
    assert_eq!(
        build_europepmc_variant_strict_query("TP53", "c.356C>A"),
        "TITLE_ABS:\"TP53 c.356C>A\""
    );
    assert_eq!(
        build_pubmed_variant_strict_query("TP53\" OR X", "c.356C>A\\\\x"),
        "(\"TP53\\\" OR X\"[Title/Abstract] AND \"c.356C>A\\\\\\\\x\"[Title/Abstract])"
    );
}

#[test]
fn europepmc_keyword_does_not_quote_whitespace() {
    let term = europepmc_keyword("large language model clinical trials");
    assert_eq!(term, "large language model clinical trials");
}

#[test]
fn build_search_query_keeps_phrase_quoting_for_entity_filters() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF V600E".into());
    filters.author = Some("Jane Doe".into());

    let query = build_search_query(&filters).expect("query should build");
    assert!(query.contains("\"BRAF V600E\""));
    assert!(query.contains("AUTH:\"Jane Doe\""));
}

#[test]
fn build_search_query_uses_gene_anchor_field_when_requested() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.gene_anchored = true;
    let query = build_search_query(&filters).expect("query should build");
    assert!(query.contains("GENE_PROTEIN:BRAF"));
}

#[test]
fn build_search_query_combines_keyword_and_since() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.keyword = Some("large language model".into());
    filters.date_from = Some("2024-01-01".into());
    filters.no_preprints = true;

    let query = build_search_query(&filters).expect("query should build");
    assert!(query.contains("BRAF"));
    assert!(query.contains("large language model"));
    assert!(query.contains("FIRST_PDATE:[2024-01-01 TO *]"));
    assert!(query.contains("NOT SRC:PPR"));
}

#[test]
fn build_search_query_excludes_retracted_when_requested() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.exclude_retracted = true;
    let query = build_search_query(&filters).expect("query should build");
    assert!(query.contains("NOT PUB_TYPE:\"retracted publication\""));
}

#[test]
fn native_query_builders_share_keyword_field_validation() {
    let mut filters = empty_filters();
    filters.keyword = Some("Williams LS[Author]".into());

    assert!(build_search_query(&filters).is_err());
    assert!(build_pubmed_search_term(&filters).is_err());
}

#[test]
fn native_query_builders_keep_unrecognized_keyword_punctuation_literal() {
    let mut filters = empty_filters();
    filters.keyword = Some("BRAF[variant] protein:protein".into());

    assert_eq!(
        build_search_query(&filters).expect("Europe PMC query should build"),
        "BRAF\\[variant\\] protein\\:protein"
    );
    assert_eq!(
        build_pubmed_search_term(&filters).expect("PubMed query should build"),
        "BRAF[variant] protein:protein"
    );
}

#[test]
fn build_search_query_rejects_unknown_article_type() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.article_type = Some("invalid".into());

    let err = build_search_query(&filters).expect_err("invalid article type should fail");
    let msg = err.to_string();
    assert!(msg.contains("Invalid argument"));
    assert!(msg.contains("case-reports"));
}

#[test]
fn build_free_text_article_query_preserves_mixed_semantic_anchors() {
    let mut filters = empty_filters();
    filters.gene = Some(" RET ".into());
    filters.disease = Some(" Hirschsprung disease ".into());
    filters.drug = Some(" selpercatinib ".into());
    filters.keyword = Some(" ganglion cells ".into());
    filters.author = Some(" Alice Smith ".into());

    let query = build_free_text_article_query(&filters);

    assert_eq!(
        query,
        "RET Hirschsprung disease selpercatinib ganglion cells Alice Smith"
    );
}

#[test]
fn strip_pubmed_stopwords_cleans_question_patterns() {
    let cases = [
        (
            "What drug treatment can cause a spinal epidural hematoma?",
            "drug treatment spinal epidural hematoma",
        ),
        (
            "What is the incidence of cystic fibrosis in the caucasian population?",
            "incidence cystic fibrosis caucasian population",
        ),
        (
            "Which are the genes thought to be regulated by EWS/FLI?",
            "genes regulated EWS FLI",
        ),
        ("How does BRAF regulate melanoma.", "BRAF regulate melanoma"),
        (
            "List the drugs for cystic fibrosis.",
            "drugs cystic fibrosis",
        ),
        ("Can TP53 and MDM2 be regulated?", "TP53 MDM2 regulated"),
        ("What DOES iPSC model CFTR?", "iPSC model CFTR"),
        ("BRAF melanoma", "BRAF melanoma"),
        ("AND or the?", "AND or the?"),
    ];

    for (raw, expected) in cases {
        assert_eq!(strip_pubmed_stopwords(raw), expected, "raw={raw:?}");
    }
}

#[test]
fn ticket_406_myd88_exact_protein_alias_article_precision() {
    let mut filters = empty_filters();
    filters.gene = Some("MYD88".into());
    filters.keyword = Some("p.N291S".into());

    let term = build_pubmed_search_term(&filters).expect("pubmed term should build");

    assert!(
        term.contains("MYD88") && term.contains("\"p.N291S\""),
        "exact gene/protein-alias article searches must preserve both the gene anchor and quoted exact protein alias, got {term:?}"
    );
}

#[test]
fn build_pubmed_search_term_cleans_unfielded_clauses_only() {
    let mut filters = empty_filters();
    filters.gene = Some("Which are the genes thought to be regulated by EWS/FLI?".into());
    filters.disease =
        Some("What is the incidence of cystic fibrosis in the caucasian population?".into());
    filters.drug = Some("What drug treatment can cause a spinal epidural hematoma?".into());
    filters.keyword = Some("How does BRAF regulate melanoma.".into());
    filters.author = Some("Alice Smith".into());
    filters.journal = Some("Nature Reviews".into());
    filters.article_type = Some("review".into());
    filters.exclude_retracted = true;

    let term = build_pubmed_search_term(&filters).expect("pubmed term should build");

    assert_eq!(
        term,
        "genes regulated EWS FLI AND incidence cystic fibrosis caucasian population AND drug treatment spinal epidural hematoma AND BRAF regulate melanoma AND \"Alice Smith\"[author] AND \"Nature Reviews\"[journal] AND review[pt] NOT retracted publication[pt]"
    );
}

#[test]
fn build_pubmed_search_term_keeps_quoted_author_input_inside_author_field() {
    let mut filters = empty_filters();
    filters.author = Some("Smith\" OR cancer[title] OR \"Doe".into());

    let term = build_pubmed_search_term(&filters).expect("PubMed term should build");

    assert_eq!(term, "\"Smith  OR cancer[title] OR  Doe\"[author]");
}

#[test]
fn build_pubmed_search_term_falls_back_for_all_stopword_unfielded_clause() {
    let mut filters = empty_filters();
    filters.keyword = Some("AND or the?".into());

    let term = build_pubmed_search_term(&filters).expect("pubmed term should build");

    assert_eq!(term, "AND or the?");
}

#[test]
fn build_pubmed_esearch_params_reuses_article_type_aliases() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.keyword = Some("melanoma".into());
    filters.author = Some("Alice Smith".into());
    filters.journal = Some("Nature".into());
    filters.article_type = Some("research".into());
    filters.date_from = Some("2020".into());
    filters.date_to = Some("2024-12".into());
    filters.exclude_retracted = true;

    let params = build_pubmed_esearch_params(&filters, 5, 10).expect("pubmed params should build");

    assert_eq!(
        params.term,
        "BRAF AND melanoma AND \"Alice Smith\"[author] AND \"Nature\"[journal] AND journal article[pt] NOT retracted publication[pt]"
    );
    assert_eq!(params.retstart, 10);
    assert_eq!(params.retmax, 5);
    assert_eq!(params.date_from.as_deref(), Some("2020-01-01"));
    assert_eq!(params.date_to.as_deref(), Some("2024-12-01"));
}

#[test]
fn build_pubmed_search_term_uses_standalone_not_for_retraction_filter() {
    let mut filters = empty_filters();
    filters.gene = Some("WDR5".into());
    filters.exclude_retracted = true;

    let term = build_pubmed_search_term(&filters).expect("pubmed term should build");

    assert_eq!(term, "WDR5 NOT retracted publication[pt]");
    assert!(
        !term.contains("AND NOT"),
        "term must not contain 'AND NOT': {term:?}"
    );
}

#[test]
fn build_pubmed_esearch_params_allows_federated_windows_above_user_limit() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    let params = build_pubmed_esearch_params(&filters, 75, 0).expect("pubmed params should build");

    assert_eq!(params.retmax, 75);
    assert_eq!(params.retstart, 0);
}

#[test]
fn pubtator_variant_candidate_requires_source_proof_for_gene_and_protein() {
    let intent = ArticleVariantIntent {
        original: "BRAF p.Val600Glu".into(),
        gene: Some("BRAF".into()),
        change: Some("V600E".into()),
        entity_id: None,
    };
    let row = |name: &str| PubTatorAutocompleteResult {
        id: Some("variant-fixture".into()),
        biotype: Some("variant".into()),
        db_id: None,
        db: None,
        name: Some(name.into()),
    };

    assert!(variant_candidate_matches(&row("BRAF p.V600E"), &intent));
    assert!(!variant_candidate_matches(&row("BRAF p.V601E"), &intent));
    assert!(!variant_candidate_matches(&row("ARAF p.V600E"), &intent));
    assert!(!variant_candidate_matches(&row("BRAF mutation"), &intent));
}

#[test]
fn build_pubmed_esearch_params_rejects_open_access() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.open_access = true;

    let err = build_pubmed_esearch_params(&filters, 5, 0)
        .expect_err("open-access should be rejected for PubMed builder");

    assert!(err.to_string().contains("--open-access"));
    assert!(err.to_string().contains("PubMed"));
}

#[test]
fn build_pubmed_esearch_params_rejects_no_preprints() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());
    filters.no_preprints = true;

    let err = build_pubmed_esearch_params(&filters, 5, 0)
        .expect_err("no-preprints should be rejected for PubMed builder");

    assert!(err.to_string().contains("--no-preprints"));
    assert!(err.to_string().contains("PubMed"));
}

#[test]
fn build_pubmed_esearch_params_rejects_federated_window_overflow() {
    let mut filters = empty_filters();
    filters.gene = Some("BRAF".into());

    let err = build_pubmed_esearch_params(&filters, 1, MAX_FEDERATED_FETCH_RESULTS)
        .expect_err("offset + limit overflow should be rejected");

    assert!(
        err.to_string()
            .contains("--offset + --limit must be <= 1250 for federated article search")
    );
}
