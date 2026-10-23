//! Query binding and lossless candidate storage for immutable article pools.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::entities::article::{ArticleSearchResult, ArticleSource};
use crate::entities::article::candidates::{ArticleCandidate, ArticleSourcePosition};
use crate::entities::variant::RequestedVariantIdentity;
use super::{VariantArticleCandidateTraceRecord, VariantArticleProvenance, VariantArticleStrategy};
use super::super::identity_verification::{VariantArticleIdentity, VariantArticleVerificationOptions};

/// This wrapper retains the fields deliberately omitted by public article JSON.
#[derive(Serialize, Deserialize)]
pub(super) struct StoredCandidate {
    row: ArticleSearchResult,
    normalized_title: String,
    normalized_abstract: String,
    publication_type: Option<String>,
    source_local_position: usize,
    source_positions: Vec<(ArticleSource, usize)>,
    semantic_signal: Option<f64>,
    provenance: Vec<VariantArticleProvenance>,
    trace: Vec<VariantArticleCandidateTraceRecord>,
    identity: Option<StoredIdentity>,
}

impl From<ArticleCandidate> for StoredCandidate {
    fn from(value: ArticleCandidate) -> Self {
        Self {
            normalized_title: value.row.normalized_title.clone(),
            normalized_abstract: value.row.normalized_abstract.clone(),
            publication_type: value.row.publication_type.clone(),
            source_local_position: value.row.source_local_position,
            row: value.row,
            source_positions: value.source_positions.into_iter().map(|p| (p.source, p.local_position)).collect(),
            semantic_signal: value.semantic_signal,
            provenance: value.variant_provenance,
            trace: value.candidate_trace,
            identity: value.identity.map(Into::into),
        }
    }
}
impl From<StoredCandidate> for ArticleCandidate {
    fn from(value: StoredCandidate) -> Self {
        let mut row = value.row;
        row.normalized_title = value.normalized_title;
        row.normalized_abstract = value.normalized_abstract;
        row.publication_type = value.publication_type;
        row.source_local_position = value.source_local_position;
        Self {
            row,
            source_positions: value.source_positions.into_iter().map(|(source, local_position)| ArticleSourcePosition { source, local_position }).collect(),
            semantic_signal: value.semantic_signal,
            variant_provenance: value.provenance,
            candidate_trace: value.trace,
            identity: value.identity.map(Into::into),
        }
    }
}

/// Only semantic settings enter this binding. Credentials never enter its input.
/// Unsafe custom endpoints disable persistence instead of hashing secrets.
pub(super) fn query_key(
    literal: &str,
    requested: &RequestedVariantIdentity,
    strategy: VariantArticleStrategy,
    verification: VariantArticleVerificationOptions,
) -> Option<String> {
    let mut settings = BTreeMap::new();
    for name in [
        "BIOMCP_MYVARIANT_BASE", "BIOMCP_MYGENE_BASE", "BIOMCP_PUBMED_BASE",
        "BIOMCP_EUROPEPMC_BASE", "BIOMCP_S2_BASE", "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_LITVAR_BASE", "BIOMCP_CLINGEN_CAR_BASE", "BIOMCP_CLINGEN_LDH_FIXTURE_ORIGIN",
    ] {
        if let Ok(value) = std::env::var(name) {
            let url = url::Url::parse(value.trim()).ok()?;
            if !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() {
                return None;
            }
            settings.insert(name, url.to_string().trim_end_matches('/').to_owned());
        }
    }
    if let Ok(disabled) = std::env::var("BIOMCP_DISABLE_SOURCES") {
        let mut disabled = disabled.split([',', '|']).map(|v| v.trim().to_ascii_lowercase()).filter(|v| !v.is_empty()).collect::<Vec<_>>();
        disabled.sort();
        disabled.dedup();
        settings.insert("BIOMCP_DISABLE_SOURCES", disabled.join(","));
    }
    let binding = serde_json::to_vec(&(
        1_u32, super::page::ORDERING_VERSION,
        super::super::identity_verification::VERIFIER_VERSION,
        literal.split_whitespace().collect::<Vec<_>>().join(" "), requested,
        strategy, verification.verify_identity, verification.confirmed_only, settings,
    )).ok()?;
    Some(format!("{:x}", Sha256::digest(binding)))
}

/// Identity observations have a compact public serializer. Store their full
/// internal graph here so restoration cannot discard typed linkage fields.
#[derive(Serialize, Deserialize)]
struct StoredIdentity {
    #[serde(deserialize_with = "crate::cache::variant_article_atoms::atom")]
    status: &'static str,
    #[serde(deserialize_with = "crate::cache::variant_article_atoms::atom")]
    basis: &'static str,
    #[serde(deserialize_with = "crate::cache::variant_article_atoms::optional")]
    reason: Option<&'static str>,
    requested_gene: Option<String>,
    requested_allele: Option<String>,
    observations: Vec<StoredObservation>,
    contradictions: Vec<StoredObservation>,
    incomplete: bool,
}

#[derive(Serialize, Deserialize)]
struct StoredObservation {
    #[serde(deserialize_with = "crate::cache::variant_article_atoms::atom")]
    source: &'static str,
    section: String,
    locator: String,
    linked_gene: String,
    observed_alias: String,
    gene_annotation_id: String,
    allele_annotation_id: String,
    provider_relation: Option<String>,
    provider_linkage: Option<super::super::identity_verification::ProviderLinkage>,
    canonical_content_hash: String,
}

impl From<VariantArticleIdentity> for StoredIdentity {
    fn from(value: VariantArticleIdentity) -> Self {
        Self {
            status: value.status, basis: value.basis, reason: value.reason,
            requested_gene: value.requested_gene, requested_allele: value.requested_allele,
            observations: value.observations.into_iter().map(Into::into).collect(),
            contradictions: value.contradictions.into_iter().map(Into::into).collect(),
            incomplete: value.incomplete,
        }
    }
}
impl From<StoredIdentity> for VariantArticleIdentity {
    fn from(value: StoredIdentity) -> Self {
        Self {
            status: value.status, basis: value.basis, reason: value.reason,
            requested_gene: value.requested_gene, requested_allele: value.requested_allele,
            observations: value.observations.into_iter().map(Into::into).collect(),
            contradictions: value.contradictions.into_iter().map(Into::into).collect(),
            incomplete: value.incomplete,
        }
    }
}
impl From<super::super::identity_verification::VariantArticleIdentityObservation> for StoredObservation {
    fn from(value: super::super::identity_verification::VariantArticleIdentityObservation) -> Self {
        Self {
            source: value.source, section: value.section, locator: value.locator,
            linked_gene: value.linked_gene, observed_alias: value.observed_alias,
            gene_annotation_id: value.gene_annotation_id, allele_annotation_id: value.allele_annotation_id,
            provider_relation: value.provider_relation, provider_linkage: value.provider_linkage,
            canonical_content_hash: value.canonical_content_hash,
        }
    }
}
impl From<StoredObservation> for super::super::identity_verification::VariantArticleIdentityObservation {
    fn from(value: StoredObservation) -> Self {
        Self {
            source: value.source, section: value.section, locator: value.locator,
            linked_gene: value.linked_gene, observed_alias: value.observed_alias,
            gene_annotation_id: value.gene_annotation_id, allele_annotation_id: value.allele_annotation_id,
            provider_relation: value.provider_relation, provider_linkage: value.provider_linkage,
            canonical_content_hash: value.canonical_content_hash,
        }
    }
}
