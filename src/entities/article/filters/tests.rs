#[allow(unused_imports)]
use super::super::test_support::*;
use super::*;

fn keyword_filters(keyword: &str) -> ArticleSearchFilters {
    ArticleSearchFilters {
        keyword: Some(keyword.into()),
        ..empty_filters()
    }
}

#[test]
fn native_keyword_fields_are_rejected_with_typed_guidance() {
    for (keyword, guidance) in [
        ("Williams LS[Author]", "--author"),
        ("Williams LS[au] AND melanoma", "--author"),
        ("Williams LS[author]\u{a0}AND melanoma", "--author"),
        ("Smith[Ad]", "ordinary unfielded -k/--keyword text"),
        ("Nature[journal]", "--journal"),
        ("Nature[jour])", "--journal"),
        ("AUTH:Williams LS", "--author"),
        ("melanoma (aUtH:Williams LS)", "--author"),
        (
            "AFFILIATION:Harvard",
            "ordinary unfielded -k/--keyword text",
        ),
        ("melanoma JOURNAL:Nature", "--journal"),
    ] {
        let err = validate_search_filter_values(&keyword_filters(keyword))
            .expect_err("native field syntax should be rejected");
        let message = err.to_string();
        assert!(message.contains("provider-neutral"), "{keyword}: {message}");
        assert!(message.contains(guidance), "{keyword}: {message}");
    }
}

#[test]
fn ordinary_bracket_and_colon_keywords_remain_valid() {
    for keyword in [
        "BRAF p.V600E",
        "NM_004333.6:c.1799T>A",
        "BRAF[variant] melanoma",
        "MYAUTH:Williams",
        "AUTH receptor signaling",
        "[author] Williams",
        "Williams[author]ized",
        "protein:protein interaction",
        "TP53 (p.Arg175His)",
    ] {
        validate_search_filter_values(&keyword_filters(keyword))
            .unwrap_or_else(|err| panic!("literal keyword {keyword:?} should remain valid: {err}"));
    }
}

#[test]
fn reserved_article_keyword_fields_have_precise_boundaries_and_guidance() {
    const GENE: &str = "Invalid argument: keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example \"gene\":\"RB1\".";
    const DISEASE: &str = "Invalid argument: keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example \"disease\":\"melanoma\".";
    const DRUG: &str = "Invalid argument: keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example \"drug\":\"vemurafenib\".";
    for (keyword, expected) in [
        ("gene:RB1", GENE),
        ("GENE:RB1", GENE),
        ("melanoma gene:RB1", GENE),
        ("melanoma (gene:RB1)", GENE),
        ("melanoma\u{a0}gene:RB1", GENE),
        ("gene:\"RB1\"", GENE),
        ("disease:melanoma", DISEASE),
        ("DRUG:vemurafenib", DRUG),
    ] {
        let err = validate_search_filter_values(&keyword_filters(keyword))
            .expect_err("reserved article field syntax should be rejected");
        let message = err.to_string();
        assert_eq!(message, expected, "{keyword:?}");
        assert!(
            !message.contains(keyword),
            "must not reflect input: {message}"
        );
    }
}

#[test]
fn literal_colons_false_prefixes_and_quote_bytes_remain_keywords() {
    for keyword in [
        "NM_004333.6:c.1799T>A",
        "protein:protein interaction",
        "oncogene:RB1",
        "MYGENE:RB1",
        "ratio 1:2",
        "BRAF[variant]",
        "\"gene:gene interaction\"",
        "melanoma \"gene:RB1",
    ] {
        validate_search_filter_values(&keyword_filters(keyword))
            .unwrap_or_else(|err| panic!("literal keyword {keyword:?} should remain valid: {err}"));
    }
}

#[test]
fn article_gene_accepts_one_trimmed_whitespace_free_token() {
    for gene in ["BRAF", "braf", "PD-L1", "H3-3A", "  BRAF  "] {
        let filters = ArticleSearchFilters {
            gene: Some(gene.into()),
            ..empty_filters()
        };
        validate_search_filter_values(&filters)
            .unwrap_or_else(|err| panic!("gene {gene:?} should remain valid: {err}"));
    }
}

#[test]
fn article_gene_rejects_empty_or_unicode_whitespace_with_fixed_guidance() {
    const EXPECTED: &str = "Invalid argument: gene accepts one symbol, for example TPMT. Put additional concepts in keyword: use --gene TPMT --keyword mercaptopurine for CLI or raw MCP, or typed MCP fields \"gene\":\"TPMT\" and \"keyword\":[\"mercaptopurine\"].";
    for gene in [
        "",
        " \t\n",
        "TPMT mercaptopurine",
        "TPMT\tmercaptopurine",
        "TPMT\nmercaptopurine",
        "TPMT\u{a0}mercaptopurine",
    ] {
        let filters = ArticleSearchFilters {
            gene: Some(gene.into()),
            ..empty_filters()
        };
        let err = validate_search_filter_values(&filters)
            .expect_err("empty or multi-concept article gene should be rejected");
        let message = err.to_string();
        assert_eq!(message, EXPECTED, "{gene:?}");
        if !gene.is_empty() {
            assert!(!message.contains(gene), "must not reflect input: {message}");
        }
    }
}

