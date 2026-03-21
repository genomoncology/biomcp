use serde::Serialize;

use crate::entities::discover::{AliasFallbackDecision, DiscoverResult};
use crate::entities::variant::{VariantGuidance, VariantGuidanceKind};
use crate::error::BioMcpError;
use crate::render::markdown::discover_evidence_urls;
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

#[derive(Serialize)]
struct DiscoverMeta {
    evidence_urls: Vec<EvidenceUrl>,
    next_commands: Vec<String>,
    section_sources: Vec<SectionSource>,
    discovery_sources: Vec<String>,
}

#[derive(Serialize)]
struct DiscoverJsonResponse<'a> {
    #[serde(flatten)]
    result: &'a DiscoverResult,
    _meta: DiscoverMeta,
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

pub fn to_discover_json(result: &DiscoverResult) -> Result<String, BioMcpError> {
    let evidence_urls = discover_evidence_urls(result)
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
    let next_commands = result
        .next_commands
        .iter()
        .map(|cmd| cmd.trim().to_string())
        .filter(|cmd| !cmd.is_empty())
        .collect::<Vec<_>>();
    let section_sources = crate::render::provenance::discover_section_sources(result)
        .into_iter()
        .filter_map(SectionSource::normalized)
        .collect::<Vec<_>>();
    let mut discovery_sources = Vec::new();
    let mut seen_sources = std::collections::HashSet::new();
    for section in &section_sources {
        for source in &section.sources {
            if seen_sources.insert(source.to_ascii_lowercase()) {
                discovery_sources.push(source.clone());
            }
        }
    }

    to_pretty(&DiscoverJsonResponse {
        result,
        _meta: DiscoverMeta {
            evidence_urls,
            next_commands,
            section_sources,
            discovery_sources,
        },
    })
}

#[derive(Serialize)]
struct AliasError {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct AliasJsonResponse<'a> {
    error: AliasError,
    _meta: AliasMeta<'a>,
}

#[derive(Serialize)]
struct AliasMeta<'a> {
    not_found: bool,
    alias_resolution: AliasResolution<'a>,
    next_commands: &'a [String],
}

#[derive(Serialize)]
struct AliasCandidateJson<'a> {
    label: &'a str,
    primary_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    primary_id: Option<&'a str>,
    confidence: &'static str,
    match_tier: &'static str,
}

#[derive(Serialize)]
#[serde(tag = "kind")]
enum AliasResolution<'a> {
    #[serde(rename = "canonical")]
    Canonical {
        requested_entity: &'static str,
        query: &'a str,
        canonical: &'a str,
        canonical_id: &'a str,
        confidence: &'static str,
        match_tier: &'static str,
        sources: &'a [String],
    },
    #[serde(rename = "ambiguous")]
    Ambiguous {
        requested_entity: &'static str,
        query: &'a str,
        candidates: Vec<AliasCandidateJson<'a>>,
    },
}

#[derive(Serialize)]
struct VariantGuidanceJsonResponse<'a> {
    error: AliasError,
    _meta: VariantGuidanceMeta<'a>,
}

#[derive(Serialize)]
struct VariantGuidanceMeta<'a> {
    not_found: bool,
    alias_resolution: VariantAliasResolution<'a>,
    next_commands: &'a [String],
}

#[derive(Serialize)]
#[serde(tag = "kind")]
enum VariantAliasResolution<'a> {
    #[serde(rename = "gene_residue_alias")]
    GeneResidueAlias {
        requested_entity: &'static str,
        query: &'a str,
        gene: &'a str,
        alias: &'a str,
    },
    #[serde(rename = "protein_change_only")]
    ProteinChangeOnly {
        requested_entity: &'static str,
        query: &'a str,
        change: &'a str,
    },
}

pub(crate) fn to_alias_suggestion_json(
    decision: &AliasFallbackDecision,
) -> Result<String, BioMcpError> {
    match decision {
        AliasFallbackDecision::Canonical(alias) => to_pretty(&AliasJsonResponse {
            error: AliasError {
                code: "not_found",
                message: format!(
                    "No exact {} match for '{}'.",
                    alias.requested_entity.cli_name(),
                    alias.query
                ),
            },
            _meta: AliasMeta {
                not_found: true,
                alias_resolution: AliasResolution::Canonical {
                    requested_entity: alias.requested_entity.cli_name(),
                    query: &alias.query,
                    canonical: &alias.canonical,
                    canonical_id: &alias.canonical_id,
                    confidence: discover_confidence_name(alias.confidence),
                    match_tier: match_tier_name(alias.match_tier),
                    sources: &alias.sources,
                },
                next_commands: &alias.next_commands,
            },
        }),
        AliasFallbackDecision::Ambiguous(alias) => to_pretty(&AliasJsonResponse {
            error: AliasError {
                code: "not_found",
                message: format!(
                    "No exact {} match for '{}'.",
                    alias.requested_entity.cli_name(),
                    alias.query
                ),
            },
            _meta: AliasMeta {
                not_found: true,
                alias_resolution: AliasResolution::Ambiguous {
                    requested_entity: alias.requested_entity.cli_name(),
                    query: &alias.query,
                    candidates: alias
                        .candidates
                        .iter()
                        .map(|candidate| AliasCandidateJson {
                            label: &candidate.label,
                            primary_type: candidate.primary_type.cli_name(),
                            primary_id: candidate.primary_id.as_deref(),
                            confidence: discover_confidence_name(candidate.confidence),
                            match_tier: match_tier_name(candidate.match_tier),
                        })
                        .collect(),
                },
                next_commands: &alias.next_commands,
            },
        }),
        AliasFallbackDecision::None => Err(BioMcpError::InvalidArgument(
            "Alias suggestion JSON requires a canonical or ambiguous alias decision".into(),
        )),
    }
}

