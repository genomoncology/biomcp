#[test]
fn gene_markdown_includes_evidence_links() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P15056".to_string()),
        summary: Some("Kinase involved in MAPK signaling.".to_string()),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["BRAF1".to_string()],
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

    let markdown = gene_markdown(&gene, &[]).expect("rendered markdown");
    assert!(markdown.contains("BRAF"));
    assert!(markdown.contains("[NCBI Gene](https://www.ncbi.nlm.nih.gov/gene/673)"));
    assert!(markdown.contains("[UniProt](https://www.uniprot.org/uniprot/P15056)"));
}

#[test]
fn ticket_406_coordinate_outputs_carry_genome_build_context() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: Some(crate::entities::GenomicCoordinate {
            coordinate: "7:140719327-140924976 (strand: -1)".into(),
            genome_build: "GRCh38".into(),
            source: "test".into(),
            provenance: Some("MyGene.info provider default".into()),
        }),
        omim_id: None,
        uniprot_id: Some("P15056".to_string()),
        summary: Some("Kinase involved in MAPK signaling.".to_string()),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["BRAF1".to_string()],
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

    let markdown = gene_markdown(&gene, &[]).expect("rendered markdown");

    assert!(
        markdown.contains("GRCh") && markdown.contains("7:140719327-140924976"),
        "coordinate output must include explicit genome-build context with the position, got {markdown:?}"
    );

    let search_result = GeneSearchResult {
        symbol: gene.symbol,
        name: gene.name,
        entrez_id: gene.entrez_id,
        genomic_coordinates: gene.genomic_coordinates,
        uniprot_id: gene.uniprot_id,
        omim_id: gene.omim_id,
    };
    let search_markdown =
        gene_search_markdown("BRAF", &[search_result]).expect("rendered search markdown");

    assert!(
        search_markdown.contains("| Coordinate | Build |")
            && search_markdown.contains("7:140719327-140924976")
            && search_markdown.contains("| GRCh38 |"),
        "gene search coordinate output must include explicit genome-build context with the position, got {search_markdown:?}"
    );
}

#[test]
fn gene_markdown_section_only_shows_new_gene_enrichment_sections() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P15056".to_string()),
        summary: Some("Kinase involved in MAPK signaling.".to_string()),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["BRAF1".to_string()],
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

    let markdown = gene_markdown(
        &gene,
        &[
            "expression".to_string(),
            "hpa".to_string(),
            "druggability".to_string(),
            "clingen".to_string(),
        ],
    )
    .expect("rendered markdown");

    assert!(markdown.contains("# BRAF - expression, hpa, druggability, clingen"));
    assert!(markdown.contains("## Expression (GTEx)"));
    assert!(markdown.contains("## Human Protein Atlas"));
    assert!(markdown.contains("## Druggability"));
    assert!(markdown.contains("## ClinGen"));
    assert!(markdown.contains("No GTEx expression records returned"));
    assert!(markdown.contains("No Human Protein Atlas records returned"));
    assert!(markdown.contains("No DGIdb interactions returned"));
    assert!(markdown.contains("No ClinGen records returned"));
}

