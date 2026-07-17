use super::*;
use crate::entities::diagnostic::DiagnosticRegulatoryRecord;

#[test]
fn diagnostic_markdown_renders_requested_sections_and_truthful_empty_states() {
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
        genes: Some(vec![]),
        conditions: Some(vec!["Breast cancer".to_string()]),
        methods: Some(vec![]),
        regulatory: None,
    };

    let markdown = diagnostic_markdown(
        &diagnostic,
        &[
            "genes".to_string(),
            "conditions".to_string(),
            "methods".to_string(),
        ],
    )
    .expect("rendered markdown");

    assert!(markdown.contains("# Diagnostic: GTR000000001.1"));
    assert!(markdown.contains("Source: NCBI Genetic Testing Registry"));
    assert!(markdown.contains("Method Categories: Molecular genetics"));
    assert!(markdown.contains("## Genes"));
    assert!(markdown.contains("No genes listed in NCBI Genetic Testing Registry."));
    assert!(markdown.contains("## Conditions"));
    assert!(markdown.contains("Breast cancer"));
    assert!(markdown.contains("## Methods"));
    assert!(markdown.contains("No methods listed in NCBI Genetic Testing Registry."));
    assert!(markdown.contains("biomcp list diagnostic"));
}

#[test]
fn diagnostic_search_markdown_shows_source_column_and_detail_hint() {
    let results = vec![
        DiagnosticSearchResult {
            source: "gtr".to_string(),
            accession: "GTR000000001.1".to_string(),
            name: "BRCA1 Hereditary Cancer Panel".to_string(),
            test_type: Some("molecular".to_string()),
            manufacturer_or_lab: Some("OncoPanel BRCA1".to_string()),
            genes: vec!["BRCA1".to_string(), "BARD1".to_string()],
            conditions: vec!["Breast cancer".to_string()],
        },
        DiagnosticSearchResult {
            source: "who-ivd".to_string(),
            accession: "ITPW02232- TC40".to_string(),
            name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
            test_type: Some("Immunochromatographic (lateral flow)".to_string()),
            manufacturer_or_lab: Some("InTec Products, Inc.".to_string()),
            genes: vec![],
            conditions: vec!["HIV".to_string()],
        },
    ];

    let markdown = diagnostic_search_markdown("gene=BRCA1", &results, Some(results.len()))
        .expect("rendered markdown");

    assert!(markdown.contains("# Diagnostic tests: gene=BRCA1"));
    assert!(markdown.contains("|Accession|Name|Type|Manufacturer / Lab|Source|Genes|Conditions|"));
    assert!(markdown.contains("|GTR000000001.1|BRCA1 Hereditary Cancer Panel|molecular|OncoPanel BRCA1|NCBI Genetic Testing Registry|BRCA1, BARD1|Breast cancer|"));
    assert!(markdown.contains("|ITPW02232- TC40|ONE STEP Anti-HIV (1&2) Test|Immunochromatographic (lateral flow)|InTec Products, Inc.|WHO Prequalified IVD|-|HIV|"));
    assert!(markdown.contains("Use `biomcp get diagnostic GTR000000001.1` for details."));
}

#[test]
fn diagnostic_search_markdown_renders_true_zero_result_recovery() {
    let markdown = diagnostic_search_markdown_with_footer(
        "disease=qzvxxptl, source=gtr",
        &[],
        Some(0),
        "Showing 0 of 0 results.",
    )
    .expect("rendered markdown");

    assert!(markdown.contains("# Diagnostic tests: disease=qzvxxptl, source=gtr"));
    assert!(markdown.contains("No diagnostic tests found."));
    assert!(markdown.contains(
        "Try adjusting or removing diagnostic filters: --gene, --disease, --type, or --manufacturer."
    ));
    assert!(markdown.contains("See also:"));
    assert!(markdown.contains("biomcp list diagnostic"));
    assert!(markdown.contains("Showing 0 of 0 results."));
}

