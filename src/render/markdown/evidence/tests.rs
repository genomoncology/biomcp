use super::*;

#[test]
fn gene_evidence_urls_include_ensembl_and_omim() {
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

    let urls = gene_evidence_urls(&gene);
    assert!(urls.contains(&(
        "Ensembl",
        "https://www.ensembl.org/Homo_sapiens/Gene/Summary?g=ENSG00000157764".to_string()
    )));
    assert!(urls.contains(&("OMIM", "https://www.omim.org/entry/164757".to_string())));
}

#[test]
fn variant_evidence_urls_include_dbsnp_and_cosmic() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr7:g.140453136A>T",
        "gene": "BRAF",
        "rsid": "rs113488022",
        "cosmic_id": "COSM476",
        "clinvar_id": "13961"
    }))
    .expect("variant should deserialize");

    let urls = variant_evidence_urls(&variant);
    assert!(urls.contains(&(
        "dbSNP",
        "https://www.ncbi.nlm.nih.gov/snp/rs113488022".to_string()
    )));
    assert!(urls.contains(&(
        "COSMIC",
        "https://cancer.sanger.ac.uk/cosmic/mutation/overview?id=COSM476".to_string()
    )));
}

#[test]
fn variant_evidence_urls_include_gnomad_for_population_data() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr11:g.5227002A>T",
        "gene": "HBB",
        "rsid": "rs334",
        "gnomad_af": 0.042
    }))
    .expect("variant should deserialize");

    let urls = variant_evidence_urls(&variant);
    assert!(urls.contains(&(
        "gnomAD",
        "https://gnomad.broadinstitute.org/variant/rs334".to_string()
    )));
}

#[test]
fn variant_evidence_urls_fall_back_to_hgvs_slug_for_population_data() {
    let variant: Variant = serde_json::from_value(serde_json::json!({
        "id": "chr7:g.140453136A>T",
        "gene": "BRAF",
        "population_breakdown": {
            "populations": [{"population": "global", "af": 0.01}]
        }
    }))
    .expect("variant should deserialize");

    let urls = variant_evidence_urls(&variant);
    assert!(urls.contains(&(
        "gnomAD",
        "https://gnomad.broadinstitute.org/variant/7-140453136-A-T".to_string()
    )));
}

#[test]
fn disease_evidence_urls_include_record_links() {
    let disease = Disease {
        id: "MONDO:0009061".to_string(),
        name: "cystic fibrosis".to_string(),
        definition: None,
        synonyms: Vec::new(),
        parents: Vec::new(),
        associated_genes: Vec::new(),
        gene_associations: vec![crate::entities::disease::DiseaseGeneAssociation {
            gene: "CFTR".to_string(),
            relationship: None,
            source: Some("infores:orphanet".to_string()),
            opentargets_score: None,
        }],
        top_genes: Vec::new(),
        top_gene_scores: Vec::new(),
        treatment_landscape: Vec::new(),
        recruiting_trial_count: None,
        pathways: Vec::new(),
        phenotypes: vec![crate::entities::disease::DiseasePhenotype {
            hpo_id: "HP:0001945".to_string(),
            name: Some("Dehydration".to_string()),
            evidence: None,
            frequency: None,
            frequency_qualifier: None,
            onset_qualifier: None,
            sex_qualifier: None,
            stage_qualifier: None,
            qualifiers: Vec::new(),
            source: Some("infores:omim".to_string()),
        }],
        clinical_features: Vec::new(),
        key_features: Vec::new(),
        variants: Vec::new(),
        top_variant: None,
        models: vec![crate::entities::disease::DiseaseModelAssociation {
            model: "MGI:3698752".to_string(),
            model_id: None,
            organism: Some("Mus musculus".to_string()),
            relationship: Some("model of".to_string()),
            source: Some("infores:mgi".to_string()),
            evidence_count: Some(2),
        }],
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
        xrefs: std::collections::HashMap::from([
            ("Orphanet".to_string(), "586".to_string()),
            ("OMIM".to_string(), "219700".to_string()),
        ]),
    };

    let urls = disease_evidence_urls(&disease);
    assert!(urls.contains(&(
        "Orphanet".to_string(),
        "https://www.orpha.net/en/disease/detail/586".to_string()
    )));
    assert!(urls.contains(&(
        "OMIM".to_string(),
        "https://www.omim.org/entry/219700".to_string()
    )));
    assert!(urls.iter().any(|(label, url)| {
        *label == "MGI" && url.starts_with("https://www.informatics.jax.org/accession/MGI:")
    }));
}

#[test]
fn drug_evidence_urls_include_chembl() {
    let drug = Drug {
        name: "osimertinib".to_string(),
        drugbank_id: Some("DB09330".to_string()),
        chembl_id: Some("CHEMBL3353410".to_string()),
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

    let urls = drug_evidence_urls(&drug);
    assert!(urls.contains(&(
        "ChEMBL",
        "https://www.ebi.ac.uk/chembl/compound_report_card/CHEMBL3353410".to_string()
    )));
}

#[test]
fn drug_evidence_urls_include_faers_and_dailymed_when_sections_exist() {
    let drug = Drug {
            name: "ivacaftor".to_string(),
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
            top_adverse_events: vec!["Rash".to_string()],
            faers_query: Some(
                "(patient.drug.openfda.generic_name:\"ivacaftor\") AND patient.drug.drugcharacterization:1"
                    .to_string(),
            ),
            label: Some(crate::entities::drug::DrugLabel {
                indication_summary: Vec::new(),
                indications: None,
                warnings: Some("Warnings".to_string()),
                dosage: None,
            }),
            label_set_id: Some("set-123".to_string()),
            shortage: None,
            approvals: None,
            us_safety_warnings: None,
            ema_regulatory: None,
            ema_safety: None,
            ema_shortage: None,
            who_prequalification: None,
            civic: None,
        };

    let urls = drug_evidence_urls(&drug);
    assert!(urls.iter().any(|(label, url)| {
        *label == "OpenFDA FAERS"
            && url.starts_with("https://api.fda.gov/drug/event.json?search=")
            && url.contains("count=patient.reaction.reactionmeddrapt.exact")
    }));
    assert!(urls.iter().any(|(label, url)| {
        *label == "DailyMed" && url.contains("/drugInfo.cfm?setid=set-123")
    }));
}
