#[test]
fn disease_markdown_section_only_shows_disgenet_section() {
    let disease = Disease {
        id: "MONDO:0007254".to_string(),
        name: "breast cancer".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: Some(crate::entities::disease::DiseaseDisgenet {
            associations: vec![crate::entities::disease::DiseaseDisgenetAssociation {
                symbol: "TP53".to_string(),
                entrez_id: Some(7157),
                score: 0.91,
                publication_count: Some(1234),
                clinical_trial_count: Some(4),
                evidence_index: Some(0.72),
                evidence_level: Some("Definitive".to_string()),
            }],
        }),
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let markdown =
        disease_markdown(&disease, &["disgenet".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("# breast cancer - disgenet"));
    assert!(markdown.contains("## DisGeNET"));
    assert!(markdown.contains("| Gene | Entrez ID | Score | PMIDs | Trials | EL | EI |"));
    assert!(markdown.contains("| TP53 | 7157 | 0.910 | 1234 | 4 | Definitive | 0.720 |"));
}

#[test]
fn disease_markdown_disgenet_renders_sparse_optional_fields() {
    let disease = Disease {
        id: "MONDO:0000001".to_string(),
        name: "sparse disease".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: Some(crate::entities::disease::DiseaseDisgenet {
            associations: vec![crate::entities::disease::DiseaseDisgenetAssociation {
                symbol: "KYNU".to_string(),
                entrez_id: None,
                score: 0.23,
                publication_count: None,
                clinical_trial_count: None,
                evidence_index: None,
                evidence_level: None,
            }],
        }),
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let markdown =
        disease_markdown(&disease, &["disgenet".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("| Gene | Entrez ID | Score | PMIDs | Trials | EL | EI |"));
    assert!(markdown.contains("| KYNU | - | 0.230 | - | - | - | - |"));
}

#[test]
fn disease_markdown_funding_renders_truthful_notes_without_table() {
    let mut disease = Disease {
        id: "MONDO:0007947".to_string(),
        name: "Marfan syndrome".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: Vec::new(),
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: Vec::new(),
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: Vec::new(),
        prevalence: Vec::new(),
        prevalence_note: None,
        survival: None,
        survival_note: None,
        civic: None,
        disgenet: None,
        funding: Some(crate::sources::nih_reporter::NihReporterFundingSection {
            query: "Marfan syndrome".to_string(),
            fiscal_years: vec![2022, 2023, 2024, 2025, 2026],
            matching_project_years: 0,
            grants: Vec::new(),
        }),
        funding_note: Some("No NIH funding data found for this query.".to_string()),
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let no_hit =
        disease_markdown(&disease, &["funding".to_string()]).expect("no-hit funding markdown");
    assert!(no_hit.contains("## Funding (NIH Reporter)"));
    assert!(no_hit.contains("No NIH funding data found for this query."));
    assert!(!no_hit.contains("| Project | PI | Organization | FY | Amount |"));

    disease.funding = None;
    disease.funding_note =
        Some("NIH Reporter funding data is temporarily unavailable.".to_string());

    let unavailable =
        disease_markdown(&disease, &["funding".to_string()]).expect("unavailable funding markdown");
    assert!(unavailable.contains("## Funding (NIH Reporter)"));
    assert!(unavailable.contains("NIH Reporter funding data is temporarily unavailable."));
    assert!(!unavailable.contains("| Project | PI | Organization | FY | Amount |"));
}

#[test]
fn disease_markdown_all_keeps_opt_in_sections_hidden() {
    let disease = disease_with_clinical_features();

    let markdown = disease_markdown(&disease, &["all".to_string()]).expect("all markdown");

    assert!(!markdown.contains("## Diagnostics"));
    assert!(!markdown.contains("## Funding (NIH Reporter)"));
    assert!(!markdown.contains("## DisGeNET"));
    assert!(!markdown.contains("## Clinical Features (Monarch / HPO)"));
}

#[test]
fn disease_search_empty_state_includes_discover_hint() {
    let markdown = disease_search_markdown_with_footer(
        "definitelynotarealdisease",
        "definitelynotarealdisease",
        &[],
        false,
        "",
    )
    .expect("markdown");

    assert!(markdown.contains("Try: biomcp discover definitelynotarealdisease"));
}

#[test]
fn disease_search_empty_state_uses_raw_query_in_discover_hint() {
    let markdown = disease_search_markdown_with_footer(
        "Arnold Chiari syndrome",
        "Arnold Chiari syndrome, offset=5",
        &[],
        false,
        "",
    )
    .expect("markdown");

    assert!(markdown.contains("Try: biomcp discover \"Arnold Chiari syndrome\""));
    assert!(!markdown.contains("offset=5\""));
}

#[test]
fn disease_search_fallback_renders_provenance_columns() {
    let markdown = disease_search_markdown_with_footer(
        "Arnold Chiari syndrome",
        "Arnold Chiari syndrome",
        &[DiseaseSearchResult {
            id: "MONDO:0000115".into(),
            name: "Arnold-Chiari malformation".into(),
            synonyms_preview: Some("Chiari malformation".into()),
            resolved_via: Some("MESH crosswalk".into()),
            source_id: Some("MESH:D001139".into()),
        }],
        true,
        "",
    )
    .expect("markdown");

    assert!(markdown.contains("Resolved via discover + crosswalk"));
    assert!(markdown.contains("| ID | Name | Resolved via | Source ID |"));
    assert!(markdown.contains("MESH crosswalk"));
    assert!(markdown.contains("MESH:D001139"));
}