#[test]
fn diagnostic_search_markdown_does_not_render_recovery_for_high_offset_empty_page() {
    let markdown = diagnostic_search_markdown_with_footer(
        "disease=tuberculosis, source=gtr, offset=99",
        &[],
        Some(10),
        "Showing 0 of 10 results.",
    )
    .expect("rendered markdown");

    assert!(markdown.contains("No diagnostic tests found."));
    assert!(!markdown.contains("Try adjusting or removing diagnostic filters"));
    assert!(!markdown.contains("See also:"));
    assert!(!markdown.contains("biomcp list diagnostic"));
    assert!(markdown.contains("Showing 0 of 10 results."));
}

#[test]
fn diagnostic_search_rows_caps_genes_and_conditions_with_overflow_marker() {
    let result = DiagnosticSearchResult {
        source: "gtr".to_string(),
        accession: "GTR000000003.1".to_string(),
        name: "Broad Hereditary Cancer Panel".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer_or_lab: Some("PanCancer Pro".to_string()),
        genes: [
            "BRAF", "BRCA1", "BRCA2", "ATM", "PALB2", "CHEK2", "NBN", "CDH1", "STK11",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        conditions: [
            "Breast cancer",
            "Ovarian cancer",
            "Hereditary breast ovarian cancer syndrome",
            "Pancreatic cancer",
            "Lynch syndrome",
            "Colon cancer",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
    };

    let results = vec![result];
    let rows = diagnostic_search_rows(&results);
    assert_eq!(
        rows[0].genes_cell,
        "BRAF, BRCA1, BRCA2, ATM, PALB2, +4 more"
    );
    assert_eq!(
        rows[0].conditions_cell,
        "Breast cancer, Ovarian cancer, Hereditary breast ovarian cancer syndrome, Pancreatic cancer, Lynch syndrome, +1 more"
    );
}

#[test]
fn diagnostic_search_rows_escapes_markdown_table_cells() {
    let result = DiagnosticSearchResult {
        source: "gtr".to_string(),
        accession: "GTR000000004.1".to_string(),
        name: "Escaping Probe".to_string(),
        test_type: None,
        manufacturer_or_lab: None,
        genes: vec!["A|B".to_string()],
        conditions: vec!["Line\nBreak".to_string()],
    };

    let results = vec![result];
    let rows = diagnostic_search_rows(&results);
    assert_eq!(rows[0].genes_cell, "A\\|B");
    assert_eq!(rows[0].conditions_cell, "Line Break");
}

#[test]
fn diagnostic_markdown_renders_who_summary_fields_and_supported_sections_only() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "who-ivd".to_string(),
        source_id: "ITPW02232- TC40".to_string(),
        accession: "ITPW02232- TC40".to_string(),
        name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
        test_type: Some("Immunochromatographic (lateral flow)".to_string()),
        manufacturer: Some("InTec Products, Inc.".to_string()),
        target_marker: Some("HIV".to_string()),
        regulatory_version: Some("Rest-of-World".to_string()),
        prequalification_year: Some("2019".to_string()),
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: vec![],
        genes: None,
        conditions: Some(vec!["HIV".to_string()]),
        methods: None,
        regulatory: None,
    };

    let markdown =
        diagnostic_markdown(&diagnostic, &["all".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("Source: WHO Prequalified IVD"));
    assert!(markdown.contains("Assay Format: Immunochromatographic (lateral flow)"));
    assert!(markdown.contains("Target / Marker: HIV"));
    assert!(markdown.contains("Regulatory Version: Rest-of-World"));
    assert!(markdown.contains("Prequalification Year: 2019"));
    assert!(markdown.contains("## Conditions"));
    assert!(markdown.contains("HIV"));
    assert!(!markdown.contains("## Genes"));
    assert!(!markdown.contains("## Methods"));
    assert!(markdown.contains("biomcp list diagnostic"));
}

#[test]
fn diagnostic_markdown_renders_regulatory_section_rows() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "gtr".to_string(),
        source_id: "GTR000000001.1".to_string(),
        accession: "GTR000000001.1".to_string(),
        name: "FoundationOne CDx".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer: Some("Foundation Medicine, Inc.".to_string()),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: vec![],
        genes: None,
        conditions: None,
        methods: None,
        regulatory: Some(vec![DiagnosticRegulatoryRecord {
            submission_type: "PMA".to_string(),
            number: "P000019".to_string(),
            display_name: "FoundationOne CDx".to_string(),
            trade_name: Some("FoundationOne CDx".to_string()),
            generic_name: None,
            applicant: Some("Foundation Medicine, Inc.".to_string()),
            decision_date: Some("2017-11-30".to_string()),
            decision_description: Some("approved".to_string()),
            advisory_committee: None,
            product_code: Some("PQP".to_string()),
            supplement_count: Some(2),
        }]),
    };

    let markdown =
        diagnostic_markdown(&diagnostic, &["regulatory".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("## Regulatory (FDA Device)"));
    assert!(markdown.contains("| Type | Number | Name | Applicant | Decision Date | Decision | Product Code | Supplements |"));
    assert!(markdown.contains("| PMA | P000019 | FoundationOne CDx | Foundation Medicine, Inc. | 2017-11-30 | approved | PQP | 2 |"));
}

