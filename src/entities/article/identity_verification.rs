use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::entities::variant::RequestedVariantIdentity;
use crate::sources::pubtator::{PubTatorAnnotation, PubTatorExportResponse, PubTatorRelation};

pub(crate) const VERIFIER_VERSION: &str = "article-identity-v2";
pub(crate) const PUBTATOR_EXPORT_TEMPLATE_VERSION: &str = "pubtator-export-biocjson-v1";
const RESPONSE_SUBSET_VERSION: &str = "clinically-relevant-response-v1";
const CONTENT_SUBSET_VERSION: &str = "clinically-relevant-content-v1";
const MAX_ANNOTATION_ID_BYTES: usize = 256;
const MAX_HGVS_BYTES: usize = 1024;
const MAX_IDENTIFIER_TOKENS: usize = 16;
const MAX_IDENTIFIER_TOKEN_BYTES: usize = 256;

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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct VariantArticleIdentityObservation {
    pub source: &'static str,
    pub section: String,
    pub locator: String,
    pub linked_gene: String,
    pub observed_alias: String,
    pub gene_annotation_id: String,
    pub allele_annotation_id: String,
    pub provider_relation: Option<String>,
    pub provider_linkage: Option<ProviderLinkage>,
    pub canonical_content_hash: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ProviderLinkage {
    kind: &'static str,
    expected_pmid: String,
    returned_pmid: String,
    gene_annotation_id: String,
    variant_annotation_id: String,
    gene_id: u64,
    observed_hgvs: String,
    identifier_tokens: Vec<String>,
    relation_id: Option<String>,
    relation_type: Option<String>,
    relation_roles: Option<Vec<String>>,
    provenance: ProviderLinkageProvenance,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ProviderLinkageProvenance {
    source: &'static str,
    request_template_version: &'static str,
    verifier_version: &'static str,
    response_subset_version: &'static str,
    canonical_response_subset_sha256: String,
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
        .map(|node| serde_json::json!({"refid": node.refid, "role": node.role}).to_string())
        .collect::<Vec<_>>();
    nodes.sort_unstable();
    serde_json::json!({"id": relation.id, "infons": relation.infons, "nodes": nodes}).to_string()
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
                let mut annotations = passage
                    .annotations
                    .iter()
                    .map(|annotation| {
                        let infons = annotation.infons.as_ref();
                        serde_json::json!({
                            "id": annotation.id,
                            "type": infons.and_then(|value| value.kind.as_deref()),
                            "name": infons.and_then(|value| value.name.as_deref()),
                            "identifier": infons.and_then(|value| value.identifier.as_deref()),
                            "normalized_id": infons.and_then(|value| value.normalized_id),
                            "hgvs": infons.and_then(|value| value.hgvs.as_deref()),
                            "gene_id": infons.and_then(|value| value.gene_id),
                            "gene_ids": infons.and_then(|value| value.gene_ids.as_ref()),
                        })
                        .to_string()
                    })
                    .collect::<Vec<_>>();
                annotations.sort_unstable();
                annotations.dedup();
                serde_json::json!({
                    "id": document.id,
                    "pmid": document.pmid,
                    "section": passage.infons.as_ref().and_then(|infons| infons.kind.as_deref()),
                    "annotations": annotations,
                    "relations": relations,
                })
                .to_string()
            })
        })
        .collect();
    canonical_subset_hash(facts)
}

pub(crate) fn canonical_content_subset(identity: &VariantArticleIdentity) -> String {
    canonical_subset_hash(
        identity
            .observations
            .iter()
            .chain(&identity.contradictions)
            .map(|observation| serde_json::to_string(observation).unwrap_or_default())
            .collect(),
    )
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

fn canonical_pmid(value: &str) -> Option<&str> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())).then_some(value)
}

fn bounded(value: &str, limit: usize) -> bool {
    !value.is_empty() && value.len() <= limit
}

fn identifier_tokens(value: &str) -> Option<Vec<String>> {
    let mut tokens = value
        .split(';')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    (tokens.len() <= MAX_IDENTIFIER_TOKENS
        && tokens
            .iter()
            .all(|token| bounded(token, MAX_IDENTIFIER_TOKEN_BYTES)))
    .then(|| {
        tokens.sort_unstable();
        tokens.dedup();
        tokens
    })
}

fn same_coordinate(left: &str, right: &str) -> bool {
    let coordinate = |value: &str| {
        value
            .char_indices()
            .take_while(|(_, character)| !character.is_ascii_digit() || *character == '.')
            .last()
            .map_or(0, |(index, character)| index + character.len_utf8())
            + value
                .chars()
                .skip_while(|character| !character.is_ascii_digit())
                .take_while(|character| character.is_ascii_digit())
                .map(char::len_utf8)
                .sum::<usize>()
    };
    let left_end = coordinate(left);
    left_end > 0 && left.get(..left_end) == right.get(..left_end)
}