fn variant_guidance_message(guidance: &VariantGuidance) -> String {
    match &guidance.kind {
        VariantGuidanceKind::GeneResidueAlias { .. } => {
            format!(
                "BioMCP could not map '{}' to an exact variant.",
                guidance.query
            )
        }
        VariantGuidanceKind::ProteinChangeOnly { .. } => format!(
            "BioMCP could not map '{}' to an exact variant without gene context.",
            guidance.query
        ),
    }
}

pub(crate) fn to_variant_guidance_json(guidance: &VariantGuidance) -> Result<String, BioMcpError> {
    let alias_resolution = match &guidance.kind {
        VariantGuidanceKind::GeneResidueAlias { gene, alias } => {
            VariantAliasResolution::GeneResidueAlias {
                requested_entity: "variant",
                query: &guidance.query,
                gene,
                alias,
            }
        }
        VariantGuidanceKind::ProteinChangeOnly { change } => {
            VariantAliasResolution::ProteinChangeOnly {
                requested_entity: "variant",
                query: &guidance.query,
                change,
            }
        }
    };

    to_pretty(&VariantGuidanceJsonResponse {
        error: AliasError {
            code: "not_found",
            message: variant_guidance_message(guidance),
        },
        _meta: VariantGuidanceMeta {
            not_found: true,
            alias_resolution,
            next_commands: &guidance.next_commands,
        },
    })
}

fn discover_confidence_name(
    confidence: crate::entities::discover::DiscoverConfidence,
) -> &'static str {
    match confidence {
        crate::entities::discover::DiscoverConfidence::CanonicalId => "CanonicalId",
        crate::entities::discover::DiscoverConfidence::UmlsOnly => "UmlsOnly",
        crate::entities::discover::DiscoverConfidence::LabelOnly => "LabelOnly",
    }
}

