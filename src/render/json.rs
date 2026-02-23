use serde::Serialize;

use crate::error::BioMcpError;

pub fn to_pretty<T: Serialize>(value: &T) -> Result<String, BioMcpError> {
    Ok(serde_json::to_string_pretty(value)?)
}

#[cfg(test)]
mod tests {
    use super::to_pretty;
    use crate::entities::drug::Drug;
    use crate::entities::gene::Gene;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Demo<'a> {
        symbol: &'a str,
        score: f64,
    }

    #[test]
    fn to_pretty_serializes_with_indentation() {
        let payload = Demo {
            symbol: "BRAF",
            score: 0.98,
        };
        let json = to_pretty(&payload).expect("json");
        assert!(json.contains('\n'));
        assert!(json.contains("\"symbol\": \"BRAF\""));
        assert!(json.contains("\"score\": 0.98"));
    }

    #[test]
    fn json_render_gene_entity() {
        let gene = Gene {
            symbol: "EGFR".to_string(),
            name: "epidermal growth factor receptor".to_string(),
            entrez_id: "1956".to_string(),
            ensembl_id: Some("ENSG00000146648".to_string()),
            location: Some("7".to_string()),
            genomic_coordinates: None,
            omim_id: None,
            uniprot_id: Some("P00533".to_string()),
            summary: Some("Kinase receptor".to_string()),
            gene_type: Some("protein-coding".to_string()),
            aliases: vec!["ERBB".to_string()],
            clinical_diseases: Vec::new(),
            clinical_drugs: Vec::new(),
            pathways: None,
            ontology: None,
            diseases: None,
            protein: None,
            go: None,
            interactions: None,
            civic: None,
        };

        let json = to_pretty(&gene).expect("gene json");
        assert!(json.contains("\"symbol\": \"EGFR\""));
        assert!(json.contains("\"entrez_id\": \"1956\""));
    }

    #[test]
    fn json_render_drug_entity() {
        let drug = Drug {
            name: "osimertinib".to_string(),
            drugbank_id: Some("DB09330".to_string()),
            chembl_id: Some("CHEMBL3353410".to_string()),
            unii: None,
            drug_type: Some("small-molecule".to_string()),
            mechanism: Some("Inhibitor of EGFR".to_string()),
            mechanisms: vec!["Inhibitor of EGFR".to_string()],
            approval_date: Some("2015-11-13".to_string()),
            brand_names: vec!["Tagrisso".to_string()],
            route: None,
            targets: vec!["EGFR".to_string()],
            indications: vec!["Non-small cell lung cancer".to_string()],
            interactions: Vec::new(),
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            label: None,
            shortage: None,
            approvals: None,
            civic: None,
        };

        let json = to_pretty(&drug).expect("drug json");
        assert!(json.contains("\"name\": \"osimertinib\""));
        assert!(json.contains("\"targets\""));
    }
}
