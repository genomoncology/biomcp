use super::*;
use crate::entities::diagnostic::{Diagnostic, DiagnosticRegulatoryRecord, DiagnosticSearchResult};

#[test]
fn diagnostic_json_next_commands_parse() {
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
        genes: Some(vec!["BRCA1".to_string(), "BARD1".to_string()]),
        conditions: None,
        methods: None,
        regulatory: None,
    };
    let requested_sections = ["genes".to_string()];
    let next_commands =
        crate::render::markdown::diagnostic_next_commands(&diagnostic, &requested_sections);

    assert!(
        !next_commands.contains(&"biomcp get diagnostic GTR000000001.1 genes".to_string()),
        "requested section should not be suggested again: {next_commands:?}"
    );
    assert!(next_commands.contains(&"biomcp get diagnostic GTR000000001.1 conditions".to_string()));
    assert!(next_commands.contains(&"biomcp get diagnostic GTR000000001.1 methods".to_string()));
    assert!(next_commands.contains(&"biomcp get diagnostic GTR000000001.1 regulatory".to_string()));
    assert!(next_commands.contains(&"biomcp list diagnostic".to_string()));

    assert_entity_json_next_commands(
        "diagnostic",
        &diagnostic,
        crate::render::markdown::diagnostic_evidence_urls(&diagnostic),
        next_commands,
        crate::render::provenance::diagnostic_section_sources(&diagnostic),
    );
}

#[test]
fn diagnostic_search_json_next_commands_parse() {
    let results = vec![DiagnosticSearchResult {
        source: "who-ivd".to_string(),
        accession: "ITPW02232- TC40".to_string(),
        name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
        test_type: Some("Immunochromatographic (lateral flow)".to_string()),
        manufacturer_or_lab: Some("InTec Products, Inc.".to_string()),
        genes: vec![],
        conditions: vec!["HIV".to_string()],
    }];
    let pagination = crate::cli::PaginationMeta::offset(0, 10, results.len(), Some(results.len()));
    let json = crate::cli::search_json_with_meta(
        results.clone(),
        pagination,
        crate::render::markdown::search_next_commands_diagnostic(&results),
    )
    .expect("diagnostic search json");
    let commands = collect_next_commands(&json);

    assert_eq!(
        commands,
        vec![
            "biomcp get diagnostic \"ITPW02232- TC40\"".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
    assert_json_next_commands_parse("diagnostic-search", &json);
}

#[test]
fn diagnostic_json_next_commands_quote_who_follow_up() {
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
        regulatory: None,
    };
    let requested_sections = ["conditions".to_string()];
    let next_commands =
        crate::render::markdown::diagnostic_next_commands(&diagnostic, &requested_sections);

    assert!(
        next_commands.contains(&"biomcp get diagnostic \"ITPW02232- TC40\" regulatory".to_string())
    );

    assert_entity_json_next_commands(
        "diagnostic",
        &diagnostic,
        crate::render::markdown::diagnostic_evidence_urls(&diagnostic),
        next_commands,
        crate::render::provenance::diagnostic_section_sources(&diagnostic),
    );
}

#[test]
fn diagnostic_json_next_commands_keep_four_visible_gtr_sections() {
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

    let next_commands = crate::render::markdown::diagnostic_next_commands(&diagnostic, &[]);
    assert_eq!(
        next_commands[..5],
        [
            "biomcp get diagnostic GTR000000001.1 genes".to_string(),
            "biomcp get diagnostic GTR000000001.1 conditions".to_string(),
            "biomcp get diagnostic GTR000000001.1 methods".to_string(),
            "biomcp get diagnostic GTR000000001.1 regulatory".to_string(),
            "biomcp list diagnostic".to_string(),
        ]
    );
}

#[test]
fn diagnostic_json_omits_regulatory_field_when_unrequested() {
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

    let value = crate::render::json::to_entity_json_value(
        &diagnostic,
        crate::render::markdown::diagnostic_evidence_urls(&diagnostic),
        crate::render::markdown::diagnostic_next_commands(&diagnostic, &[]),
        crate::render::provenance::diagnostic_section_sources(&diagnostic),
    )
    .expect("diagnostic json value");

    assert!(value.get("regulatory").is_none());
    assert!(
        !value["_meta"]["section_sources"]
            .as_array()
            .expect("section_sources array")
            .iter()
            .any(|source| source["key"] == "regulatory")
    );
}

#[test]
fn diagnostic_json_includes_regulatory_field_and_provenance_when_requested() {
    let mut section_outcomes = crate::entities::diagnostic::default_diagnostic_section_outcomes();
    section_outcomes.complete(
        "regulatory",
        crate::entities::section_outcome::SectionOutcome::data("OpenFDA Device 510(k) / PMA"),
    );
    let diagnostic = Diagnostic {
        section_outcomes,
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
        regulatory: Some(vec![DiagnosticRegulatoryRecord {
            submission_type: "510(k)".to_string(),
            number: "K123456".to_string(),
            display_name: "ONE STEP Anti-HIV (1&2) Test".to_string(),
            trade_name: None,
            generic_name: None,
            applicant: Some("InTec Products, Inc.".to_string()),
            decision_date: Some("2023-08-01".to_string()),
            decision_description: Some("cleared".to_string()),
            advisory_committee: None,
            product_code: Some("NPR".to_string()),
            supplement_count: None,
        }]),
    };

    let value = crate::render::json::to_entity_json_value(
        &diagnostic,
        crate::render::markdown::diagnostic_evidence_urls(&diagnostic),
        crate::render::markdown::diagnostic_next_commands(&diagnostic, &["regulatory".to_string()]),
        crate::render::provenance::diagnostic_section_sources(&diagnostic),
    )
    .expect("diagnostic json value");

    assert_eq!(value["regulatory"][0]["number"], "K123456");
    let regulatory_source = value["_meta"]["section_sources"]
        .as_array()
        .expect("section_sources array")
        .iter()
        .find(|source| source["key"] == "regulatory")
        .expect("regulatory section source");
    assert_eq!(regulatory_source["label"], "Regulatory");
    assert_eq!(
        regulatory_source["sources"][0],
        "OpenFDA Device 510(k) / PMA"
    );
}
