use serde::Serialize;

use crate::error::BioMcpError;
use crate::render::provenance::SectionSource;

pub fn to_pretty<T: Serialize>(value: &T) -> Result<String, BioMcpError> {
    Ok(serde_json::to_string_pretty(value)?)
}

#[derive(Debug, Clone, Serialize)]
pub struct EvidenceUrl {
    pub label: String,
    pub url: String,
}

#[derive(Serialize)]
struct EntityMeta {
    evidence_urls: Vec<EvidenceUrl>,
    next_commands: Vec<String>,
    section_sources: Vec<SectionSource>,
}

#[derive(Serialize)]
struct EntityJsonResponse<'a, T: Serialize> {
    #[serde(flatten)]
    entity: &'a T,
    _meta: EntityMeta,
}

pub fn to_entity_json<T: Serialize>(
    entity: &T,
    evidence_urls: Vec<(&str, String)>,
    next_commands: Vec<String>,
    section_sources: Vec<SectionSource>,
) -> Result<String, BioMcpError> {
    let evidence_urls = evidence_urls
        .into_iter()
        .filter_map(|(label, url)| {
            let label = label.trim();
            let url = url.trim();
            if label.is_empty() || url.is_empty() {
                return None;
            }
            Some(EvidenceUrl {
                label: label.to_string(),
                url: url.to_string(),
            })
        })
        .collect::<Vec<_>>();
    let next_commands = next_commands
        .into_iter()
        .map(|cmd| cmd.trim().to_string())
        .filter(|cmd| !cmd.is_empty())
        .collect::<Vec<_>>();
    let section_sources = section_sources
        .into_iter()
        .filter_map(SectionSource::normalized)
        .collect::<Vec<_>>();

    to_pretty(&EntityJsonResponse {
        entity,
        _meta: EntityMeta {
            evidence_urls,
            next_commands,
            section_sources,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{to_entity_json, to_pretty};
    use crate::entities::drug::Drug;
    use crate::entities::gene::Gene;
    use crate::render::provenance::SectionSource;
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
            expression: None,
            hpa: None,
            druggability: None,
            clingen: None,
            constraint: None,
            disgenet: None,
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
            interaction_text: None,
            pharm_classes: Vec::new(),
            top_adverse_events: Vec::new(),
            faers_query: None,
            label: None,
            label_set_id: None,
            shortage: None,
            approvals: None,
            civic: None,
        };

        let json = to_pretty(&drug).expect("drug json");
        assert!(json.contains("\"name\": \"osimertinib\""));
        assert!(json.contains("\"targets\""));
    }

    #[test]
    fn to_entity_json_adds_meta_and_flattens_entity() {
        #[derive(Serialize)]
        struct DemoEntity<'a> {
            id: &'a str,
            label: &'a str,
        }

        let json = to_entity_json(
            &DemoEntity {
                id: "demo-1",
                label: "Demo",
            },
            vec![
                ("Source A", "https://example.org/source-a".to_string()),
                ("Source B", "https://example.org/source-b".to_string()),
            ],
            vec!["biomcp get gene BRAF".to_string()],
            vec![SectionSource {
                key: "summary".to_string(),
                label: "Summary".to_string(),
                sources: vec!["NCBI Gene".to_string()],
            }],
        )
        .expect("entity json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["id"], "demo-1");
        assert_eq!(value["label"], "Demo");
        assert_eq!(value["_meta"]["evidence_urls"][0]["label"], "Source A");
        assert_eq!(
            value["_meta"]["evidence_urls"][0]["url"],
            "https://example.org/source-a"
        );
        assert_eq!(value["_meta"]["next_commands"][0], "biomcp get gene BRAF");
        assert_eq!(value["_meta"]["section_sources"][0]["key"], "summary");
        assert_eq!(value["_meta"]["section_sources"][0]["label"], "Summary");
        assert_eq!(
            value["_meta"]["section_sources"][0]["sources"][0],
            "NCBI Gene"
        );
    }

    #[test]
    fn to_entity_json_filters_blank_evidence_rows() {
        #[derive(Serialize)]
        struct DemoEntity<'a> {
            id: &'a str,
        }

        let json = to_entity_json(
            &DemoEntity { id: "demo-2" },
            vec![
                ("", "https://example.org/empty-label".to_string()),
                ("Missing URL", "".to_string()),
                ("Valid", "https://example.org/valid".to_string()),
            ],
            Vec::new(),
            Vec::new(),
        )
        .expect("entity json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let urls = value["_meta"]["evidence_urls"]
            .as_array()
            .expect("evidence url array");
        assert_eq!(urls.len(), 1);
        assert_eq!(urls[0]["label"], "Valid");
        assert_eq!(urls[0]["url"], "https://example.org/valid");
    }

    #[test]
    fn to_entity_json_filters_blank_section_source_rows() {
        #[derive(Serialize)]
        struct DemoEntity<'a> {
            id: &'a str,
        }

        let json = to_entity_json(
            &DemoEntity { id: "demo-3" },
            Vec::new(),
            Vec::new(),
            vec![
                SectionSource {
                    key: " ".to_string(),
                    label: "Summary".to_string(),
                    sources: vec!["NCBI Gene".to_string()],
                },
                SectionSource {
                    key: "summary".to_string(),
                    label: " ".to_string(),
                    sources: vec!["NCBI Gene".to_string()],
                },
                SectionSource {
                    key: "summary".to_string(),
                    label: "Summary".to_string(),
                    sources: vec![" ".to_string()],
                },
                SectionSource {
                    key: "identity".to_string(),
                    label: "Identity".to_string(),
                    sources: vec![" NCBI Gene / MyGene.info ".to_string(), "".to_string()],
                },
            ],
        )
        .expect("entity json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let section_sources = value["_meta"]["section_sources"]
            .as_array()
            .expect("section sources array");
        assert_eq!(section_sources.len(), 1);
        assert_eq!(section_sources[0]["key"], "identity");
        assert_eq!(section_sources[0]["label"], "Identity");
        assert_eq!(section_sources[0]["sources"][0], "NCBI Gene / MyGene.info");
    }
}
