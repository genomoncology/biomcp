use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::entities::variant::RequestedVariantIdentity;
use crate::sources::pubtator::{PubTatorExportResponse, PubTatorRelation};

pub(crate) const VERIFIER_VERSION: &str = "article-identity-v2";
pub(crate) const PUBTATOR_EXPORT_TEMPLATE_VERSION: &str = "pubtator-export-biocjson-v1";
const RESPONSE_SUBSET_VERSION: &str = "clinically-relevant-response-v1";
const CONTENT_SUBSET_VERSION: &str = "clinically-relevant-content-v1";

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct VariantArticleVerificationOptions {
    pub(crate) verify_identity: bool,
    pub(crate) confirmed_only: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantArticleIdentity {
    pub status: &'static str,
    pub basis: &'static str,
    pub requested_gene: Option<String>,
    pub requested_allele: Option<String>,
    pub observations: Vec<VariantArticleIdentityObservation>,
    pub contradictions: Vec<VariantArticleIdentityObservation>,
    pub incomplete: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantArticleIdentityObservation {
    pub source: &'static str,
    pub section: String,
    pub locator: String,
    pub linked_gene: String,
    pub observed_alias: String,
    pub gene_annotation_id: String,
    pub allele_annotation_id: String,
    pub provider_relation: String,
    pub canonical_content_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantArticleVerificationPlan {
    pub verifier_version: &'static str,
    pub provider_template_version: String,
    pub response_subset_version: &'static str,
    pub content_subset_version: &'static str,
    pub canonical_response_subset_hash: String,
    pub canonical_content_subset_hash: String,
    pub artifact_id: String,
    pub response_hashes_are_post_response: bool,
    pub captured_content_hashes_are_post_response: bool,
}

fn normalized(value: &str) -> String {
    crate::entities::variant::normalize_protein_change(value)
        .unwrap_or_else(|| value.trim().to_ascii_lowercase())
}

fn requested_allele(requested: &RequestedVariantIdentity) -> Option<String> {
    requested
        .protein_change
        .as_deref()
        .or(requested.coding_change.as_deref())
        .map(str::to_string)
}

fn contains_token(text: &str, token: &str) -> bool {
    text.match_indices(token).any(|(start, _)| {
        let before = text[..start].chars().next_back();
        let after = text[start + token.len()..].chars().next();
        before.is_none_or(|character| !character.is_ascii_alphanumeric())
            && after.is_none_or(|character| !character.is_ascii_alphanumeric())
    })
}

fn hash(value: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(
            value
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .as_bytes()
        )
    )
}

fn canonical_subset_hash(mut facts: Vec<String>) -> String {
    facts.sort_unstable();
    facts.dedup();
    hash(&facts.join("\n"))
}

fn canonical_relation(relation: &PubTatorRelation) -> String {
    let mut nodes = relation
        .nodes
        .iter()
        .map(|node| {
            serde_json::json!({
                "refid": node.refid,
                "role": node.role,
            })
            .to_string()
        })
        .collect::<Vec<_>>();
    nodes.sort_unstable();
    serde_json::json!({
        "id": relation.id,
        "infons": relation.infons,
        "nodes": nodes,
    })
    .to_string()
}

pub(crate) fn canonical_response_subset(response: &PubTatorExportResponse) -> String {
    let facts = response
        .documents
        .iter()
        .flat_map(|document| {
            let mut relations = document
                .relations
                .iter()
                .map(canonical_relation)
                .collect::<Vec<_>>();
            relations.sort_unstable();
            document.passages.iter().map(move |passage| {
                let annotations = |kind| {
                    let mut values = passage
                        .annotations
                        .iter()
                        .filter(|annotation| {
                            annotation
                                .infons
                                .as_ref()
                                .and_then(|infons| infons.kind.as_deref())
                                == Some(kind)
                        })
                        .map(|annotation| {
                            serde_json::json!({
                                "id": annotation.id,
                                "identifier": annotation.infons.as_ref().and_then(|infons| infons.identifier.as_deref()),
                                "text": annotation.text.as_deref().map(normalized),
                            })
                            .to_string()
                        })
                        .collect::<Vec<_>>();
                    values.sort_unstable();
                    values.dedup();
                    values
                };
                serde_json::json!({
                    "pmid": document.pmid,
                    "section": passage.infons.as_ref().and_then(|infons| infons.kind.as_deref()),
                    "genes": annotations("Gene"),
                    "alleles": annotations("Mutation"),
                    "relations": relations,
                })
                .to_string()
            })
        })
        .collect();
    canonical_subset_hash(facts)
}

pub(crate) fn canonical_content_subset(identity: &VariantArticleIdentity) -> String {
    let facts = identity
        .observations
        .iter()
        .chain(&identity.contradictions)
        .map(|observation| {
            serde_json::json!({
                "source": observation.source,
                "section": observation.section,
                "linked_gene": observation.linked_gene,
                "observed_alias": normalized(&observation.observed_alias),
                "gene_annotation_id": observation.gene_annotation_id,
                "allele_annotation_id": observation.allele_annotation_id,
                "provider_relation": observation.provider_relation,
                "canonical_content_hash": observation.canonical_content_hash,
            })
            .to_string()
        })
        .collect();
    canonical_subset_hash(facts)
}

pub(crate) fn verification_plan(
    requested: &RequestedVariantIdentity,
    provider_template_version: &str,
    response_subsets: &[String],
    content_subsets: &[String],
) -> VariantArticleVerificationPlan {
    let request = serde_json::to_string(requested).unwrap_or_default();
    let canonical_response_subset_hash = canonical_subset_hash(response_subsets.to_vec());
    let canonical_content_subset_hash = canonical_subset_hash(content_subsets.to_vec());
    VariantArticleVerificationPlan {
        verifier_version: VERIFIER_VERSION,
        provider_template_version: provider_template_version.into(),
        response_subset_version: RESPONSE_SUBSET_VERSION,
        content_subset_version: CONTENT_SUBSET_VERSION,
        artifact_id: hash(&format!(
            "{VERIFIER_VERSION}:{provider_template_version}:{request}:{canonical_response_subset_hash}:{canonical_content_subset_hash}"
        )),
        canonical_response_subset_hash,
        canonical_content_subset_hash,
        response_hashes_are_post_response: true,
        captured_content_hashes_are_post_response: true,
    }
}

fn relation_links(
    relation: &PubTatorRelation,
    gene_annotation_id: &str,
    allele_annotation_id: &str,
) -> bool {
    let node_ids = relation
        .nodes
        .iter()
        .filter_map(|node| node.refid.as_deref())
        .collect::<Vec<_>>();
    node_ids.contains(&gene_annotation_id) && node_ids.contains(&allele_annotation_id)
}

pub(crate) fn verify_pubtator(
    requested: &RequestedVariantIdentity,
    response: &PubTatorExportResponse,
    incomplete: bool,
) -> VariantArticleIdentity {
    let requested_gene = requested.gene.as_deref().map(normalized);
    let normalized_requested_allele = requested_allele(requested).map(|value| normalized(&value));
    let mut observations = Vec::new();
    let mut contradictions = Vec::new();
    for (document_index, document) in response.documents.iter().enumerate() {
        for (passage_index, passage) in document.passages.iter().enumerate() {
            let text = passage.text.as_deref().unwrap_or_default();
            let content_hash = hash(text);
            let section = passage
                .infons
                .as_ref()
                .and_then(|infons| infons.kind.as_deref())
                .unwrap_or("unknown")
                .to_string();
            let Some(gene) = requested_gene.as_deref() else {
                continue;
            };
            let Some(allele) = normalized_requested_allele.as_deref() else {
                continue;
            };
            let locator = format!(
                "document:{}:passage:{}",
                document_index + 1,
                passage_index + 1
            );
            for gene_annotation in passage.annotations.iter().filter(|annotation| {
                annotation
                    .infons
                    .as_ref()
                    .and_then(|infons| infons.kind.as_deref())
                    == Some("Gene")
                    && annotation.text.as_deref().map(normalized).as_deref() == Some(gene)
            }) {
                let Some(gene_annotation_id) = gene_annotation.id.as_deref() else {
                    continue;
                };
                for allele_annotation in passage.annotations.iter().filter(|annotation| {
                    annotation
                        .infons
                        .as_ref()
                        .and_then(|infons| infons.kind.as_deref())
                        == Some("Mutation")
                }) {
                    let Some(allele_annotation_id) = allele_annotation.id.as_deref() else {
                        continue;
                    };
                    let Some(observed_allele) = allele_annotation.text.as_deref() else {
                        continue;
                    };
                    let Some(relation) = document.relations.iter().find(|relation| {
                        relation_links(relation, gene_annotation_id, allele_annotation_id)
                    }) else {
                        continue;
                    };
                    let observation = VariantArticleIdentityObservation {
                        source: "pubtator",
                        section: section.clone(),
                        locator: locator.clone(),
                        linked_gene: gene_annotation.text.clone().unwrap_or_default(),
                        observed_alias: observed_allele.into(),
                        gene_annotation_id: gene_annotation_id.into(),
                        allele_annotation_id: allele_annotation_id.into(),
                        provider_relation: canonical_relation(relation),
                        canonical_content_hash: content_hash.clone(),
                    };
                    if normalized(observed_allele) == allele {
                        observations.push(observation);
                    } else {
                        contradictions.push(observation);
                    }
                }
            }
        }
    }
    let status = status_for(&observations, &contradictions);
    VariantArticleIdentity {
        status,
        basis: if status == "confirmed" {
            "structured_annotation"
        } else {
            "none"
        },
        requested_gene: requested.gene.clone(),
        requested_allele: requested_allele(requested),
        observations,
        contradictions,
        incomplete,
    }
}

fn status_for(
    observations: &[VariantArticleIdentityObservation],
    contradictions: &[VariantArticleIdentityObservation],
) -> &'static str {
    if !observations.is_empty() && !contradictions.is_empty() {
        "conflicting"
    } else if !contradictions.is_empty() {
        "contradictory"
    } else if !observations.is_empty() {
        "confirmed"
    } else {
        "unverified"
    }
}

pub(crate) fn verify_captured_abstract(
    requested: &RequestedVariantIdentity,
    abstract_text: &str,
) -> VariantArticleIdentity {
    let requested_gene = requested.gene.as_deref().map(normalized);
    let requested_allele = requested_allele(requested);
    let normalized_allele = requested_allele.as_deref().map(normalized);
    let mut contradictions = Vec::new();
    static SENTENCE_BOUNDARY_RE: OnceLock<Regex> = OnceLock::new();
    let sentence_boundary_re = SENTENCE_BOUNDARY_RE
        .get_or_init(|| Regex::new(r"[.!?]\s+").expect("valid sentence-boundary regex"));
    static ALLELE_RE: OnceLock<Regex> = OnceLock::new();
    let allele_re = ALLELE_RE
        .get_or_init(|| Regex::new(r"(?i)\b[pc]\.[a-z0-9_+\-]+\b").expect("valid allele regex"));
    static GENE_RE: OnceLock<Regex> = OnceLock::new();
    let gene_re =
        GENE_RE.get_or_init(|| Regex::new(r"\b[A-Z]{2,10}[0-9]*\b").expect("valid gene regex"));
    let text_units = if abstract_text.lines().any(|line| line.contains('|')) {
        abstract_text.lines().collect::<Vec<_>>()
    } else {
        sentence_boundary_re
            .split(abstract_text)
            .collect::<Vec<_>>()
    };
    for (index, sentence) in text_units.into_iter().enumerate() {
        let Some(gene) = requested_gene.as_deref() else {
            continue;
        };
        let Some(allele) = normalized_allele.as_deref() else {
            continue;
        };
        let normalized_sentence = sentence.to_ascii_lowercase();
        let genes = gene_re
            .find_iter(sentence)
            .map(|matched| normalized(matched.as_str()))
            .collect::<Vec<_>>();
        if !contains_token(&normalized_sentence, gene) || genes.len() != 1 || genes[0] != gene {
            continue;
        }
        let mut observed_alleles = allele_re
            .find_iter(sentence)
            .map(|matched| matched.as_str())
            .collect::<Vec<_>>();
        observed_alleles.sort_unstable();
        observed_alleles.dedup();
        let observation = |observed_alias: &str| VariantArticleIdentityObservation {
            source: "captured_abstract",
            section: "abstract".into(),
            locator: format!("sentence:{}", index + 1),
            linked_gene: requested.gene.clone().unwrap_or_default(),
            observed_alias: observed_alias.into(),
            gene_annotation_id: String::new(),
            allele_annotation_id: String::new(),
            provider_relation: String::new(),
            canonical_content_hash: hash(sentence),
        };
        if observed_alleles
            .iter()
            .any(|observed| normalized(observed) != allele)
        {
            contradictions.extend(observed_alleles.into_iter().map(observation));
        }
    }
    let status = status_for(&[], &contradictions);
    VariantArticleIdentity {
        status,
        basis: "none",
        requested_gene: requested.gene.clone(),
        requested_allele,
        observations: Vec::new(),
        contradictions,
        incomplete: false,
    }
}

pub(crate) fn combine_identities(
    captured: VariantArticleIdentity,
    fetched: VariantArticleIdentity,
) -> VariantArticleIdentity {
    let mut observations = captured.observations;
    observations.extend(fetched.observations);
    let mut contradictions = captured.contradictions;
    contradictions.extend(fetched.contradictions);
    let status = status_for(&observations, &contradictions);
    VariantArticleIdentity {
        status,
        basis: if status == "confirmed" {
            if fetched.basis != "none" {
                fetched.basis
            } else {
                captured.basis
            }
        } else {
            "none"
        },
        requested_gene: captured.requested_gene.or(fetched.requested_gene),
        requested_allele: captured.requested_allele.or(fetched.requested_allele),
        observations,
        contradictions,
        incomplete: captured.incomplete || fetched.incomplete,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::pubtator::{
        PubTatorAnnotation, PubTatorAnnotationInfons, PubTatorDocument, PubTatorInfons,
        PubTatorPassage, PubTatorRelation, PubTatorRelationNode,
    };

    fn requested() -> RequestedVariantIdentity {
        RequestedVariantIdentity::from_variant_input("BRAF p.V600E").expect("identity")
    }

    fn response(alleles: &[&str]) -> PubTatorExportResponse {
        PubTatorExportResponse {
            documents: vec![PubTatorDocument {
                pmid: Some(1),
                pmcid: None,
                date: None,
                journal: None,
                authors: vec![],
                relations: alleles
                    .iter()
                    .enumerate()
                    .map(|(index, _)| PubTatorRelation {
                        id: Some(format!("relation-{}", index + 1)),
                        infons: Some(serde_json::json!({"type": "gene_variant"})),
                        nodes: vec![
                            PubTatorRelationNode {
                                refid: Some("gene-1".into()),
                                role: Some("gene".into()),
                            },
                            PubTatorRelationNode {
                                refid: Some(format!("allele-{}", index + 1)),
                                role: Some("mutation".into()),
                            },
                        ],
                    })
                    .collect(),
                passages: vec![PubTatorPassage {
                    infons: Some(PubTatorInfons {
                        kind: Some("abstract".into()),
                    }),
                    text: Some("captured content".into()),
                    annotations: std::iter::once(("Gene", "BRAF"))
                        .chain(alleles.iter().map(|allele| ("Mutation", *allele)))
                        .enumerate()
                        .map(|(index, (kind, text))| PubTatorAnnotation {
                            id: Some(if kind == "Gene" {
                                "gene-1".into()
                            } else {
                                format!("allele-{index}")
                            }),
                            text: Some(text.into()),
                            infons: Some(PubTatorAnnotationInfons {
                                kind: Some(kind.into()),
                                identifier: Some(format!("provider-{index}")),
                            }),
                        })
                        .collect(),
                }],
            }],
        }
    }

    #[test]
    fn provider_linked_annotation_confirms_and_hashes_captured_content() {
        let identity = verify_pubtator(&requested(), &response(&["p.V600E"]), false);
        assert_eq!(identity.status, "confirmed");
        assert_eq!(identity.observations[0].linked_gene, "BRAF");
        assert_eq!(identity.observations[0].gene_annotation_id, "gene-1");
        assert_eq!(identity.observations[0].allele_annotation_id, "allele-1");
        assert!(!identity.observations[0].provider_relation.is_empty());
        assert!(!identity.observations[0].canonical_content_hash.is_empty());
    }

    #[test]
    fn same_passage_gene_and_unlinked_allele_is_unverified() {
        let mut unlinked = response(&["p.V600E"]);
        unlinked.documents[0].relations.clear();
        assert_eq!(
            verify_pubtator(&requested(), &unlinked, false).status,
            "unverified"
        );
    }

    #[test]
    fn alternate_allele_is_contradictory_and_mixed_evidence_conflicts() {
        assert_eq!(
            verify_pubtator(&requested(), &response(&["p.V600K"]), false).status,
            "contradictory"
        );
        let mut conflicting = response(&["p.V600E"]);
        let mut contradictory = response(&["p.V600K"]);
        conflicting.documents[0]
            .passages
            .append(&mut contradictory.documents[0].passages);
        let identity = verify_pubtator(&requested(), &conflicting, false);
        assert_eq!(identity.status, "conflicting");
        assert_eq!(identity.contradictions.len(), 1);
    }

    #[test]
    fn mixed_same_passage_evidence_is_conflicting_and_incomplete_does_not_hide_it() {
        let identity = verify_pubtator(&requested(), &response(&["p.V600E", "p.V600K"]), true);
        assert_eq!(identity.status, "conflicting");
        assert_eq!(identity.observations.len(), 1);
        assert_eq!(identity.contradictions.len(), 1);
        assert!(identity.incomplete);
    }

    #[test]
    fn captured_sentences_cannot_confirm_without_provider_linkage() {
        let same_sentence = verify_captured_abstract(&requested(), "BRAF p.V600E was observed.");
        assert_eq!(same_sentence.status, "unverified");
        let separate_sentences = verify_captured_abstract(
            &requested(),
            "BRAF was observed. The tumour carried p.V600E.",
        );
        assert_eq!(separate_sentences.status, "unverified");
        let contradictory = verify_captured_abstract(&requested(), "BRAF p.V600E and p.V600K.");
        assert_eq!(contradictory.status, "contradictory");
        let table = verify_captured_abstract(&requested(), "gene | allele\nBRAF | p.V600E");
        assert_eq!(table.status, "unverified");
        let second_gene = verify_captured_abstract(&requested(), "BRAF and ATM p.V600E.");
        assert_eq!(second_gene.status, "unverified");
    }

    #[test]
    fn canonical_subsets_ignore_annotation_order_but_include_provider_linkage() {
        let ordered = response(&["p.V600E", "p.V600K"]);
        let mut reordered = response(&["p.V600E", "p.V600K"]);
        reordered.documents[0].passages[0].annotations.reverse();
        reordered.documents[0].relations.reverse();
        reordered.documents[0].relations[0].nodes.reverse();
        let mut changed_identifier = response(&["p.V600E", "p.V600K"]);
        changed_identifier.documents[0].passages[0].annotations[0].id = Some("gene-other".into());
        let mut changed_link = response(&["p.V600E", "p.V600K"]);
        changed_link.documents[0].relations[0].nodes[1].refid = Some("allele-other".into());

        assert_eq!(
            canonical_response_subset(&ordered),
            canonical_response_subset(&reordered)
        );
        assert_eq!(
            canonical_content_subset(&verify_pubtator(&requested(), &ordered, false)),
            canonical_content_subset(&verify_pubtator(&requested(), &reordered, false))
        );
        assert_ne!(
            canonical_response_subset(&ordered),
            canonical_response_subset(&changed_identifier)
        );
        assert_ne!(
            canonical_response_subset(&ordered),
            canonical_response_subset(&changed_link)
        );
    }

    #[test]
    fn unavailable_verification_is_incomplete_and_unverified() {
        let identity = verify_pubtator(
            &requested(),
            &PubTatorExportResponse {
                documents: Vec::new(),
            },
            true,
        );
        assert_eq!(identity.status, "unverified");
        assert!(identity.incomplete);
    }

    #[test]
    fn artifact_is_post_response_and_includes_the_provider_template_version() {
        let response_hashes = ["response".into()];
        let content_hashes = ["content".into()];
        let plan = verification_plan(
            &requested(),
            PUBTATOR_EXPORT_TEMPLATE_VERSION,
            &response_hashes,
            &content_hashes,
        );
        let different_template = verification_plan(
            &requested(),
            "pubtator-export-biocjson-v2",
            &response_hashes,
            &content_hashes,
        );
        assert!(plan.response_hashes_are_post_response);
        assert_eq!(plan.response_subset_version, RESPONSE_SUBSET_VERSION);
        assert_eq!(plan.content_subset_version, CONTENT_SUBSET_VERSION);
        assert_eq!(
            plan.provider_template_version,
            PUBTATOR_EXPORT_TEMPLATE_VERSION
        );
        assert_ne!(plan.artifact_id, different_template.artifact_id);
    }
}
