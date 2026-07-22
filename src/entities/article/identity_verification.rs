use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::entities::variant::RequestedVariantIdentity;
use crate::sources::pubtator::PubTatorExportResponse;

pub(crate) const VERIFIER_VERSION: &str = "article-identity-v1";

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
    pub canonical_content_hash: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VariantArticleVerificationPlan {
    pub verifier_version: &'static str,
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

pub(crate) fn verification_plan(
    requested: &RequestedVariantIdentity,
    response_hashes: &[String],
    content_hashes: &[String],
) -> VariantArticleVerificationPlan {
    let request = serde_json::to_string(requested).unwrap_or_default();
    VariantArticleVerificationPlan {
        verifier_version: VERIFIER_VERSION,
        artifact_id: hash(&format!(
            "{VERIFIER_VERSION}:{request}:{}:{}",
            response_hashes.join(","),
            content_hashes.join(",")
        )),
        response_hashes_are_post_response: true,
        captured_content_hashes_are_post_response: true,
    }
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
            let genes = passage
                .annotations
                .iter()
                .filter(|annotation| {
                    annotation
                        .infons
                        .as_ref()
                        .and_then(|infons| infons.kind.as_deref())
                        == Some("Gene")
                })
                .filter_map(|annotation| annotation.text.as_deref())
                .map(normalized)
                .collect::<Vec<_>>();
            let alleles = passage
                .annotations
                .iter()
                .filter(|annotation| {
                    annotation
                        .infons
                        .as_ref()
                        .and_then(|infons| infons.kind.as_deref())
                        == Some("Mutation")
                })
                .filter_map(|annotation| annotation.text.as_deref())
                .map(str::to_string)
                .collect::<Vec<_>>();
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
            let matching_gene = genes.iter().any(|candidate| candidate == gene);
            let matching_allele = alleles
                .iter()
                .any(|candidate| normalized(candidate) == allele);
            let observation = |observed_alias: String| VariantArticleIdentityObservation {
                source: "pubtator",
                section: section.clone(),
                locator: locator.clone(),
                linked_gene: requested.gene.clone().unwrap_or_default(),
                observed_alias,
                canonical_content_hash: content_hash.clone(),
            };
            if matching_gene && matching_allele && genes.len() == 1 && alleles.len() == 1 {
                observations.push(observation(alleles[0].clone()));
            } else if matching_gene && !alleles.is_empty() {
                contradictions.extend(alleles.into_iter().map(observation));
            }
        }
    }
    let status = if incomplete {
        "unverified"
    } else if !observations.is_empty() && !contradictions.is_empty() {
        "conflicting"
    } else if !contradictions.is_empty() {
        "contradictory"
    } else if !observations.is_empty() {
        "confirmed"
    } else {
        "unverified"
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::pubtator::{
        PubTatorAnnotation, PubTatorAnnotationInfons, PubTatorDocument, PubTatorInfons,
        PubTatorPassage,
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
                passages: vec![PubTatorPassage {
                    infons: Some(PubTatorInfons {
                        kind: Some("abstract".into()),
                    }),
                    text: Some("captured content".into()),
                    annotations: std::iter::once(("Gene", "BRAF"))
                        .chain(alleles.iter().map(|allele| ("Mutation", *allele)))
                        .map(|(kind, text)| PubTatorAnnotation {
                            text: Some(text.into()),
                            infons: Some(PubTatorAnnotationInfons {
                                kind: Some(kind.into()),
                                identifier: None,
                            }),
                        })
                        .collect(),
                }],
            }],
        }
    }

    #[test]
    fn structured_annotation_confirms_and_hashes_captured_content() {
        let identity = verify_pubtator(&requested(), &response(&["p.V600E"]), false);
        assert_eq!(identity.status, "confirmed");
        assert_eq!(identity.observations[0].linked_gene, "BRAF");
        assert!(!identity.observations[0].canonical_content_hash.is_empty());
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
    fn artifact_is_post_response_and_separate_from_requested_identity() {
        let plan = verification_plan(&requested(), &["response".into()], &["content".into()]);
        assert!(plan.response_hashes_are_post_response);
        assert_ne!(plan.artifact_id, "BRAF p.V600E");
    }
}
