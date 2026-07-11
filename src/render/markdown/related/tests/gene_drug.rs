#[test]
fn related_gene_prioritizes_localization_deepening_when_supported() {
    let gene = Gene {
        symbol: "OPA1".to_string(),
        name: "OPA1 mitochondrial dynamin like GTPase".to_string(),
        entrez_id: "4976".to_string(),
        ensembl_id: Some("ENSG00000198836".to_string()),
        location: Some("3q29".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("O60313".to_string()),
        summary: Some(
            "Mitochondrial inner membrane fusion GTPase required for cristae organization."
                .to_string(),
        ),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["large GTPase 1".to_string()],
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "O60313".to_string(),
            name: "Dynamin-like 120 kDa protein, mitochondrial".to_string(),
            function: None,
            length: None,
            isoforms: Vec::new(),
            alternative_names: Vec::new(),
        }),
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: Some(crate::sources::hpa::GeneHpa {
            tissues: vec![crate::sources::hpa::HpaTissueExpression {
                tissue: "Retina".to_string(),
                level: "High".to_string(),
            }],
            subcellular_main_location: vec!["Mitochondria".to_string()],
            subcellular_additional_location: Vec::new(),
            reliability: Some("Approved".to_string()),
            protein_summary: None,
            rna_summary: None,
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

    let related = related_gene(&gene);
    assert_eq!(related[0], "biomcp get gene OPA1 protein");
    assert_eq!(related[1], "biomcp get gene OPA1 hpa");
    assert!(related.contains(&"biomcp search pgx -g OPA1".to_string()));
    assert!(related.contains(&"biomcp search variant -g OPA1".to_string()));
    assert!(related.contains(&"biomcp search diagnostic --gene OPA1".to_string()));
}

#[test]
fn related_gene_promotes_clingen_trial_search_before_generic_pivots() {
    let gene = Gene {
        symbol: "OPA1".to_string(),
        name: "OPA1 mitochondrial dynamin like GTPase".to_string(),
        entrez_id: "4976".to_string(),
        ensembl_id: Some("ENSG00000198836".to_string()),
        location: Some("3q29".to_string()),
        genomic_coordinates: None,
        omim_id: None,
        uniprot_id: Some("O60313".to_string()),
        summary: Some(
            "Mitochondrial inner membrane fusion GTPase required for cristae organization."
                .to_string(),
        ),
        gene_type: Some("protein-coding".to_string()),
        aliases: vec!["large GTPase 1".to_string()],
        clinical_diseases: Vec::new(),
        clinical_drugs: Vec::new(),
        pathways: None,
        ontology: None,
        diseases: None,
        protein: Some(crate::entities::gene::GeneProtein {
            accession: "O60313".to_string(),
            name: "Dynamin-like 120 kDa protein, mitochondrial".to_string(),
            function: None,
            length: None,
            isoforms: Vec::new(),
            alternative_names: Vec::new(),
        }),
        go: None,
        interactions: None,
        civic: None,
        expression: None,
        hpa: Some(crate::sources::hpa::GeneHpa {
            tissues: vec![crate::sources::hpa::HpaTissueExpression {
                tissue: "Retina".to_string(),
                level: "High".to_string(),
            }],
            subcellular_main_location: vec!["Mitochondria".to_string()],
            subcellular_additional_location: Vec::new(),
            reliability: Some("Approved".to_string()),
            protein_summary: None,
            rna_summary: None,
        }),
        druggability: None,
        clingen: Some(crate::sources::clingen::GeneClinGen {
            validity: vec![crate::sources::clingen::ClinGenValidity {
                disease: "dominant optic atrophy".to_string(),
                classification: "Definitive".to_string(),
                review_date: Some("2024-01-01".to_string()),
                moi: Some("AD".to_string()),
            }],
            haploinsufficiency: None,
            triplosensitivity: None,
        }),
        constraint: None,
        disgenet: None,
        funding: None,
        funding_note: None,
        diagnostics: None,
        diagnostics_note: None,
    };

    let related = related_gene(&gene);
    assert_eq!(related[0], "biomcp get gene OPA1 protein");
    assert_eq!(related[1], "biomcp get gene OPA1 hpa");
    assert_eq!(
        related[2],
        "biomcp search trial -c \"dominant optic atrophy\" -s recruiting"
    );
    assert_eq!(
        related_command_description(&related[2]),
        Some("recruiting trials for the top ClinGen disease on this gene card")
    );
    assert!(related.contains(&"biomcp search pgx -g OPA1".to_string()));
    assert!(related.contains(&"biomcp search diagnostic --gene OPA1".to_string()));
}

#[test]
fn related_drug_includes_pgx_search() {
    let drug = Drug {
        name: "warfarin".to_string(),
        drugbank_id: None,
        chembl_id: None,
        unii: None,
        drug_type: None,
        mechanism: None,
        mechanisms: Vec::new(),
        approval_date: None,
        approval_date_raw: None,
        approval_date_display: None,
        approval_summary: None,
        brand_names: Vec::new(),
        route: None,
        targets: Vec::new(),
        variant_targets: Vec::new(),
        target_family: None,
        target_family_name: None,
        indications: Vec::new(),
        interactions: Vec::new(),
        interaction_text: None,
        interaction_pagination: None,
        interaction_bundle_freshness: None,
        pharm_classes: Vec::new(),
        top_adverse_events: Vec::new(),
        faers_query: None,
        label: None,
        label_set_id: None,
        shortage: None,
        approvals: None,
        us_safety_warnings: None,
        ema_regulatory: None,
        ema_safety: None,
        ema_shortage: None,
        who_prequalification: None,
        civic: None,
    };

    let related = related_drug(&drug);
    assert!(related.contains(&"biomcp search pgx -d warfarin".to_string()));
}
