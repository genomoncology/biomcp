use super::*;

#[test]
fn disease_search_request_records_normalized_filters_and_fetch_plan() {
    let filters = DiseaseSearchFilters {
        query: Some(" chronic myeloid leukemia ".into()),
        source: Some(" DOID ".into()),
        inheritance: Some(" autosomal dominant ".into()),
        phenotype: Some(" HP:0001250 ".into()),
        onset: Some(" childhood ".into()),
    };

    let request = DiseaseSearchRequest::new(&filters, 3, 2).expect("request");

    assert_eq!(request.query, "chronic myeloid leukemia");
    assert_eq!(request.source.as_deref(), Some("DOID"));
    assert_eq!(request.inheritance.as_deref(), Some("autosomal dominant"));
    assert_eq!(request.phenotype.as_deref(), Some("HP:0001250"));
    assert_eq!(request.onset.as_deref(), Some("childhood"));
    assert_eq!(request.limit, 3);
    assert_eq!(request.offset, 2);
    assert_eq!(request.fetch_size, 25);
    assert!(
        request
            .resolver_queries
            .iter()
            .any(|value| value == "chronic myeloid leukemia")
    );
    assert!(request.prefer_doid);
}

#[test]
fn ticket_400_request_command_disease_search_fields_drive_source_query_and_pagination() {
    let filters = DiseaseSearchFilters {
        query: Some(" chronic myeloid leukemia ".into()),
        source: Some(" doid ".into()),
        inheritance: Some(" autosomal dominant ".into()),
        phenotype: Some(" HP:0001250 ".into()),
        onset: Some(" childhood ".into()),
    };
    let request = DiseaseSearchRequest::new(&filters, 3, 2).expect("request");
    let client =
        crate::sources::mydisease::MyDiseaseClient::new_for_test("http://127.0.0.1/v1".into())
            .expect("mydisease client");
    let plan = client
        .query_request_plan(
            &request.resolver_queries[0],
            request.fetch_size,
            0,
            request.source.as_deref(),
            request.inheritance.as_deref(),
            request.phenotype.as_deref(),
            request.onset.as_deref(),
        )
        .expect("source query plan");

    assert_eq!(request.limit, 3);
    assert_eq!(request.offset, 2);
    assert_eq!(plan.path, "/query");
    assert!(plan.query_params.contains(&("size", "25".to_string())));
    assert!(plan.query_params.contains(&("from", "0".to_string())));
    assert!(plan.query_params.iter().any(|(key, value)| {
        *key == "q"
            && value.contains("chronic myeloid leukemia")
            && value.contains("disease_ontology.doid:*")
            && value.contains("hpo.inheritance.hpo_name:*autosomal dominant*")
            && value.contains("hpo.phenotype_related_to_disease.hpo_id:*HP\\:0001250*")
            && value.contains("hpo.clinical_course.hpo_name:*childhood*")
    }));
}

#[test]
fn disease_filter_normalizers_accept_supported_values() {
    let inheritance_names = [
        "autosomal dominant",
        "autosomal recessive",
        "x-linked",
        "x-linked dominant",
        "x-linked recessive",
        "y-linked",
        "mitochondrial",
        "multifactorial",
        "oligogenic",
        "polygenic",
        "sporadic",
        "somatic mosaicism",
        "dominant",
        "recessive",
    ];
    for value in inheritance_names {
        assert_eq!(normalize_inheritance(value).unwrap(), value);
    }
    for value in [
        "HP:0000006",
        "HP:0000007",
        "HP:0001417",
        "HP:0001423",
        "HP:0001419",
        "HP:0001450",
        "HP:0001427",
        "HP:0001426",
        "HP:0010983",
        "HP:0010982",
        "HP:0003745",
        "HP:0001442",
    ] {
        assert_eq!(normalize_inheritance(value).unwrap(), value);
    }

    for value in [
        "antenatal",
        "embryonal",
        "fetal",
        "congenital",
        "neonatal",
        "infantile",
        "childhood",
        "juvenile",
        "adolescent",
        "young adult",
        "adult",
        "middle age",
        "late onset",
    ] {
        assert_eq!(normalize_onset(value).unwrap(), value);
    }
    assert_eq!(normalize_onset("infancy").unwrap(), "infantile");
}

#[test]
fn disease_filter_normalizers_handle_case_whitespace_and_unknown_values() {
    assert_eq!(
        normalize_inheritance(" Autosomal Dominant ").unwrap(),
        "autosomal dominant"
    );
    assert_eq!(normalize_inheritance(" hp:0000006 ").unwrap(), "HP:0000006");
    assert_eq!(normalize_onset(" Young Adult ").unwrap(), "young adult");
    for value in ["", "unknown inheritance"] {
        assert!(normalize_inheritance(value).is_err());
    }
    for value in ["", "unknown onset"] {
        assert!(normalize_onset(value).is_err());
    }
}

