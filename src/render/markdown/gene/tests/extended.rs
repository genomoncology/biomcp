#[test]
fn gene_markdown_section_only_shows_constraint_section() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "TP53".to_string(),
        name: "tumor protein p53".to_string(),
        entrez_id: "7157".to_string(),
        ensembl_id: Some("ENSG00000141510".to_string()),
        location: Some("17p13.1".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P04637".to_string()),
        summary: Some("Tumor suppressor.".to_string()),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["P53".to_string()],
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
        constraint: Some(crate::entities::gene::GeneConstraint {
            pli: None,
            loeuf: None,
            mis_z: None,
            syn_z: None,
            transcript: Some("ENST00000269305".to_string()),
            source: "gnomAD".to_string(),
            source_version: "v4".to_string(),
            reference_genome: "GRCh38".to_string(),
        }),
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let markdown = gene_markdown(&gene, &["constraint".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("# TP53 - constraint"));
    assert!(markdown.contains("## Constraint (gnomAD)"));
    assert!(markdown.contains("Source: gnomAD"));
    assert!(markdown.contains("Version: v4"));
    assert!(markdown.contains("Reference genome: GRCh38"));
    assert!(markdown.contains("Transcript: ENST00000269305"));
    assert!(markdown.contains("No gnomAD constraint metrics returned for this gene query."));
}

#[test]
fn gene_markdown_section_only_shows_disgenet_section() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "TP53".to_string(),
        name: "tumor protein p53".to_string(),
        entrez_id: "7157".to_string(),
        ensembl_id: None,
        location: None,
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: None,
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
        disgenet: Some(crate::entities::gene::GeneDisgenet {
            associations: vec![crate::entities::gene::GeneDisgenetAssociation {
                disease_name: "Breast Carcinoma".to_string(),
                disease_cui: "C0678222".to_string(),
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
    };

    let markdown = gene_markdown(&gene, &["disgenet".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("# TP53 - disgenet"));
    assert!(markdown.contains("## DisGeNET"));
    assert!(markdown.contains("| Disease | UMLS CUI | Score | PMIDs | Trials | EL | EI |"));
    assert!(
        markdown
            .contains("| Breast Carcinoma | C0678222 | 0.910 | 1234 | 4 | Definitive | 0.720 |")
    );
}

#[test]
fn gene_markdown_disgenet_renders_sparse_optional_fields() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "KYNU".to_string(),
        name: "kynureninase".to_string(),
        entrez_id: "8942".to_string(),
        ensembl_id: None,
        location: None,
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: None,
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
        disgenet: Some(crate::entities::gene::GeneDisgenet {
            associations: vec![crate::entities::gene::GeneDisgenetAssociation {
                disease_name: "Sparse Disease".to_string(),
                disease_cui: "C1234567".to_string(),
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
    };

    let markdown = gene_markdown(&gene, &["disgenet".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("| Disease | UMLS CUI | Score | PMIDs | Trials | EL | EI |"));
    assert!(markdown.contains("| Sparse Disease | C1234567 | 0.230 | - | - | - | - |"));
}

#[test]
fn gene_markdown_funding_renders_linked_rows_and_currency() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "ERBB2".to_string(),
        name: "erb-b2 receptor tyrosine kinase 2".to_string(),
        entrez_id: "2064".to_string(),
        ensembl_id: None,
        location: None,
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: None,
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
        funding: Some(crate::sources::nih_reporter::NihReporterFundingSection {
            query: "ERBB2".to_string(),
            fiscal_years: vec![2022, 2023, 2024, 2025, 2026],
            matching_project_years: 176,
            grants: vec![crate::sources::nih_reporter::NihReporterGrant {
                project_title: "Regulation Of Epidermal Differentiation".to_string(),
                project_num: "1ZIAAR041124-23".to_string(),
                core_project_num: Some("ZIAAR041124".to_string()),
                project_detail_url: Some(
                    "https://reporter.nih.gov/project-details/10697688".to_string(),
                ),
                pi_name: Some("MORASSO, MARIA".to_string()),
                organization: Some(
                    "NATIONAL INSTITUTE OF ARTHRITIS AND MUSCULOSKELETAL AND SKIN DISEASES"
                        .to_string(),
                ),
                fiscal_year: 2022,
                award_amount: 2_219_287,
            }],
        }),
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let markdown = gene_markdown(&gene, &["funding".to_string()]).expect("funding markdown");
    let summary =
        "Showing top 1 unique grants from 176 matching NIH project-year records across FY2022-FY2026.";
    let row = "| [Regulation Of Epidermal Differentiation](https://reporter.nih.gov/project-details/10697688) | MORASSO, MARIA | NATIONAL INSTITUTE OF ARTHRITIS AND MUSCULOSKELETAL AND SKIN DISEASES | 2022 | $2,219,287 |";

    assert!(markdown.contains("# ERBB2 - funding"));
    assert!(markdown.contains("## Funding (NIH Reporter)"));
    assert!(markdown.contains(summary));
    assert!(markdown.contains("| Project | PI | Organization | FY | Amount |"));
    assert!(markdown.contains(
        "[Regulation Of Epidermal Differentiation](https://reporter.nih.gov/project-details/10697688)"
    ));
    assert!(markdown.contains("| MORASSO, MARIA |"));
    assert!(markdown.contains("| 2022 | $2,219,287 |"));
    assert!(
        markdown
            .find(summary)
            .expect("funding summary should render")
            > markdown.find(row).expect("funding row should render")
    );
}

#[test]
fn gene_markdown_all_keeps_opt_in_sections_hidden() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "ERBB2".to_string(),
        name: "erb-b2 receptor tyrosine kinase 2".to_string(),
        entrez_id: "2064".to_string(),
        ensembl_id: None,
        location: None,
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: None,
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

    let markdown = gene_markdown(&gene, &["all".to_string()]).expect("all markdown");

    assert!(!markdown.contains("## Diagnostics"));
    assert!(!markdown.contains("## Funding (NIH Reporter)"));
    assert!(!markdown.contains("## DisGeNET"));
}

#[test]
fn gene_markdown_pathways_show_source_labels() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: None,
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: None,
        summary: None,
        gene_type: None,
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: Some(vec![
            crate::entities::gene::GenePathway {
                source: "KEGG".to_string(),
                id: "hsa05200".to_string(),
                name: "Pathways in cancer".to_string(),
            },
            crate::entities::gene::GenePathway {
                source: "Reactome".to_string(),
                id: "R-HSA-5673001".to_string(),
                name: "RAF/MAP kinase cascade".to_string(),
            },
        ]),
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

    let markdown = gene_markdown(&gene, &[]).expect("rendered markdown");
    assert!(markdown.contains("| Source | ID | Name |"));
    assert!(markdown.contains("| KEGG | hsa05200 | Pathways in cancer |"));
    assert!(markdown.contains("| Reactome | R-HSA-5673001 | RAF/MAP kinase cascade |"));
    assert!(!markdown.contains("Showing pathway rows from Reactome search results."));
}
