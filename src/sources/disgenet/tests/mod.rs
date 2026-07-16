use std::collections::HashMap;

use super::*;

mod construction;
mod parsing;

fn test_gene(entrez_id: &str) -> Gene {
    Gene {
        section_outcomes: Default::default(),
        symbol: "TP53".to_string(),
        name: "tumor protein p53".to_string(),
        entrez_id: entrez_id.to_string(),
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
    }
}

fn test_disease(name: &str, umls_cui: Option<&str>) -> Disease {
    let mut xrefs = HashMap::new();
    if let Some(cui) = umls_cui {
        xrefs.insert("umls_cui".to_string(), cui.to_string());
    }
    Disease {
        id: "MONDO:0007254".to_string(),
        name: name.to_string(),
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
        xrefs,
    }
}

fn summary_response_bytes() -> &'static [u8] {
    br#"{
        "status": "OK",
        "httpStatus": 200,
        "paging": {
            "pageSize": 100,
            "totalElements": 2,
            "totalElementsInPage": 2,
            "currentPageNumber": 0
        },
        "warnings": [],
        "payload": [
            {
                "symbolOfGene": "TP53",
                "geneNcbiID": 7157,
                "diseaseName": "Breast Carcinoma",
                "diseaseUMLSCUI": "C0678222",
                "score": 0.91,
                "numPMIDs": 1234,
                "numCTsupportingAssociation": 4,
                "ei": 0.72,
                "el": "Definitive"
            },
            {
                "symbolOfGene": "TP53",
                "geneNcbiID": 7157,
                "diseaseName": "Li-Fraumeni Syndrome",
                "diseaseUMLSCUI": "C0085390",
                "score": 0.88,
                "numPMIDs": 400,
                "numCTsupportingAssociation": 1,
                "ei": 0.66,
                "el": "Strong"
            }
        ]
    }"#
}

fn empty_summary_response_bytes() -> &'static [u8] {
    br#"{
        "status": "OK",
        "httpStatus": 200,
        "payload": []
    }"#
}

fn disease_response_bytes() -> &'static [u8] {
    br#"{
        "status": "OK",
        "httpStatus": 200,
        "payload": [
            {
                "name": "Breast carcinoma",
                "diseaseUMLSCUI": "C0678222",
                "search_rank": 0.82,
                "synonyms": [
                    {"name": "Breast cancer"}
                ]
            }
        ]
    }"#
}

fn empty_disease_response_bytes() -> &'static [u8] {
    br#"{
        "status": "OK",
        "httpStatus": 200,
        "payload": []
    }"#
}