fn typed_gene(annotation: &PubTatorAnnotation) -> Option<(&str, u64, &str)> {
    let infons = annotation.infons.as_ref()?;
    let name = infons.name.as_deref()?;
    let identifier = infons.identifier.as_deref()?;
    let id = identifier.parse::<u64>().ok()?;
    (infons.kind.as_deref() == Some("Gene")
        && bounded(annotation.id.as_deref()?, MAX_ANNOTATION_ID_BYTES)
        && !name.is_empty()
        && identifier.bytes().all(|byte| byte.is_ascii_digit())
        && infons
            .normalized_id
            .is_none_or(|normalized_id| normalized_id == id))
    .then_some((name, id, annotation.id.as_deref()?))
}

fn typed_variant(annotation: &PubTatorAnnotation) -> Option<(&str, u64, Vec<String>, &str)> {
    let infons = annotation.infons.as_ref()?;
    let hgvs = infons.hgvs.as_deref()?;
    let identifier = infons.identifier.as_deref()?;
    let gene_id = infons.gene_id?;
    let gene_ids = infons.gene_ids.as_ref()?;
    let tokens = identifier_tokens(identifier)?;
    (infons.kind.as_deref() == Some("Variant")
        && bounded(annotation.id.as_deref()?, MAX_ANNOTATION_ID_BYTES)
        && bounded(hgvs, MAX_HGVS_BYTES)
        && gene_ids.contains(&gene_id)
        && tokens
            .iter()
            .any(|token| token == &format!("CorrespondingGene:{gene_id}")))
    .then_some((hgvs, gene_id, tokens, annotation.id.as_deref()?))
}

