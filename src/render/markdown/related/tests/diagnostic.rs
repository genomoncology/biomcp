#[test]
fn search_next_commands_diagnostic_prefers_top_accession_then_list() {
    let results = vec![
        DiagnosticSearchResult {
            source: "gtr".to_string(),
            accession: "GTR000000001.1".to_string(),
            name: "BRCA1 Hereditary Cancer Panel".to_string(),
            test_type: Some("molecular".to_string()),
            manufacturer_or_lab: Some("OncoPanel BRCA1".to_string()),
            genes: vec!["BRCA1".to_string()],
            conditions: vec!["Breast cancer".to_string()],
        },
        DiagnosticSearchResult {
            source: "gtr".to_string(),
            accession: "GTR000000002.1".to_string(),
            name: "EGFR Melanoma Molecular Assay".to_string(),
            test_type: Some("molecular".to_string()),
            manufacturer_or_lab: Some("Precision Diagnostics".to_string()),
            genes: vec!["EGFR".to_string()],
            conditions: vec!["Cutaneous melanoma".to_string()],
        },
    ];

    let commands = search_next_commands_diagnostic(&results);
    assert_eq!(
        commands,
        vec![
            "biomcp get diagnostic GTR000000001.1".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
}

#[test]
fn related_diagnostic_only_points_back_to_list_help() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "gtr".to_string(),
        source_id: "GTR000000001.1".to_string(),
        accession: "GTR000000001.1".to_string(),
        name: "BRCA1 Hereditary Cancer Panel".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer: Some("OncoPanel BRCA1".to_string()),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: Some("GenomOncology Lab".to_string()),
        institution: Some("GenomOncology Institute".to_string()),
        country: Some("USA".to_string()),
        clia_number: Some("12D3456789".to_string()),
        state_licenses: Some("NY|CA".to_string()),
        current_status: Some("Current".to_string()),
        public_status: Some("Public".to_string()),
        method_categories: vec!["Molecular genetics".to_string()],
        genes: None,
        conditions: None,
        methods: None,
        regulatory: None,
    };

    assert_eq!(
        related_diagnostic(&diagnostic),
        vec!["biomcp list diagnostic".to_string()]
    );
}

#[test]
fn search_next_commands_diagnostic_quotes_who_product_code() {
    let results = vec![DiagnosticSearchResult {
        source: "who-ivd".to_string(),
        accession: "ITPW02232- TC40".to_string(),
        name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
        test_type: Some("Immunochromatographic (lateral flow)".to_string()),
        manufacturer_or_lab: Some("InTec Products, Inc.".to_string()),
        genes: vec![],
        conditions: vec!["HIV".to_string()],
    }];

    assert_eq!(
        search_next_commands_diagnostic(&results),
        vec![
            "biomcp get diagnostic \"ITPW02232- TC40\"".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
}

#[test]
fn diagnostic_zero_result_recovery_commands_point_to_list_help() {
    let commands = diagnostic_zero_result_recovery_commands();
    assert_eq!(commands, vec!["biomcp list diagnostic".to_string()]);

    let rendered = format_related_block(commands);
    assert!(rendered.contains("biomcp list diagnostic"));
    assert!(rendered.contains("source-aware diagnostic filters and local GTR/WHO IVD usage"));
}
