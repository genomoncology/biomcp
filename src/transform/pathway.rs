use crate::entities::pathway::{Pathway, PathwaySearchResult};
use crate::sources::kegg::{KeggPathwayHit, KeggPathwayRecord};
use crate::sources::reactome::{ReactomePathwayHit, ReactomePathwayRecord};

pub fn from_reactome_hit(hit: ReactomePathwayHit) -> PathwaySearchResult {
    PathwaySearchResult {
        source: "Reactome".to_string(),
        id: hit.id,
        name: hit.name,
    }
}

pub fn from_reactome_record(record: ReactomePathwayRecord) -> Pathway {
    Pathway {
        source: "Reactome".to_string(),
        id: record.id,
        name: record.name,
        species: record.species,
        summary: record.summary,
        genes: Vec::new(),
        events: Vec::new(),
        enrichment: Vec::new(),
    }
}

pub fn from_kegg_hit(hit: KeggPathwayHit) -> PathwaySearchResult {
    PathwaySearchResult {
        source: "KEGG".to_string(),
        id: hit.id,
        name: hit.name,
    }
}

pub fn from_kegg_record(record: KeggPathwayRecord) -> Pathway {
    Pathway {
        source: "KEGG".to_string(),
        id: record.id,
        name: record.name,
        species: Some("Homo sapiens".to_string()),
        summary: record.summary,
        genes: record.genes,
        events: Vec::new(),
        enrichment: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_reactome_hit_maps_fields() {
        let hit = ReactomePathwayHit {
            id: "R-HSA-5673001".to_string(),
            name: "RAF/MAP kinase cascade".to_string(),
        };

        let out = from_reactome_hit(hit);
        assert_eq!(out.source, "Reactome");
        assert_eq!(out.id, "R-HSA-5673001");
        assert_eq!(out.name, "RAF/MAP kinase cascade");
    }

    #[test]
    fn from_reactome_record_maps_fields() {
        let record = ReactomePathwayRecord {
            id: "R-HSA-5673001".to_string(),
            name: "RAF/MAP kinase cascade".to_string(),
            species: Some("Homo sapiens".to_string()),
            summary: Some("Signal transduction pathway".to_string()),
        };

        let out = from_reactome_record(record);
        assert_eq!(out.source, "Reactome");
        assert_eq!(out.id, "R-HSA-5673001");
        assert_eq!(out.name, "RAF/MAP kinase cascade");
        assert_eq!(out.species.as_deref(), Some("Homo sapiens"));
        assert_eq!(out.summary.as_deref(), Some("Signal transduction pathway"));
        assert!(out.genes.is_empty());
        assert!(out.events.is_empty());
        assert!(out.enrichment.is_empty());
    }

    #[test]
    fn from_reactome_record_handles_missing_summary() {
        let record = ReactomePathwayRecord {
            id: "R-HSA-6802957".to_string(),
            name: "Signaling by BRAF and RAF fusions".to_string(),
            species: Some("Homo sapiens".to_string()),
            summary: None,
        };

        let out = from_reactome_record(record);
        assert_eq!(out.summary, None);
        assert!(out.events.is_empty());
    }

    #[test]
    fn pathway_sections_maps_cell_cycle() {
        let record = ReactomePathwayRecord {
            id: "R-HSA-69278".to_string(),
            name: "Cell Cycle, Mitotic".to_string(),
            species: Some("Homo sapiens".to_string()),
            summary: Some("Mitotic checkpoints and progression.".to_string()),
        };

        let out = from_reactome_record(record);
        assert_eq!(out.id, "R-HSA-69278");
        assert_eq!(out.name, "Cell Cycle, Mitotic");
        assert_eq!(out.species.as_deref(), Some("Homo sapiens"));
    }

    #[test]
    fn from_kegg_hit_maps_fields() {
        let hit = KeggPathwayHit {
            id: "hsa04010".to_string(),
            name: "MAPK signaling pathway".to_string(),
        };

        let out = from_kegg_hit(hit);
        assert_eq!(out.source, "KEGG");
        assert_eq!(out.id, "hsa04010");
        assert_eq!(out.name, "MAPK signaling pathway");
    }

    #[test]
    fn from_kegg_record_maps_fields() {
        let record = KeggPathwayRecord {
            id: "hsa05200".to_string(),
            name: "Pathways in cancer".to_string(),
            summary: Some("Cancer overview.".to_string()),
            genes: vec!["BRAF".to_string(), "EGFR".to_string()],
        };

        let out = from_kegg_record(record);
        assert_eq!(out.source, "KEGG");
        assert_eq!(out.id, "hsa05200");
        assert_eq!(out.species.as_deref(), Some("Homo sapiens"));
        assert_eq!(out.genes, vec!["BRAF".to_string(), "EGFR".to_string()]);
        assert!(out.events.is_empty());
    }
}