pub(crate) fn verify_pubtator(
    requested: &RequestedVariantIdentity,
    expected_pmid: &str,
    response: &PubTatorExportResponse,
    incomplete: bool,
) -> VariantArticleIdentity {
    let requested_gene = requested.gene.as_deref().map(normalized);
    let requested_allele = requested_allele(requested);
    let normalized_requested_allele = requested_allele.as_deref().map(normalized);
    let response_digest = canonical_response_subset(response);
    let mut observations = Vec::new();
    let mut contradictions = Vec::new();
    let mut semantic_anomaly = canonical_pmid(expected_pmid).is_none();

    for document in &response.documents {
        let Some(returned_pmid) = document.id.as_deref().and_then(canonical_pmid) else {
            semantic_anomaly = true;
            continue;
        };
        if document
            .pmid
            .is_some_and(|pmid| pmid.to_string() != returned_pmid)
            || returned_pmid != expected_pmid
        {
            semantic_anomaly = true;
            continue;
        }
        let (Some(gene), Some(allele)) = (
            requested_gene.as_deref(),
            normalized_requested_allele.as_deref(),
        ) else {
            continue;
        };
        for (passage_index, passage) in document.passages.iter().enumerate() {
            let section = passage
                .infons
                .as_ref()
                .and_then(|infons| infons.kind.as_deref())
                .unwrap_or("unknown")
                .to_string();
            let content_hash = hash(passage.text.as_deref().unwrap_or_default());
            let locator = format!("document:{returned_pmid}:passage:{}", passage_index + 1);
            for gene_annotation in &passage.annotations {
                let Some((gene_name, gene_id, gene_annotation_id)) = typed_gene(gene_annotation)
                else {
                    continue;
                };
                for variant_annotation in &passage.annotations {
                    let Some((observed_hgvs, variant_gene_id, tokens, variant_annotation_id)) =
                        typed_variant(variant_annotation)
                    else {
                        continue;
                    };
                    if variant_gene_id != gene_id {
                        continue;
                    }
                    let matches_gene = normalized(gene_name) == gene;
                    let matches_allele = normalized(observed_hgvs) == allele;
                    if !(matches_allele
                        || matches_gene
                            && same_coordinate(
                                observed_hgvs,
                                &requested_allele.clone().unwrap_or_default(),
                            ))
                    {
                        continue;
                    }
                    let observation = VariantArticleIdentityObservation {
                        source: "pubtator",
                        section: section.clone(),
                        locator: locator.clone(),
                        linked_gene: gene_name.into(),
                        observed_alias: observed_hgvs.into(),
                        gene_annotation_id: gene_annotation_id.into(),
                        allele_annotation_id: variant_annotation_id.into(),
                        provider_relation: None,
                        provider_linkage: Some(ProviderLinkage {
                            kind: "pubtator_corresponding_gene",
                            expected_pmid: expected_pmid.into(),
                            returned_pmid: returned_pmid.into(),
                            gene_annotation_id: gene_annotation_id.into(),
                            variant_annotation_id: variant_annotation_id.into(),
                            gene_id,
                            observed_hgvs: observed_hgvs.into(),
                            identifier_tokens: tokens,
                            relation_id: None,
                            relation_type: None,
                            relation_roles: None,
                            provenance: ProviderLinkageProvenance {
                                source: "pubtator3",
                                request_template_version: PUBTATOR_EXPORT_TEMPLATE_VERSION,
                                verifier_version: VERIFIER_VERSION,
                                response_subset_version: RESPONSE_SUBSET_VERSION,
                                canonical_response_subset_sha256: response_digest.clone(),
                            },
                        }),
                        canonical_content_hash: content_hash.clone(),
                    };
                    if matches_gene && matches_allele {
                        observations.push(observation);
                    } else if (matches_allele && !matches_gene) || (matches_gene && !matches_allele)
                    {
                        contradictions.push(observation);
                    }
                }
            }
        }
    }
    observations.sort_by_key(|observation| serde_json::to_string(observation).unwrap_or_default());
    observations.dedup();
    contradictions
        .sort_by_key(|observation| serde_json::to_string(observation).unwrap_or_default());
    contradictions.dedup();
    let status = status_for(&observations, &contradictions);
    VariantArticleIdentity {
        status,
        basis: if status == "confirmed" {
            "structured_annotation"
        } else {
            "none"
        },
        requested_gene: requested.gene.clone(),
        requested_allele,
        observations,
        contradictions,
        incomplete: incomplete || semantic_anomaly || response.documents.is_empty(),
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
        let (Some(gene), Some(allele)) = (requested_gene.as_deref(), normalized_allele.as_deref())
        else {
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
            provider_relation: None,
            provider_linkage: None,
            canonical_content_hash: hash(sentence),
        };
        if observed_alleles
            .iter()
            .any(|observed| normalized(observed) != allele)
        {
            contradictions.extend(observed_alleles.into_iter().map(observation));
        }
    }
    VariantArticleIdentity {
        status: status_for(&[], &contradictions),
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
        PubTatorAnnotationInfons, PubTatorDocument, PubTatorInfons, PubTatorPassage,
        PubTatorRelation, PubTatorRelationNode,
    };

    fn requested() -> RequestedVariantIdentity {
        RequestedVariantIdentity::from_variant_input("BRAF p.V600E").expect("identity")
    }

    fn response(relation: bool) -> PubTatorExportResponse {
        PubTatorExportResponse {
            documents: vec![PubTatorDocument {
                id: Some("1".into()),
                pmid: Some(1),
                pmcid: None,
                date: None,
                journal: None,
                authors: vec![],
                passages: vec![PubTatorPassage {
                    infons: Some(PubTatorInfons {
                        kind: Some("abstract".into()),
                    }),
                    text: Some("captured".into()),
                    annotations: vec![
                        PubTatorAnnotation {
                            id: Some("gene-1".into()),
                            text: None,
                            infons: Some(PubTatorAnnotationInfons {
                                kind: Some("Gene".into()),
                                name: Some("BRAF".into()),
                                identifier: Some("673".into()),
                                normalized_id: Some(673),
                                hgvs: None,
                                gene_id: None,
                                gene_ids: None,
                            }),
                        },
                        PubTatorAnnotation {
                            id: Some("variant-1".into()),
                            text: None,
                            infons: Some(PubTatorAnnotationInfons {
                                kind: Some("Variant".into()),
                                name: None,
                                identifier: Some("Variant:p.V600E;CorrespondingGene:673".into()),
                                normalized_id: None,
                                hgvs: Some("p.V600E".into()),
                                gene_id: Some(673),
                                gene_ids: Some(vec![673]),
                            }),
                        },
                    ],
                }],
                relations: if relation {
                    vec![PubTatorRelation {
                        id: Some("relation-1".into()),
                        infons: Some(serde_json::json!({"type":"Association"})),
                        nodes: vec![
                            PubTatorRelationNode {
                                refid: Some("gene-1".into()),
                                role: Some("subject".into()),
                            },
                            PubTatorRelationNode {
                                refid: Some("variant-1".into()),
                                role: Some("object".into()),
                            },
                        ],
                    }]
                } else {
                    Vec::new()
                },
            }],
        }
    }

    #[test]
    fn arbitrary_relation_membership_cannot_confirm_typed_pubtator_linkage() {
        let mut arbitrary = response(true);
        arbitrary.documents[0].passages[0].annotations[1]
            .infons
            .as_mut()
            .unwrap()
            .gene_id = None;
        assert_eq!(
            verify_pubtator(&requested(), "1", &arbitrary, false).status,
            "unverified"
        );
    }

    #[test]
    fn typed_corresponding_gene_requires_the_returned_pmid() {
        let mut wrong = response(false);
        wrong.documents[0].id = Some("2".into());
        assert_eq!(
            verify_pubtator(&requested(), "1", &wrong, false).status,
            "unverified"
        );
        assert!(verify_pubtator(&requested(), "1", &wrong, false).incomplete);
    }

    #[test]
    fn typed_corresponding_gene_confirms_and_deduplicates() {
        let mut duplicate = response(false);
        duplicate
            .documents
            .push(response(false).documents.remove(0));
        let identity = verify_pubtator(&requested(), "1", &duplicate, false);
        assert_eq!(identity.status, "confirmed");
        assert_eq!(identity.observations.len(), 1);
    }

    #[test]
    fn unavailable_verification_is_incomplete_and_unverified() {
        let identity = verify_pubtator(
            &requested(),
            "1",
            &PubTatorExportResponse {
                documents: Vec::new(),
            },
            true,
        );
        assert_eq!(identity.status, "unverified");
        assert!(identity.incomplete);
    }
}