fn match_tier_name(match_tier: crate::entities::discover::MatchTier) -> &'static str {
    match match_tier {
        crate::entities::discover::MatchTier::Exact => "Exact",
        crate::entities::discover::MatchTier::Prefix => "Prefix",
        crate::entities::discover::MatchTier::Contains => "Contains",
        crate::entities::discover::MatchTier::Weak => "Weak",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        to_alias_suggestion_json, to_discover_json, to_entity_json, to_pretty,
        to_variant_guidance_json,
    };
    use crate::entities::discover::{
        AliasCanonicalMatch, AliasFallbackDecision, ConceptSource, ConceptXref, DiscoverConcept,
        DiscoverConfidence, DiscoverIntent, DiscoverResult, DiscoverType, MatchTier,
        PlainLanguageTopic,
    };
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
    fn json_render_gene_entity_with_sparse_disgenet_omits_optional_fields() {
        let gene = Gene {
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
        };

        let json = to_pretty(&gene).expect("gene json");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        let association = &value["disgenet"]["associations"][0];

        assert_eq!(association["disease_name"], "Sparse Disease");
        assert_eq!(association["disease_cui"], "C1234567");
        assert_eq!(association["score"], 0.23);
        assert!(association.get("publication_count").is_none());
        assert!(association.get("clinical_trial_count").is_none());
        assert!(association.get("evidence_index").is_none());
        assert!(association.get("evidence_level").is_none());
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
            approval_date_raw: Some("2015-11-13".to_string()),
            approval_date_display: Some("November 13, 2015".to_string()),
            approval_summary: Some("FDA approved on November 13, 2015".to_string()),
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

    #[test]
    fn to_discover_json_adds_discover_meta_aliases() {
        let json = to_discover_json(&DiscoverResult {
            query: "Keytruda".to_string(),
            normalized_query: "keytruda".to_string(),
            concepts: vec![DiscoverConcept {
                label: "pembrolizumab".to_string(),
                primary_id: Some("RXNORM:1547545".to_string()),
                primary_type: DiscoverType::Drug,
                synonyms: vec!["Keytruda".to_string()],
                xrefs: vec![ConceptXref {
                    source: "RXNORM".to_string(),
                    id: "1547545".to_string(),
                }],
                sources: vec![ConceptSource {
                    source: "OLS4".to_string(),
                    id: "DRON:00018671".to_string(),
                    label: "pembrolizumab".to_string(),
                    source_type: "DRON".to_string(),
                }],
                match_tier: MatchTier::Exact,
                confidence: DiscoverConfidence::CanonicalId,
            }],
            plain_language: Some(PlainLanguageTopic {
                title: "Chest Pain".to_string(),
                url: "https://medlineplus.gov/chestpain.html".to_string(),
                summary_excerpt: "Summary".to_string(),
            }),
            next_commands: vec!["biomcp get drug pembrolizumab".to_string()],
            notes: vec!["UMLS enrichment unavailable (set UMLS_API_KEY)".to_string()],
            ambiguous: false,
            intent: DiscoverIntent::General,
        })
        .expect("discover json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(
            value["_meta"]["next_commands"][0],
            "biomcp get drug pembrolizumab"
        );
        assert_eq!(
            value["_meta"]["section_sources"][0]["key"],
            "structured_concepts"
        );
        assert_eq!(value["_meta"]["discovery_sources"][0], "OLS4");
        assert_eq!(value["_meta"]["evidence_urls"][0]["label"], "OLS4");
    }

    #[test]
    fn to_alias_suggestion_json_includes_alias_resolution_and_next_commands() {
        let json =
            to_alias_suggestion_json(&AliasFallbackDecision::Canonical(AliasCanonicalMatch {
                requested_entity: DiscoverType::Gene,
                query: "ERBB1".to_string(),
                canonical: "EGFR".to_string(),
                canonical_id: "HGNC:3236".to_string(),
                confidence: DiscoverConfidence::CanonicalId,
                match_tier: MatchTier::Exact,
                sources: vec!["OLS4/HGNC".to_string()],
                next_commands: vec!["biomcp get gene EGFR".to_string()],
            }))
            .expect("alias json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["error"]["code"], "not_found");
        assert_eq!(value["_meta"]["not_found"], true);
        assert_eq!(value["_meta"]["alias_resolution"]["kind"], "canonical");
        assert_eq!(value["_meta"]["alias_resolution"]["canonical"], "EGFR");
        assert_eq!(value["_meta"]["next_commands"][0], "biomcp get gene EGFR");
    }

    #[test]
    fn to_alias_suggestion_json_includes_ambiguous_resolution() {
        use crate::entities::discover::{AliasAmbiguity, AliasCandidateSummary};
        let json = to_alias_suggestion_json(&AliasFallbackDecision::Ambiguous(AliasAmbiguity {
            requested_entity: DiscoverType::Gene,
            query: "V600E".to_string(),
            candidates: vec![AliasCandidateSummary {
                label: "V600E".to_string(),
                primary_type: DiscoverType::Variant,
                primary_id: Some("SO:0001583".to_string()),
                confidence: DiscoverConfidence::CanonicalId,
                match_tier: MatchTier::Exact,
            }],
            next_commands: vec![
                "biomcp discover V600E".to_string(),
                "biomcp search gene -q V600E".to_string(),
            ],
        }))
        .expect("ambiguous alias json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["error"]["code"], "not_found");
        assert_eq!(value["_meta"]["not_found"], true);
        assert_eq!(value["_meta"]["alias_resolution"]["kind"], "ambiguous");
        assert_eq!(value["_meta"]["alias_resolution"]["query"], "V600E");
        assert_eq!(
            value["_meta"]["alias_resolution"]["candidates"][0]["label"],
            "V600E"
        );
        assert_eq!(
            value["_meta"]["alias_resolution"]["candidates"][0]["primary_type"],
            "variant"
        );
        assert_eq!(value["_meta"]["next_commands"][0], "biomcp discover V600E");
        assert_eq!(
            value["_meta"]["next_commands"][1],
            "biomcp search gene -q V600E"
        );
    }

    #[test]
    fn to_variant_guidance_json_includes_alias_resolution_and_next_commands() {
        let json = to_variant_guidance_json(&crate::entities::variant::VariantGuidance {
            query: "PTPN22 620W".to_string(),
            kind: crate::entities::variant::VariantGuidanceKind::GeneResidueAlias {
                gene: "PTPN22".to_string(),
                alias: "620W".to_string(),
            },
            next_commands: vec![
                "biomcp search variant \"PTPN22 620W\" --limit 10".to_string(),
                "biomcp search variant -g PTPN22 --limit 10".to_string(),
            ],
        })
        .expect("variant guidance json");

        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(value["error"]["code"], "not_found");
        assert_eq!(value["_meta"]["not_found"], true);
        assert_eq!(
            value["_meta"]["alias_resolution"]["kind"],
            "gene_residue_alias"
        );
        assert_eq!(value["_meta"]["alias_resolution"]["gene"], "PTPN22");
        assert_eq!(value["_meta"]["alias_resolution"]["alias"], "620W");
        assert_eq!(
            value["_meta"]["next_commands"][0],
            "biomcp search variant \"PTPN22 620W\" --limit 10"
        );
    }
}