#[test]
fn disease_search_request_preserves_limit_and_query_validation() {
    let filters = DiseaseSearchFilters::default();
    let err = DiseaseSearchRequest::new(&filters, 0, 0).expect_err("limit should fail");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));

    let err = DiseaseSearchRequest::new(&filters, 1, 0).expect_err("query should fail");
    assert!(matches!(err, BioMcpError::InvalidArgument(_)));
}

#[test]
fn parse_hpo_query_terms_requires_valid_ids() {
    let parsed = parse_hpo_query_terms("HP:0001250 HP:0001263").expect("valid terms");
    assert_eq!(parsed, vec!["HP:0001250", "HP:0001263"]);
    let comma_separated = parse_hpo_query_terms("hp:0001250, HP:0001263").expect("comma terms");
    assert_eq!(comma_separated, vec!["HP:0001250", "HP:0001263"]);
    assert!(parse_hpo_query_terms("NOT_AN_HPO").is_err());
}

#[test]
fn phenotype_terms_and_result_window_are_bounded() {
    let eleven = (1..=11)
        .map(|index| format!("HP:{index:07}"))
        .collect::<Vec<_>>()
        .join(" ");
    let err = parse_hpo_query_terms(&eleven).expect_err("eleven HPO terms should fail");
    assert!(err.to_string().contains("at most 10 unique HPO terms"));

    assert_eq!(validate_phenotype_search_window(10, 40).unwrap(), 50);
    for (limit, offset) in [(1, 50), (11, 40), (2, usize::MAX)] {
        let err = validate_phenotype_search_window(limit, offset)
            .expect_err("window beyond Monarch's first 50 rows should fail");
        assert!(err.to_string().contains("--offset + --limit must be <= 50"));
    }
}

#[test]
fn phenotype_continuation_shrinks_to_the_remaining_provider_window() {
    let pagination = PhenotypePagination {
        offset: 40,
        limit: 9,
        returned: 9,
        total: None,
        has_more: true,
        next_page_token: None,
        provider_window_limit: 50,
        provider_raw_row_count: 50,
        provider_window_exhausted: true,
    };

    assert_eq!(pagination.next_window(), Some((1, 49)));
}

#[test]
fn phenotype_pages_slice_one_provider_ordered_deduplicated_window() {
    let provider = crate::sources::monarch::MonarchPhenotypeSearchResponse {
        matches: vec![
            ("MONDO:0000003", "third", 10.0),
            ("MONDO:0000001", "first tie", 9.0),
            ("MONDO:0000002", "second tie", 9.0),
            ("MONDO:0000004", "fourth", 8.0),
            ("MONDO:0000005", "fifth", 7.0),
        ]
        .into_iter()
        .map(|(disease_id, disease_name, score)| MonarchPhenotypeMatch {
            disease_id: disease_id.into(),
            disease_name: disease_name.into(),
            score,
        })
        .collect(),
        raw_row_count: 50,
        provider_window_exhausted: true,
    };

    let first = paginate_phenotype_matches(provider.clone(), 2, 0).unwrap();
    let second = paginate_phenotype_matches(provider.clone(), 3, 2).unwrap();
    let combined = paginate_phenotype_matches(provider, 5, 0).unwrap();
    let paged = first
        .results
        .iter()
        .chain(&second.results)
        .map(|row| row.disease_id.as_str())
        .collect::<Vec<_>>();
    let all = combined
        .results
        .iter()
        .map(|row| row.disease_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(paged, all);
    assert_eq!(
        all,
        [
            "MONDO:0000003",
            "MONDO:0000001",
            "MONDO:0000002",
            "MONDO:0000004",
            "MONDO:0000005"
        ]
    );
    assert!(first.pagination.has_more);
    assert!(first.pagination.provider_window_exhausted);
    assert_eq!(first.pagination.provider_window_limit, 50);
    assert_eq!(first.pagination.provider_raw_row_count, 50);
    assert!(!combined.pagination.has_more);
    assert!(combined.pagination.provider_window_exhausted);
}

#[test]
fn split_phenotype_queries_preserves_single_phrase_and_splits_commas() {
    assert_eq!(
        split_phenotype_queries("developmental delay"),
        vec!["developmental delay"]
    );
    assert_eq!(
        split_phenotype_queries("seizure, developmental delay,  hypotonia "),
        vec!["seizure", "developmental delay", "hypotonia"]
    );
}

#[tokio::test]
async fn resolve_phenotype_query_terms_empty_input_mentions_hpo_ids_and_symptom_phrases() {
    let err = resolve_phenotype_query_terms("   ")
        .await
        .expect_err("empty phenotype query should fail");

    match err {
        BioMcpError::InvalidArgument(message) => {
            assert!(message.contains("Use HPO IDs or symptom phrases"));
            assert!(message.contains("HP:0001250 HP:0001263"));
            assert!(message.contains("seizure, developmental delay"));
        }
        other => panic!("expected InvalidArgument, got: {other}"),
    }
}
