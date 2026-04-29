use super::*;

#[test]
fn sections_pathway_for_kegg_excludes_unsupported_sections() {
    let pathway = Pathway {
        source: "KEGG".to_string(),
        id: "hsa05200".to_string(),
        name: "Pathways in cancer".to_string(),
        species: None,
        summary: None,
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let sections = sections_pathway(&pathway, &[]);
    assert_eq!(sections, vec!["genes".to_string()]);
}

#[test]
fn sections_diagnostic_omit_requested_section_from_more_block() {
    let diagnostic = Diagnostic {
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
        genes: Some(vec!["BRCA1".to_string()]),
        conditions: Some(vec!["Breast cancer".to_string()]),
        methods: Some(vec!["Sequence analysis".to_string()]),
        regulatory: None,
    };

    let sections = sections_diagnostic(&diagnostic, &["genes".to_string()]);
    assert_eq!(
        sections,
        vec![
            "conditions".to_string(),
            "methods".to_string(),
            "regulatory".to_string()
        ]
    );

    let commands = diagnostic_next_commands(&diagnostic, &["genes".to_string()]);
    assert_eq!(
        commands,
        vec![
            "biomcp get diagnostic GTR000000001.1 conditions".to_string(),
            "biomcp get diagnostic GTR000000001.1 methods".to_string(),
            "biomcp get diagnostic GTR000000001.1 regulatory".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
}

#[test]
fn sections_diagnostic_for_who_only_offer_conditions_and_quote_accession() {
    let diagnostic = Diagnostic {
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

    assert_eq!(
        sections_diagnostic(&diagnostic, &[]),
        vec!["conditions".to_string(), "regulatory".to_string()]
    );
    assert_eq!(
        diagnostic_next_commands(&diagnostic, &[]),
        vec![
            "biomcp get diagnostic \"ITPW02232- TC40\" conditions".to_string(),
            "biomcp get diagnostic \"ITPW02232- TC40\" regulatory".to_string(),
            "biomcp list diagnostic".to_string()
        ]
    );
}

#[test]
fn diagnostic_more_block_keeps_four_visible_section_commands() {
    let diagnostic = Diagnostic {
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

    let block = format_sections_block(
        "diagnostic",
        &diagnostic.accession,
        sections_diagnostic(&diagnostic, &[]),
    );
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 genes"));
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 conditions"));
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 methods"));
    assert!(block.contains("biomcp get diagnostic GTR000000001.1 regulatory"));
}

#[test]
fn sections_pathway_for_reactome_keeps_full_supported_set() {
    let pathway = Pathway {
        source: "Reactome".to_string(),
        id: "R-HSA-5673001".to_string(),
        name: "RAF/MAP kinase cascade".to_string(),
        species: None,
        summary: None,
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    };

    let sections = sections_pathway(&pathway, &[]);
    assert_eq!(
        sections,
        vec![
            "genes".to_string(),
            "events".to_string(),
            "enrichment".to_string()
        ]
    );
}

#[test]
fn format_sections_block_renders_described_executable_commands() {
    let block = format_sections_block(
        "gene",
        "TP53",
        vec![
            "pathways".to_string(),
            "hpa".to_string(),
            "diseases".to_string(),
            "protein".to_string(),
        ],
    );

    assert!(block.contains("More:"));
    assert!(block.contains("biomcp get gene TP53 pathways"));
    assert!(block.contains("Reactome/KEGG pathway context"));
    assert!(block.contains("biomcp get gene TP53 hpa"));
    assert!(block.contains("Human Protein Atlas tissue expression and localization"));
    assert!(block.contains("biomcp get gene TP53 diseases"));
    assert!(block.contains("disease associations"));
    assert!(block.contains("All:"));
    assert!(block.contains("biomcp get gene TP53 all"));
}

#[test]
fn format_sections_block_keeps_gene_ontology_in_top_more_entries() {
    let block = format_sections_block(
        "gene",
        "NANOG",
        vec![
            "pathways".to_string(),
            "ontology".to_string(),
            "diseases".to_string(),
            "protein".to_string(),
        ],
    );

    let pathways = block
        .find("biomcp get gene NANOG pathways")
        .expect("pathways command");
    let ontology = block
        .find("biomcp get gene NANOG ontology")
        .expect("ontology command");
    let diseases = block
        .find("biomcp get gene NANOG diseases")
        .expect("diseases command");
    assert!(pathways < ontology);
    assert!(ontology < diseases);
}

#[test]
fn sections_disease_base_card_surfaces_diagnostics_before_optional_sections() {
    let disease = Disease {
        id: "MONDO:0005105".to_string(),
        name: "melanoma".to_string(),
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
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
        xrefs: std::collections::HashMap::new(),
    };

    let sections = sections_disease(&disease, &[]);
    assert_eq!(
        sections
            .iter()
            .take(6)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec![
            "genes",
            "pathways",
            "phenotypes",
            "diagnostics",
            "clinical_features",
            "survival"
        ]
    );

    let block = format_sections_block("disease", &disease.id, sections);
    let genes = block
        .find("biomcp get disease MONDO:0005105 genes")
        .expect("genes command");
    let pathways = block
        .find("biomcp get disease MONDO:0005105 pathways")
        .expect("pathways command");
    let phenotypes = block
        .find("biomcp get disease MONDO:0005105 phenotypes")
        .expect("phenotypes command");
    let diagnostics = block
        .find("biomcp get disease MONDO:0005105 diagnostics")
        .expect("diagnostics command");
    let survival = block
        .find("biomcp get disease MONDO:0005105 survival")
        .expect("survival command");
    let clinical_features = block
        .find("biomcp get disease MONDO:0005105 clinical_features")
        .expect("clinical features command");
    assert!(genes < pathways);
    assert!(pathways < phenotypes);
    assert!(phenotypes < diagnostics);
    assert!(diagnostics < clinical_features);
    assert!(clinical_features < survival);
    assert!(block.contains("diagnostic tests for this condition from GTR and WHO IVD"));
    assert!(block.contains("MedlinePlus clinical feature summaries"));
    assert!(block.contains("SEER Explorer cancer survival rates"));
    assert!(!block.contains("biomcp get disease MONDO:0005105 variants"));
}

#[test]
fn sections_gene_base_card_surfaces_diagnostics_as_fourth_command() {
    let gene = Gene {
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: Some("164757".to_string()),
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: None,
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: None,
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let sections = sections_gene(&gene, &[]);
    assert_eq!(
        sections
            .iter()
            .take(4)
            .map(String::as_str)
            .collect::<Vec<_>>(),
        vec!["pathways", "ontology", "diseases", "diagnostics"]
    );

    let block = format_sections_block("gene", &gene.symbol, sections);
    let pathways = block
        .find("biomcp get gene BRAF pathways")
        .expect("pathways command");
    let ontology = block
        .find("biomcp get gene BRAF ontology")
        .expect("ontology command");
    let diseases = block
        .find("biomcp get gene BRAF diseases")
        .expect("diseases command");
    let diagnostics = block
        .find("biomcp get gene BRAF diagnostics")
        .expect("diagnostics command");
    assert!(pathways < ontology);
    assert!(ontology < diseases);
    assert!(diseases < diagnostics);
    assert!(block.contains("diagnostic tests for this gene from GTR"));
    assert!(!block.contains("biomcp get gene BRAF protein"));
}

#[test]
fn format_sections_block_describes_guardrailed_drug_and_trial_sections() {
    let drug_block = format_sections_block(
        "drug",
        "pembrolizumab",
        vec![
            "label".to_string(),
            "regulatory".to_string(),
            "safety".to_string(),
        ],
    );

    assert!(drug_block.contains(
            "biomcp get drug pembrolizumab label   - approved-indication and FDA label detail beyond the base card"
        ));
    assert!(drug_block.contains(
            "biomcp get drug pembrolizumab regulatory   - approval and supplement history; use only if the base card lacks approval context"
        ));
    assert!(drug_block.contains(
            "biomcp get drug pembrolizumab safety   - regulatory safety detail; use `biomcp drug adverse-events <name>` first when you want post-marketing signal"
        ));

    let terminated = crate::entities::trial::Trial {
        nct_id: "NCT02576665".to_string(),
        source: None,
        title: "Completed trial".to_string(),
        status: "TERMINATED".to_string(),
        phase: None,
        study_type: None,
        age_range: None,
        conditions: vec!["melanoma".to_string()],
        interventions: vec!["trametinib".to_string()],
        intervention_details: Vec::new(),
        sponsor: None,
        enrollment: None,
        summary: None,
        start_date: None,
        completion_date: None,
        eligibility_text: None,
        locations: None,
        outcomes: None,
        arms: None,
        references: None,
    };
    let terminated_sections = sections_trial(&terminated, &[]);
    assert_eq!(terminated_sections[0], "outcomes");
    assert_eq!(terminated_sections[1], "references");
    assert_eq!(terminated_sections[2], "arms");

    let trial_block =
        format_sections_block("trial", &terminated.nct_id, terminated_sections.clone());
    assert!(
        trial_block.contains(
            "biomcp get trial NCT02576665 outcomes   - endpoint measures and time frames"
        )
    );
    assert!(trial_block.contains(
        "biomcp get trial NCT02576665 references   - linked publications and PMID citations"
    ));
    assert!(
        trial_block.contains(
            "biomcp get trial NCT02576665 arms   - study arms and assigned interventions"
        )
    );

    let recruiting = crate::entities::trial::Trial {
        status: "Recruiting".to_string(),
        ..terminated
    };
    let recruiting_sections = sections_trial(&recruiting, &[]);
    assert_eq!(recruiting_sections[0], "eligibility");
    assert_eq!(recruiting_sections[1], "locations");
    assert_eq!(recruiting_sections[2], "outcomes");
}