#[test]
fn diagnostic_markdown_keeps_regulatory_hidden_for_all_expansion() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "gtr".to_string(),
        source_id: "GTR000000001.1".to_string(),
        accession: "GTR000000001.1".to_string(),
        name: "FoundationOne CDx".to_string(),
        test_type: Some("molecular".to_string()),
        manufacturer: Some("Foundation Medicine, Inc.".to_string()),
        target_marker: None,
        regulatory_version: None,
        prequalification_year: None,
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: vec![],
        genes: None,
        conditions: None,
        methods: None,
        regulatory: Some(vec![DiagnosticRegulatoryRecord {
            submission_type: "PMA".to_string(),
            number: "P000019".to_string(),
            display_name: "FoundationOne CDx".to_string(),
            trade_name: Some("FoundationOne CDx".to_string()),
            generic_name: None,
            applicant: Some("Foundation Medicine, Inc.".to_string()),
            decision_date: Some("2017-11-30".to_string()),
            decision_description: Some("approved".to_string()),
            advisory_committee: None,
            product_code: Some("PQP".to_string()),
            supplement_count: Some(2),
        }]),
    };

    let markdown =
        diagnostic_markdown(&diagnostic, &["all".to_string()]).expect("rendered markdown");

    assert!(!markdown.contains("## Regulatory (FDA Device)"));
}

#[test]
fn diagnostic_markdown_renders_regulatory_empty_state_when_requested() {
    let diagnostic = Diagnostic {
        section_outcomes: crate::entities::diagnostic::default_diagnostic_section_outcomes(),
        source: "who-ivd".to_string(),
        source_id: "ITPW02232- TC40".to_string(),
        accession: "ITPW02232- TC40".to_string(),
        name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
        test_type: Some("Immunochromatographic (lateral flow)".to_string()),
        manufacturer: Some("InTec Products, Inc.".to_string()),
        target_marker: Some("HIV".to_string()),
        regulatory_version: Some("Rest-of-World".to_string()),
        prequalification_year: Some("2019".to_string()),
        laboratory: None,
        institution: None,
        country: None,
        clia_number: None,
        state_licenses: None,
        current_status: None,
        public_status: None,
        method_categories: vec![],
        genes: None,
        conditions: None,
        methods: None,
        regulatory: Some(vec![]),
    };

    let markdown =
        diagnostic_markdown(&diagnostic, &["regulatory".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("## Regulatory (FDA Device)"));
    assert!(markdown.contains("No FDA device 510(k) or PMA records matched this diagnostic."));
}
