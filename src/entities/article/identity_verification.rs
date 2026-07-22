use std::sync::OnceLock;

use regex::Regex;
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
            let observation = |observed_alias: String| VariantArticleIdentityObservation {
                source: "pubtator",
                section: section.clone(),
                locator: locator.clone(),
                linked_gene: requested.gene.clone().unwrap_or_default(),
                observed_alias,
                canonical_content_hash: content_hash.clone(),
            };
            if matching_gene && genes.len() == 1 {
                for observed_allele in alleles {
                    if normalized(&observed_allele) == allele {
                        observations.push(observation(observed_allele));
                    } else {
                        contradictions.push(observation(observed_allele));
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
    let mut observations = Vec::new();
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
            canonical_content_hash: hash(sentence),
        };
        if observed_alleles.len() == 1 && normalized(observed_alleles[0]) == allele {
            observations.push(observation(observed_alleles[0]));
        } else if observed_alleles
            .iter()
            .any(|observed| normalized(observed) != allele)
        {
            contradictions.extend(observed_alleles.into_iter().map(observation));
        }
    }
    let status = status_for(&observations, &contradictions);
    VariantArticleIdentity {
        status,
        basis: if status == "confirmed" {
            "sentence"
        } else {
            "none"
        },
        requested_gene: requested.gene.clone(),
        requested_allele,
        observations,
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
    fn mixed_same_passage_evidence_is_conflicting_and_incomplete_does_not_hide_it() {
        let identity = verify_pubtator(&requested(), &response(&["p.V600E", "p.V600K"]), true);
        assert_eq!(identity.status, "conflicting");
        assert_eq!(identity.observations.len(), 1);
        assert_eq!(identity.contradictions.len(), 1);
        assert!(identity.incomplete);
    }

    #[test]
    fn one_gene_one_allele_captured_sentence_confirms_but_article_wide_cooccurrence_does_not() {
        let confirmed = verify_captured_abstract(&requested(), "BRAF p.V600E was observed.");
        assert_eq!(confirmed.status, "confirmed");
        let unverified = verify_captured_abstract(
            &requested(),
            "BRAF was observed. The tumour carried p.V600E.",
        );
        assert_eq!(unverified.status, "unverified");
        let contradictory = verify_captured_abstract(&requested(), "BRAF p.V600E and p.V600K.");
        assert_eq!(contradictory.status, "contradictory");
        let table = verify_captured_abstract(&requested(), "gene | allele\nBRAF | p.V600E");
        assert_eq!(table.status, "confirmed");
        let second_gene = verify_captured_abstract(&requested(), "BRAF and ATM p.V600E.");
        assert_eq!(second_gene.status, "unverified");
    }

    #[test]
    fn artifact_is_post_response_and_separate_from_requested_identity() {
        let plan = verification_plan(&requested(), &["response".into()], &["content".into()]);
        assert!(plan.response_hashes_are_post_response);
        assert_ne!(plan.artifact_id, "BRAF p.V600E");
    }
}