#[test]
fn gene_markdown_renders_combined_dgidb_and_opentargets_druggability() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "EGFR".to_string(),
        name: "epidermal growth factor receptor".to_string(),
        entrez_id: "1956".to_string(),
        ensembl_id: Some("ENSG00000146648".to_string()),
        location: Some("7p11.2".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P00533".to_string()),
        summary: None,
        gene_type: Some("protein-coding".to_string()),
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
        druggability: Some(crate::sources::dgidb::GeneDruggability {
            categories: vec!["Kinase".to_string()],
            interactions: Vec::new(),
            tractability: vec![
                crate::sources::dgidb::GeneTractabilityModality {
                    modality: "small molecule".to_string(),
                    tractable: true,
                    evidence_labels: vec!["Approved Drug".to_string()],
                },
                crate::sources::dgidb::GeneTractabilityModality {
                    modality: "antibody".to_string(),
                    tractable: true,
                    evidence_labels: vec!["Clinical Precedence".to_string()],
                },
            ],
            safety_liabilities: vec![crate::sources::dgidb::GeneSafetyLiability {
                event: "Skin rash".to_string(),
                datasource: Some("ForceGenetics".to_string()),
                effect_direction: Some("activation".to_string()),
                biosample: Some("Skin".to_string()),
            }],
        }),
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let markdown = gene_markdown(&gene, &["druggability".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("## Druggability"));
    assert!(!markdown.contains("## Druggability (DGIdb)"));
    assert!(markdown.contains("OpenTargets tractability"));
    assert!(markdown.contains("| Modality | Tractable | Evidence |"));
    assert!(markdown.contains("| small molecule | yes | Approved Drug |"));
    assert!(markdown.contains("| antibody | yes | Clinical Precedence |"));
    assert!(markdown.contains("OpenTargets safety liabilities"));
    assert!(markdown.contains("Skin rash"));
    assert!(markdown.contains("No DGIdb interactions returned for this gene query."));
}

#[test]
fn gene_markdown_renders_dgidb_interaction_table_alongside_opentargets_data() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
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
        druggability: Some(crate::sources::dgidb::GeneDruggability {
            categories: vec!["Kinase".to_string()],
            interactions: vec![crate::sources::dgidb::DrugInteraction {
                drug: "Dabrafenib".to_string(),
                interaction_types: vec!["inhibitor".to_string()],
                score: Some(1.2),
                approved: Some(true),
                source_count: 2,
            }],
            tractability: vec![crate::sources::dgidb::GeneTractabilityModality {
                modality: "small molecule".to_string(),
                tractable: true,
                evidence_labels: vec!["Approved Drug".to_string()],
            }],
            safety_liabilities: Vec::new(),
        }),
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let markdown = gene_markdown(&gene, &["druggability".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("## Druggability"));
    assert!(markdown.contains("OpenTargets tractability"));
    assert!(markdown.contains("| small molecule | yes | Approved Drug |"));
    assert!(markdown.contains("| Drug | Interaction Types | Score | Approved | Sources |"));
    assert!(markdown.contains("| Dabrafenib | inhibitor | 1.200 | yes | 2 |"));
    assert!(!markdown.contains("No DGIdb interactions returned for this gene query."));
}

#[test]
fn gene_markdown_renders_hpa_section_details() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P15056".to_string()),
        summary: Some("Kinase involved in MAPK signaling.".to_string()),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["BRAF1".to_string()],
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
        hpa: Some(crate::sources::hpa::GeneHpa {
            tissues: vec![
                crate::sources::hpa::HpaTissueExpression {
                    tissue: "Liver".to_string(),
                    level: "High".to_string(),
                },
                crate::sources::hpa::HpaTissueExpression {
                    tissue: "Kidney".to_string(),
                    level: "Medium".to_string(),
                },
            ],
            subcellular_main_location: vec!["cytosol".to_string(), "vesicles".to_string()],
            subcellular_additional_location: vec!["plasma membrane".to_string()],
            reliability: Some("Supported".to_string()),
            protein_summary: Some("Ubiquitous cytoplasmic expression.".to_string()),
            rna_summary: Some("Low tissue specificity; Detected in all".to_string()),
        }),
        druggability: None,
        clingen: None,
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let summary = gene_markdown(&gene, &[]).expect("rendered markdown");
    assert!(summary.contains(
        "Aliases are alternate names used in literature and databases when a paper does not use the HGNC symbol."
    ));

    let markdown = gene_markdown(&gene, &["hpa".to_string()]).expect("rendered markdown");

    assert!(markdown.contains("# BRAF - hpa"));
    assert!(markdown.contains("## Human Protein Atlas"));
    assert!(markdown.contains("Reliability: Supported"));
    assert!(markdown.contains("Protein summary: Ubiquitous cytoplasmic expression."));
    assert!(markdown.contains("RNA summary: Low tissue specificity; Detected in all"));
    assert!(markdown.contains("Subcellular main locations: cytosol, vesicles"));
    assert!(markdown.contains("Subcellular additional locations: plasma membrane"));
    assert!(markdown.contains("| Tissue | Level |"));
    assert!(markdown.contains("| Liver | High |"));
    assert!(markdown.contains("| Kidney | Medium |"));
    assert!(
        markdown
            .find("| Tissue | Level |")
            .expect("tissue table should render")
            < markdown
                .find("RNA summary:")
                .expect("rna summary should render")
    );
}

