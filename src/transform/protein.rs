use crate::entities::protein::{Protein, ProteinSearchResult};
use crate::sources::uniprot::UniProtRecord;

pub fn from_uniprot_search_record(record: UniProtRecord) -> ProteinSearchResult {
    let accession = record.primary_accession.clone();
    let entry_name = record
        .uni_prot_kb_id
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| accession.clone());
    ProteinSearchResult {
        accession,
        uniprot_id: entry_name,
        name: record.display_name(),
        gene_symbol: record.primary_gene_symbol(),
        species: record
            .organism
            .as_ref()
            .and_then(|o| o.scientific_name.as_deref())
            .map(str::trim)
            .map(str::to_string)
            .filter(|v| !v.is_empty()),
    }
}

pub fn from_uniprot_record_base(record: UniProtRecord) -> Protein {
    let accession = record.primary_accession.clone();
    let entry_id = record.uni_prot_kb_id.clone();
    let name = record.display_name();
    let gene_symbol = record.primary_gene_symbol();
    let organism = record
        .organism
        .as_ref()
        .and_then(|o| o.scientific_name.as_deref())
        .map(str::trim)
        .map(str::to_string)
        .filter(|v| !v.is_empty());
    let length = record.sequence.as_ref().and_then(|s| s.length);
    let function = record.function_summary();

    Protein {
        accession,
        entry_id,
        name,
        gene_symbol,
        organism,
        length,
        function,
        structures: Vec::new(),
        structure_count: None,
        domains: Vec::new(),
        interactions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::uniprot::{
        UniProtComment, UniProtCrossReference, UniProtGene, UniProtNameContainer, UniProtOrganism,
        UniProtProteinDescription, UniProtSequence, UniProtTextValue,
    };

    fn sample_record() -> UniProtRecord {
        UniProtRecord {
            primary_accession: "P15056".to_string(),
            uni_prot_kb_id: Some("BRAF_HUMAN".to_string()),
            protein_description: Some(UniProtProteinDescription {
                recommended_name: Some(UniProtNameContainer {
                    full_name: Some(UniProtTextValue {
                        value: "Serine/threonine-protein kinase B-raf".to_string(),
                    }),
                }),
                submission_names: None,
            }),
            genes: vec![UniProtGene {
                gene_name: Some(UniProtTextValue {
                    value: "BRAF".to_string(),
                }),
            }],
            organism: Some(UniProtOrganism {
                scientific_name: Some("Homo sapiens".to_string()),
            }),
            sequence: Some(UniProtSequence { length: Some(766) }),
            comments: vec![UniProtComment {
                comment_type: Some("FUNCTION".to_string()),
                texts: vec![UniProtTextValue {
                    value: "Protein kinase involved in MAPK signaling.".to_string(),
                }],
            }],
            uni_prot_kb_cross_references: vec![UniProtCrossReference {
                database: Some("PDB".to_string()),
                id: Some("6PP9".to_string()),
                properties: Vec::new(),
            }],
        }
    }

    #[test]
    fn from_uniprot_search_record_maps_fields() {
        let out = from_uniprot_search_record(sample_record());
        assert_eq!(out.accession, "P15056");
        assert_eq!(out.uniprot_id, "BRAF_HUMAN");
        assert_eq!(out.name, "Serine/threonine-protein kinase B-raf");
        assert_eq!(out.gene_symbol.as_deref(), Some("BRAF"));
        assert_eq!(out.species.as_deref(), Some("Homo sapiens"));
    }

    #[test]
    fn from_uniprot_record_base_maps_fields() {
        let out = from_uniprot_record_base(sample_record());
        assert_eq!(out.accession, "P15056");
        assert_eq!(out.entry_id.as_deref(), Some("BRAF_HUMAN"));
        assert_eq!(out.name, "Serine/threonine-protein kinase B-raf");
        assert_eq!(out.gene_symbol.as_deref(), Some("BRAF"));
        assert_eq!(out.organism.as_deref(), Some("Homo sapiens"));
        assert_eq!(out.length, Some(766));
        assert!(
            out.function
                .as_deref()
                .is_some_and(|v| v.contains("MAPK signaling"))
        );
        assert!(out.domains.is_empty());
        assert!(out.interactions.is_empty());
        assert!(out.structures.is_empty());
    }

    #[test]
    fn from_uniprot_search_record_handles_missing_organism() {
        let mut record = sample_record();
        record.organism = None;
        let out = from_uniprot_search_record(record);
        assert_eq!(out.species, None);
    }

    #[test]
    fn from_uniprot_record_base_handles_missing_sequence() {
        let mut record = sample_record();
        record.sequence = None;
        let out = from_uniprot_record_base(record);
        assert_eq!(out.length, None);
    }

    #[test]
    fn protein_sections_maps_egfr() {
        let mut record = sample_record();
        record.primary_accession = "P00533".to_string();
        record.uni_prot_kb_id = Some("EGFR_HUMAN".to_string());
        record.protein_description = Some(UniProtProteinDescription {
            recommended_name: Some(UniProtNameContainer {
                full_name: Some(UniProtTextValue {
                    value: "Epidermal growth factor receptor".to_string(),
                }),
            }),
            submission_names: None,
        });
        record.genes = vec![UniProtGene {
            gene_name: Some(UniProtTextValue {
                value: "EGFR".to_string(),
            }),
        }];

        let out = from_uniprot_record_base(record);
        assert_eq!(out.accession, "P00533");
        assert_eq!(out.gene_symbol.as_deref(), Some("EGFR"));
        assert!(out.name.contains("growth factor receptor"));
    }

    #[test]
    fn protein_sections_maps_tp53() {
        let mut record = sample_record();
        record.primary_accession = "P04637".to_string();
        record.uni_prot_kb_id = Some("P53_HUMAN".to_string());
        record.protein_description = Some(UniProtProteinDescription {
            recommended_name: Some(UniProtNameContainer {
                full_name: Some(UniProtTextValue {
                    value: "Cellular tumor antigen p53".to_string(),
                }),
            }),
            submission_names: None,
        });
        record.genes = vec![UniProtGene {
            gene_name: Some(UniProtTextValue {
                value: "TP53".to_string(),
            }),
        }];

        let out = from_uniprot_search_record(record);
        assert_eq!(out.accession, "P04637");
        assert_eq!(out.gene_symbol.as_deref(), Some("TP53"));
        assert!(out.name.contains("p53"));
    }
}