#[test]
fn normalized_date_bounds_normalizes_partial_dates() {
    let mut filters = empty_filters();
    filters.date_from = Some("2020".into());
    filters.date_to = Some("2024-12".into());

    let (date_from, date_to) =
        normalized_date_bounds(&filters).expect("partial dates should normalize");

    assert_eq!(date_from.as_deref(), Some("2020-01-01"));
    assert_eq!(date_to.as_deref(), Some("2024-12-01"));
}

#[test]
fn normalized_date_bounds_rejects_bad_month() {
    let mut filters = empty_filters();
    filters.date_from = Some("2024-13-01".into());

    let err = normalized_date_bounds(&filters).expect_err("invalid month should fail");

    assert_eq!(
        err.to_string(),
        "Invalid argument: Invalid month 13 in --date-from (must be 01-12)"
    );
}

#[test]
fn normalized_date_bounds_rejects_bad_date_to_with_flag_name() {
    let mut filters = empty_filters();
    filters.date_to = Some("2024-99".into());

    let err = normalized_date_bounds(&filters).expect_err("invalid date-to should fail");

    assert_eq!(
        err.to_string(),
        "Invalid argument: Invalid month 99 in --date-to (must be 01-12)"
    );
}

#[test]
fn normalized_date_bounds_rejects_inverted_range() {
    let mut filters = empty_filters();
    filters.date_from = Some("2024-06-01".into());
    filters.date_to = Some("2020-01-01".into());

    let err = normalized_date_bounds(&filters).expect_err("inverted range should fail");

    assert_eq!(
        err.to_string(),
        "Invalid argument: --date-from must be <= --date-to"
    );
}

#[test]
fn normalize_article_type_accepts_aliases() {
    assert_eq!(
        normalize_article_type("review").expect("review should normalize"),
        "review"
    );
    assert_eq!(
        normalize_article_type("research").expect("research alias should normalize"),
        "research-article"
    );
    assert_eq!(
        normalize_article_type("research-article").expect("research-article should normalize"),
        "research-article"
    );
    assert_eq!(
        normalize_article_type("case-reports").expect("case-reports should normalize"),
        "case-reports"
    );
    assert_eq!(
        normalize_article_type("metaanalysis").expect("metaanalysis alias should normalize"),
        "meta-analysis"
    );
}

#[test]
fn partial_date_normalization_and_filtering_are_consistent() {
    assert_eq!(parse_row_date(Some("2024")), Some("2024-01-01".into()));
    assert_eq!(parse_row_date(Some("2024-06")), Some("2024-06-01".into()));
    assert_eq!(
        parse_row_date(Some("2024-06-15")),
        Some("2024-06-15".into())
    );

    assert!(matches_optional_date_filter(
        Some("2024"),
        Some("2024-01-01"),
        None,
    ));
    assert!(!matches_optional_date_filter(
        Some("2023"),
        Some("2024-01-01"),
        None,
    ));
    assert!(matches_optional_date_filter(
        Some("2024-06"),
        None,
        Some("2024-12-31"),
    ));
}

#[test]
fn exclude_retracted_only_filters_confirmed_retractions() {
    let confirmed_retracted = row_with(
        "100",
        ArticleSource::PubTator,
        Some("2025-01-01"),
        Some(1),
        Some(true),
    );
    let confirmed_not_retracted = row_with(
        "101",
        ArticleSource::PubTator,
        Some("2025-01-01"),
        Some(1),
        Some(false),
    );
    let exclude_filters = ArticleSearchFilters {
        exclude_retracted: true,
        ..empty_filters()
    };
    let include_filters = ArticleSearchFilters {
        exclude_retracted: false,
        ..empty_filters()
    };

    assert!(!matches_result_filters(
        &confirmed_retracted,
        &exclude_filters,
        None,
        None
    ));
    assert!(matches_result_filters(
        &confirmed_retracted,
        &include_filters,
        None,
        None
    ));
    assert!(matches_result_filters(
        &confirmed_not_retracted,
        &exclude_filters,
        None,
        None
    ));
}

#[test]
fn exclude_retracted_keeps_unknown_retraction_status() {
    let row = row_with(
        "100",
        ArticleSource::PubTator,
        Some("2025-01-01"),
        Some(1),
        None,
    );
    let exclude_filters = ArticleSearchFilters {
        exclude_retracted: true,
        ..empty_filters()
    };
    let include_filters = ArticleSearchFilters {
        exclude_retracted: false,
        ..empty_filters()
    };

    assert!(matches_result_filters(&row, &exclude_filters, None, None));
    assert!(matches_result_filters(&row, &include_filters, None, None));
}