#[test]
fn gene_markdown_renders_protein_isoforms_with_count_and_displayed_length() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "KRAS".to_string(),
        name: "KRAS proto-oncogene, GTPase".to_string(),
        entrez_id: "3845".to_string(),
        ensembl_id: Some("ENSG00000133703".to_string()),
        location: Some("12p12.1".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P01116".to_string()),
        summary: None,
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["K-RAS2A".to_string(), "K-RAS2B".to_string()],
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "P01116".to_string(),
            name: "GTPase KRas".to_string(),
            function: Some("Small GTPase involved in signal transduction.".to_string()),
            length: Some(189),
            isoforms: vec![
                crate::entities::gene::GeneProteinIsoform {
                    name: "K-Ras4A".to_string(),
                    length: Some(189),
                },
                crate::entities::gene::GeneProteinIsoform {
                    name: "K-Ras4B".to_string(),
                    length: None,
                },
            ],
            alternative_names: Vec::new(),
        }),
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

    let markdown = gene_markdown(&gene, &["protein".to_string()]).expect("gene markdown");
    assert!(markdown.contains("## Protein (UniProt)"));
    assert!(markdown.contains("- Length: 189 aa"));
    assert!(markdown.contains("- Isoforms (2): K-Ras4A (189 aa), K-Ras4B"));
    assert!(markdown.contains("- Function: Small GTPase involved in signal transduction."));
}

#[test]
fn gene_markdown_without_isoforms_keeps_protein_lines_contiguous() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene, serine/threonine kinase".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: Some("protein-coding".to_string()),
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "P15056".to_string(),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            function: Some("Kinase function.".to_string()),
            length: Some(766),
            isoforms: Vec::new(),
            alternative_names: Vec::new(),
        }),
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

    let markdown = gene_markdown(&gene, &["protein".to_string()]).expect("gene markdown");
    assert!(markdown.contains("- Length: 766 aa\n- Function: Kinase function."));
    assert!(!markdown.contains("- Isoforms ("));
    assert!(!markdown.contains("- Length: 766 aa\n\n- Function:"));
}

#[test]
fn gene_markdown_renders_protein_alternative_names() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "PLIN2".to_string(),
        name: "perilipin 2".to_string(),
        entrez_id: "123".to_string(),
        ensembl_id: Some("ENSG00000147889".to_string()),
        location: Some("9p22.1".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("Q99541".to_string()),
        summary: None,
        gene_type: Some("protein-coding".to_string()),
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "Q99541".to_string(),
            name: "Perilipin-2".to_string(),
            function: None,
            length: Some(437),
            isoforms: Vec::new(),
            alternative_names: vec![
                "Adipophilin".to_string(),
                "ADRP".to_string(),
                "Adipose differentiation-related protein".to_string(),
            ],
        }),
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

    let markdown = gene_markdown(&gene, &["protein".to_string()]).expect("gene markdown");
    assert!(
        markdown.contains(
            "- Also known as: Adipophilin, ADRP, Adipose differentiation-related protein"
        )
    );
}

#[test]
fn gene_markdown_preserves_full_protein_function_text() {
    let long_function = "Mitochondrial dynamin-like GTPase required for fusion. Localizes to the intermembrane space where it helps organize cristae architecture and mitochondrial DNA maintenance.".to_string();
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "OPA1".to_string(),
        name: "OPA1 mitochondrial dynamin like GTPase".to_string(),
        entrez_id: "4976".to_string(),
        ensembl_id: Some("ENSG00000198836".to_string()),
        location: Some("3q29".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("O60313".to_string()),
        summary: None,
        gene_type: Some("protein-coding".to_string()),
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "O60313".to_string(),
            name: "Dynamin-like 120 kDa protein, mitochondrial".to_string(),
            function: Some(long_function.clone()),
            length: Some(960),
            isoforms: Vec::new(),
            alternative_names: Vec::new(),
        }),
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

    let markdown = gene_markdown(&gene, &["protein".to_string()]).expect("gene markdown");
    assert!(markdown.contains(&long_function));
    assert!(markdown.contains("intermembrane space"));
    assert!(!markdown.contains("architecture and mitochondrial DNA…"));
}

#[test]
fn gene_markdown_omits_protein_alternative_names_when_absent() {
    let gene = Gene {
        section_outcomes: Default::default(),
        symbol: "BRAF".to_string(),
        name: "B-Raf proto-oncogene, serine/threonine kinase".to_string(),
        entrez_id: "673".to_string(),
        ensembl_id: Some("ENSG00000157764".to_string()),
        location: Some("7q34".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("P15056".to_string()),
        summary: None,
        gene_type: Some("protein-coding".to_string()),
        aliases: Vec::new(),
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "P15056".to_string(),
            name: "Serine/threonine-protein kinase B-raf".to_string(),
            function: Some("Kinase function.".to_string()),
            length: Some(766),
            isoforms: Vec::new(),
            alternative_names: Vec::new(),
        }),
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

    let markdown = gene_markdown(&gene, &["protein".to_string()]).expect("gene markdown");
    assert!(!markdown.contains("- Also known as:"));
}
