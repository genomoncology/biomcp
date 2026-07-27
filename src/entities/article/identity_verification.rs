use std::sync::OnceLock;

use regex::Regex;
use serde::Serialize;
use serde::ser::SerializeStruct;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::entities::variant::RequestedVariantIdentity;
use crate::sources::pubtator::{PubTatorAnnotation, PubTatorExportResponse};

pub(crate) const VERIFIER_VERSION: &str = "article-identity-v2";
pub(crate) const PUBTATOR_EXPORT_TEMPLATE_VERSION: &str = "pubtator-export-biocjson-v1";
const RESPONSE_SUBSET_VERSION: &str = "clinically-relevant-response-v1";
const CONTENT_SUBSET_VERSION: &str = "clinically-relevant-content-v1";
const MAX_ANNOTATION_ID_BYTES: usize = 256;
const MAX_HGVS_BYTES: usize = 1024;
const MAX_IDENTIFIER_TOKENS: usize = 16;
const MAX_IDENTIFIER_TOKEN_BYTES: usize = 256;
const MAX_LDH_CAID_BYTES: usize = 32;
const MAX_LDH_PMCID_BYTES: usize = 16;
const MAX_LDH_SELECTOR_TYPE_BYTES: usize = 64;
const MAX_LDH_SELECTOR_VALUE_BYTES: usize = 1024;
const MAX_LDH_ENTITY_IRI_BYTES: usize = 2048;
const MAX_LDH_CREATED_AT_BYTES: usize = 64;

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl Serialize for VariantArticleIdentityObservation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let is_ldh = self.source == "clingen_ldh";
        let mut state = serializer.serialize_struct(
            "VariantArticleIdentityObservation",
            if is_ldh { 7 } else { 10 },
        )?;
        state.serialize_field("source", self.source)?;
        state.serialize_field("section", &self.section)?;
        state.serialize_field("locator", &self.locator)?;
        state.serialize_field("linked_gene", &self.linked_gene)?;
        state.serialize_field("observed_alias", &self.observed_alias)?;
        if !is_ldh {
            state.serialize_field("gene_annotation_id", &self.gene_annotation_id)?;
            state.serialize_field("allele_annotation_id", &self.allele_annotation_id)?;
            state.serialize_field("provider_relation", &self.provider_relation)?;
        }
        state.serialize_field("provider_linkage", &self.provider_linkage)?;
        state.serialize_field("canonical_content_hash", &self.canonical_content_hash)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "kind")]
pub(crate) enum ProviderLinkage {
    #[serde(rename = "pubtator_corresponding_gene")]
    Pubtator {
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
    },
    #[serde(rename = "clingen_ldh_annotation")]
    Ldh {
        annotation_uuid: String,
        caid: String,
        gene_id: Option<u64>,
        pmcid: String,
        selector_type: String,
        selector_value: String,
        entity_iri: String,
        created_at: Option<String>,
        provenance: LdhProvenance,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct LdhProvenance {
    source: &'static str,
    request_template_version: &'static str,
    response_subset_version: &'static str,
    canonical_response_subset_sha256: String,
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

fn canonical_annotation(annotation: &PubTatorAnnotation) -> Option<String> {
    if let Some((name, id, annotation_id)) = typed_gene(annotation) {
        return Some(
            serde_json::json!({
                "id": annotation_id,
                "type": "Gene",
                "name": name,
                "gene_id": id,
            })
            .to_string(),
        );
    }
    if let Some((hgvs, gene_id, tokens, annotation_id)) = typed_variant(annotation) {
        return Some(
            serde_json::json!({
                "id": annotation_id,
                "type": "Variant",
                "hgvs": hgvs,
                "gene_id": gene_id,
                "identifier_tokens": tokens,
            })
            .to_string(),
        );
    }
    linkage_bounds_exceeded(annotation).then(|| "invalid_typed_linkage".to_string())
}

pub(crate) fn canonical_response_subset(response: &PubTatorExportResponse) -> String {
    canonical_subset_hash(
        response
            .documents
            .iter()
            .map(|document| {
                let mut annotations = document
                    .passages
                    .iter()
                    .flat_map(|passage| passage.annotations.iter().filter_map(canonical_annotation))
                    .collect::<Vec<_>>();
                annotations.sort_unstable();
                annotations.dedup();
                serde_json::json!({
                    "id": document.id,
                    "pmid": document.pmid,
                    "annotations": annotations,
                })
                .to_string()
            })
            .collect(),
    )
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
    fn coordinate(value: &str) -> Option<&str> {
        let start = value.find(|character: char| character.is_ascii_digit())?;
        let end = value[start..]
            .find(|character: char| !character.is_ascii_digit())
            .map_or(value.len(), |offset| start + offset);
        let end = value[end..]
            .strip_prefix(['+', '-'])
            .and_then(|suffix| {
                suffix
                    .find(|character: char| !character.is_ascii_digit())
                    .map(|offset| end + 1 + offset)
            })
            .unwrap_or(end);
        (start > 0 && start < end).then(|| &value[..end])
    }

    coordinate(left)
        .zip(coordinate(right))
        .is_some_and(|(left, right)| left == right)
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

fn linkage_bounds_exceeded(annotation: &PubTatorAnnotation) -> bool {
    let Some(infons) = annotation.infons.as_ref() else {
        return false;
    };
    if !matches!(infons.kind.as_deref(), Some("Gene" | "Variant")) {
        return false;
    }
    annotation
        .id
        .as_deref()
        .is_some_and(|id| !bounded(id, MAX_ANNOTATION_ID_BYTES))
        || infons
            .hgvs
            .as_deref()
            .is_some_and(|hgvs| !bounded(hgvs, MAX_HGVS_BYTES))
        || infons
            .identifier
            .as_deref()
            .is_some_and(|identifier| identifier_tokens(identifier).is_none())
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
        && !gene_ids.is_empty()
        && gene_ids.iter().all(|id| *id == gene_id)
        && tokens
            .iter()
            .filter_map(|token| token.strip_prefix("CorrespondingGene:"))
            .all(|id| {
                !id.is_empty()
                    && id.bytes().all(|byte| byte.is_ascii_digit())
                    && id == gene_id.to_string()
            })
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
    let requested_aliases = requested
        .protein_change
        .iter()
        .chain(requested.coding_change.iter())
        .map(|alias| normalized(alias))
        .collect::<Vec<_>>();
    let requested_raw_aliases = requested
        .protein_change
        .iter()
        .chain(requested.coding_change.iter())
        .map(String::as_str)
        .collect::<Vec<_>>();
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
        let Some(gene) = requested_gene.as_deref() else {
            continue;
        };
        if requested_raw_aliases.is_empty() {
            continue;
        }
        for passage in &document.passages {
            semantic_anomaly |= passage.annotations.iter().any(linkage_bounds_exceeded);
        }
        for (gene_passage_index, gene_passage) in document.passages.iter().enumerate() {
            for gene_annotation in &gene_passage.annotations {
                let Some((gene_name, gene_id, gene_annotation_id)) = typed_gene(gene_annotation)
                else {
                    continue;
                };
                for (variant_passage_index, variant_passage) in document.passages.iter().enumerate()
                {
                    for variant_annotation in &variant_passage.annotations {
                        let Some((observed_hgvs, variant_gene_id, tokens, variant_annotation_id)) =
                            typed_variant(variant_annotation)
                        else {
                            continue;
                        };
                        if variant_gene_id != gene_id {
                            continue;
                        }
                        let matches_gene = normalized(gene_name) == gene;
                        let matches_allele = requested_aliases.contains(&normalized(observed_hgvs));
                        if !(matches_allele
                            || matches_gene
                                && requested_raw_aliases
                                    .iter()
                                    .any(|requested| same_coordinate(observed_hgvs, requested)))
                        {
                            continue;
                        }
                        let section = variant_passage
                            .infons
                            .as_ref()
                            .and_then(|infons| infons.kind.as_deref())
                            .unwrap_or("unknown")
                            .to_string();
                        let content_hash = hash(&format!(
                            "{}\n{}",
                            gene_passage.text.as_deref().unwrap_or_default(),
                            variant_passage.text.as_deref().unwrap_or_default(),
                        ));
                        let locator = format!(
                            "document:{returned_pmid}:gene-passage:{}:variant-passage:{}",
                            gene_passage_index + 1,
                            variant_passage_index + 1,
                        );
                        let observation = VariantArticleIdentityObservation {
                            source: "pubtator",
                            section: section.clone(),
                            locator: locator.clone(),
                            linked_gene: gene_name.into(),
                            observed_alias: observed_hgvs.into(),
                            gene_annotation_id: gene_annotation_id.into(),
                            allele_annotation_id: variant_annotation_id.into(),
                            provider_relation: None,
                            provider_linkage: Some(ProviderLinkage::Pubtator {
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
                        } else if (matches_allele && !matches_gene)
                            || (matches_gene && !matches_allele)
                        {
                            contradictions.push(observation);
                        }
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

pub(crate) fn verify_ldh_annotation(
    requested: &RequestedVariantIdentity,
    caid: &str,
    aliases: &[String],
    pmcid: &str,
    entity_iri: &str,
    response: &Value,
) -> VariantArticleIdentity {
    let requested_gene = requested.gene.as_deref().map(normalized);
    let mut observations = Vec::new();
    let mut contradictions = Vec::new();
    let valid_shape = response.as_object().is_some_and(|object| {
        object
            .keys()
            .all(|key| key == "annotations" || key == "submittedOn")
            && object.get("annotations").is_some_and(Value::is_array)
    });
    let mut incomplete = !valid_shape;
    if valid_shape {
        for annotation in response["annotations"].as_array().into_iter().flatten() {
            let Some(id) = annotation.get("id").and_then(Value::as_str) else {
                incomplete = true;
                continue;
            };
            let Some(publication) = annotation.get("publicationId").and_then(Value::as_str) else {
                incomplete = true;
                continue;
            };
            let Some(article_pmcid) = annotation
                .pointer("/articleData/articleIDs/PMCID")
                .and_then(Value::as_str)
            else {
                incomplete = true;
                continue;
            };
            let Some(variant_match) = annotation.get("variantMatch").and_then(Value::as_str) else {
                incomplete = true;
                continue;
            };
            let items = annotation.pointer("/body/items").and_then(Value::as_array);
            let Some(items) = items else {
                incomplete = true;
                continue;
            };
            let caid_matches = items.iter().any(|item| {
                item.get("type").and_then(Value::as_str) == Some("TextualBody")
                    && item.get("value").and_then(Value::as_str) == Some(caid)
            });
            let gene_item = items.iter().find(|item| {
                item.get("type").and_then(Value::as_str) == Some("TextualBody")
                    && item.get("value").and_then(Value::as_str) == Some("GeneData")
            });
            let gene_symbols = gene_item
                .and_then(|item| item.get("geneSymbol"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>();
            let gene_matches = gene_symbols
                .iter()
                .any(|symbol| requested_gene.as_deref() == Some(normalized(symbol).as_str()));
            if publication == pmcid
                && article_pmcid == pmcid
                && caid_matches
                && !gene_symbols.is_empty()
                && requested_gene.is_some()
                && !gene_matches
            {
                contradictions.push(VariantArticleIdentityObservation {
                    source: "clingen_ldh",
                    section: "annotation".into(),
                    locator: entity_iri.into(),
                    linked_gene: requested.gene.clone().unwrap_or_default(),
                    observed_alias: caid.into(),
                    gene_annotation_id: String::new(),
                    allele_annotation_id: String::new(),
                    provider_relation: None,
                    provider_linkage: None,
                    canonical_content_hash: hash(&annotation.to_string()),
                });
                continue;
            }
            let Some(target_items) = annotation
                .pointer("/target/items")
                .and_then(Value::as_array)
            else {
                incomplete = true;
                continue;
            };
            // LDH nests selectors one level below the annotation target: each
            // `target.items[]` entry carries the `source` it was read from and a
            // `selector` array of the quotes found there. Only the article's own
            // page counts as an in-text citation; a quote lifted from a
            // supplementary XLSX or PDF attached to the same article is not one.
            let selectors = target_items
                .iter()
                .filter(|item| {
                    item.get("source")
                        .and_then(Value::as_str)
                        .is_some_and(|source| {
                            source == format!("https://www.ncbi.nlm.nih.gov/pmc/articles/{pmcid}")
                                || source
                                    == format!("https://www.ncbi.nlm.nih.gov/pmc/articles/{pmcid}/")
                        })
                })
                .filter_map(|item| item.get("selector").and_then(Value::as_array))
                .flatten()
                .filter(|selector| {
                    matches!(
                        selector.get("type").and_then(Value::as_str),
                        Some("TextQuoteSelector" | "TableTextSelector")
                    ) && selector.get("exact").and_then(Value::as_str) == Some(variant_match)
                })
                .collect::<Vec<_>>();
            if publication != pmcid || article_pmcid != pmcid {
                incomplete = true;
                continue;
            }
            if !caid_matches {
                continue;
            }
            if !gene_matches
                || !aliases.iter().any(|alias| alias == variant_match)
                || selectors.len() != 1
            {
                incomplete = true;
                continue;
            }
            let selector = selectors[0];
            let created_at = annotation.get("created").and_then(Value::as_str);
            let selector_type = selector.get("type").and_then(Value::as_str);
            if !bounded(id, MAX_ANNOTATION_ID_BYTES)
                || !bounded(caid, MAX_LDH_CAID_BYTES)
                || !bounded(pmcid, MAX_LDH_PMCID_BYTES)
                || !bounded(variant_match, MAX_LDH_SELECTOR_VALUE_BYTES)
                || !bounded(entity_iri, MAX_LDH_ENTITY_IRI_BYTES)
                || !selector_type.is_some_and(|value| bounded(value, MAX_LDH_SELECTOR_TYPE_BYTES))
                || created_at.is_some_and(|value| !bounded(value, MAX_LDH_CREATED_AT_BYTES))
            {
                incomplete = true;
                continue;
            }
            let gene_id = gene_item
                .and_then(|item| item.get("geneNCBI"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_u64)
                .next();
            observations.push(VariantArticleIdentityObservation {
                source: "clingen_ldh",
                section: "annotation".into(),
                locator: entity_iri.into(),
                linked_gene: requested.gene.clone().unwrap_or_default(),
                observed_alias: variant_match.into(),
                gene_annotation_id: String::new(),
                allele_annotation_id: String::new(),
                provider_relation: None,
                provider_linkage: Some(ProviderLinkage::Ldh {
                    annotation_uuid: id.into(),
                    caid: caid.into(),
                    gene_id,
                    pmcid: pmcid.into(),
                    selector_type: selector_type.unwrap_or_default().into(),
                    selector_value: variant_match.into(),
                    entity_iri: entity_iri.into(),
                    created_at: created_at.map(str::to_owned),
                    provenance: LdhProvenance {
                        source: "clingen_ldh",
                        request_template_version: "ldh-medium-direct-v1",
                        response_subset_version: "annotation-v1",
                        canonical_response_subset_sha256: hash(&response.to_string()),
                    },
                }),
                canonical_content_hash: hash(&annotation.to_string()),
            });
        }
    }
    let status = status_for(&observations, &contradictions);
    VariantArticleIdentity {
        status,
        basis: if observations.is_empty() {
            "none"
        } else {
            "structured_annotation"
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
    fn numeric_pmid_must_agree_with_returned_document_id() {
        let mut mismatched = response(false);
        mismatched.documents[0].pmid = Some(2);
        let identity = verify_pubtator(&requested(), "1", &mismatched, false);
        assert_eq!(identity.status, "unverified");
        assert!(identity.incomplete);
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
    fn requested_coding_or_protein_alias_can_confirm() {
        let requested = RequestedVariantIdentity {
            gene: Some("BRAF".into()),
            protein_change: Some("p.V600E".into()),
            coding_change: Some("c.1799T>A".into()),
            ..Default::default()
        };
        let mut coding_only = response(false);
        coding_only.documents[0].passages[0].annotations[1]
            .infons
            .as_mut()
            .expect("variant infons")
            .hgvs = Some("c.1799T>A".into());
        assert_eq!(
            verify_pubtator(&requested, "1", &coding_only, false).status,
            "confirmed"
        );
    }

    #[test]
    fn typed_linkage_can_span_document_passages() {
        let mut split = response(false);
        let gene = split.documents[0].passages[0].annotations.remove(0);
        split.documents[0].passages.push(PubTatorPassage {
            infons: Some(PubTatorInfons {
                kind: Some("title".into()),
            }),
            text: Some("gene mention".into()),
            annotations: vec![gene],
        });
        assert_eq!(
            verify_pubtator(&requested(), "1", &split, false).status,
            "confirmed"
        );
    }

    #[test]
    fn inconsistent_typed_gene_ids_cannot_confirm() {
        let mut inconsistent = response(false);
        inconsistent.documents[0].passages[0].annotations[1]
            .infons
            .as_mut()
            .expect("variant infons")
            .gene_ids = Some(vec![673, 1]);
        assert_eq!(
            verify_pubtator(&requested(), "1", &inconsistent, false).status,
            "unverified"
        );

        let infons = inconsistent.documents[0].passages[0].annotations[1]
            .infons
            .as_mut()
            .expect("variant infons");
        infons.gene_ids = Some(vec![673]);
        infons.identifier =
            Some("Variant:p.V600E;CorrespondingGene:673;CorrespondingGene:1".into());
        assert_eq!(
            verify_pubtator(&requested(), "1", &inconsistent, false).status,
            "unverified"
        );
    }

    #[test]
    fn exact_allele_linked_to_another_gene_is_contradictory() {
        let mut other_gene = response(false);
        other_gene.documents[0].passages[0].annotations[0]
            .infons
            .as_mut()
            .expect("gene infons")
            .name = Some("NRAS".into());
        other_gene.documents[0].passages[0].annotations[0]
            .infons
            .as_mut()
            .expect("gene infons")
            .identifier = Some("4893".into());
        other_gene.documents[0].passages[0].annotations[0]
            .infons
            .as_mut()
            .expect("gene infons")
            .normalized_id = Some(4893);
        let variant = other_gene.documents[0].passages[0].annotations[1]
            .infons
            .as_mut()
            .expect("variant infons");
        variant.gene_id = Some(4893);
        variant.gene_ids = Some(vec![4893]);
        variant.identifier = Some("Variant:p.V600E;CorrespondingGene:4893".into());
        assert_eq!(
            verify_pubtator(&requested(), "1", &other_gene, false).status,
            "contradictory"
        );
    }

    #[test]
    fn overlong_linkage_fields_are_incomplete_and_ineligible() {
        let mut overlong = response(false);
        overlong.documents[0].passages[0].annotations[1].id =
            Some("x".repeat(MAX_ANNOTATION_ID_BYTES + 1));
        let identity = verify_pubtator(&requested(), "1", &overlong, false);
        assert_eq!(identity.status, "unverified");
        assert!(identity.incomplete);
    }

    #[test]
    fn intronic_coordinates_are_not_collapsed_into_a_contradiction() {
        let requested =
            RequestedVariantIdentity::from_variant_input("ATM c.1066-6T>G").expect("identity");
        let mut alternate = response(false);
        alternate.documents[0].passages[0].annotations[0]
            .infons
            .as_mut()
            .expect("gene infons")
            .name = Some("ATM".into());
        alternate.documents[0].passages[0].annotations[1]
            .infons
            .as_mut()
            .expect("variant infons")
            .hgvs = Some("c.1066+5G>A".into());
        assert_eq!(
            verify_pubtator(&requested, "1", &alternate, false).status,
            "unverified"
        );
    }

    #[test]
    fn canonical_response_subset_ignores_arbitrary_relation_payloads() {
        let baseline = response(true);
        let mut changed = response(true);
        changed.documents[0].relations[0].infons = Some(serde_json::json!({
            "type": "Association",
            "unbounded": "x".repeat(10_000),
        }));
        assert_eq!(
            canonical_response_subset(&baseline),
            canonical_response_subset(&changed)
        );
    }

    #[test]
    fn empty_identity_anomaly_changes_the_response_digest() {
        let baseline = response(false);
        let mut anomalous = response(false);
        anomalous.documents.push(PubTatorDocument {
            id: Some("2".into()),
            pmid: Some(2),
            pmcid: None,
            date: None,
            journal: None,
            authors: vec![],
            passages: vec![],
            relations: vec![],
        });
        assert_ne!(
            canonical_response_subset(&baseline),
            canonical_response_subset(&anomalous)
        );
    }

    /// A capture minimized from the live ClinGen LDH direct response for
    /// PMC5740532 on 2026-07-27. LDH nests its selectors under
    /// `target.items[].selector[]`; reading them straight off `target.items[]`
    /// matches nothing real, so this shape is the contract, not a convenience.
    fn real_ldh_direct_capture() -> Value {
        serde_json::json!({
            "annotations": [{
                "id": "e5aa8c24-8241-5049-8a25-dd569b8ca139",
                "publicationId": "PMC5740532",
                "articleData": {"articleIDs": {"PMCID": "PMC5740532"}},
                "variantMatch": "rs180177133",
                "created": "2025-01-17T12:00:41Z",
                "body": {"items": [
                    {"type": "TextualBody", "value": "CA151245"},
                    {"type": "TextualBody", "value": "GeneData",
                     "geneSymbol": ["PALB2"], "geneNCBI": [79728]}
                ]},
                "target": {"type": "List", "items": [{
                    "source": "https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5740532",
                    "selector": [{"type": "TextQuoteSelector", "exact": "rs180177133"}]
                }]}
            }]
        })
    }

    fn palb2_requested() -> RequestedVariantIdentity {
        RequestedVariantIdentity::from_variant_input("PALB2 c.3113G>A").expect("identity")
    }

    #[test]
    fn ldh_confirms_an_annotation_in_the_live_response_shape() {
        let identity = verify_ldh_annotation(
            &palb2_requested(),
            "CA151245",
            &["rs180177133".into()],
            "PMC5740532",
            "https://ldh.genome.network/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/PMC5740532/data",
            &real_ldh_direct_capture(),
        );
        assert!(!identity.incomplete);
        let linkage = identity
            .observations
            .iter()
            .find_map(|observation| observation.provider_linkage.as_ref())
            .expect("the live shape must yield one LDH linkage");
        let ProviderLinkage::Ldh {
            caid,
            gene_id,
            pmcid,
            selector_type,
            selector_value,
            ..
        } = linkage
        else {
            panic!("expected an LDH linkage");
        };
        assert_eq!(caid, "CA151245");
        assert_eq!(*gene_id, Some(79728));
        assert_eq!(pmcid, "PMC5740532");
        assert_eq!(selector_type, "TextQuoteSelector");
        assert_eq!(selector_value, "rs180177133");
    }

    #[test]
    fn ldh_ignores_a_selector_quoted_from_a_supplementary_file() {
        // The same live document also annotates variants whose only quotes come
        // from an attached XLSX or PDF. Those are not in-article citations, so
        // they must leave the candidate unverified rather than confirmed.
        let mut response = real_ldh_direct_capture();
        response["annotations"][0]["target"]["items"][0]["source"] = serde_json::json!(
            "https://www.ncbi.nlm.nih.gov/pmc/articles/PMC5740532/bin/jmedgenet-supp007.pdf"
        );
        let identity = verify_ldh_annotation(
            &palb2_requested(),
            "CA151245",
            &["rs180177133".into()],
            "PMC5740532",
            "https://ldh.genome.network/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/PMC5740532/data",
            &response,
        );
        assert_eq!(identity.status, "unverified");
        assert!(identity.incomplete);
        assert!(identity.observations.is_empty());
    }

    #[test]
    fn ldh_explicit_requested_caid_wrong_gene_is_contradictory() {
        let response = serde_json::json!({
            "annotations": [{
                "id": "annotation-1",
                "publicationId": "PMC1",
                "articleData": {"articleIDs": {"PMCID": "PMC1"}},
                "variantMatch": "p.V600E",
                "body": {"items": [
                    {"type": "TextualBody", "value": "CA1"},
                    {"type": "TextualBody", "value": "GeneData", "geneSymbol": ["NRAS"]}
                ]},
                "target": {"items": []}
            }]
        });
        assert_eq!(
            verify_ldh_annotation(
                &requested(),
                "CA1",
                &["p.V600E".into()],
                "PMC1",
                "https://ldh.genome.network/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/PMC1/data",
                &response,
            )
            .status,
            "contradictory"
        );
    }

    #[test]
    fn ldh_unrequested_caid_in_a_multi_caid_annotation_is_ignored() {
        let response = serde_json::json!({
            "annotations": [{
                "id": "annotation-1",
                "publicationId": "PMC1",
                "articleData": {"articleIDs": {"PMCID": "PMC1"}},
                "variantMatch": "p.V600E",
                "body": {"items": [
                    {"type": "TextualBody", "value": "CA2"},
                    {"type": "TextualBody", "value": "GeneData", "geneSymbol": ["BRAF"]}
                ]},
                "target": {"type": "List", "items": [{
                    "source": "https://www.ncbi.nlm.nih.gov/pmc/articles/PMC1",
                    "selector": [{"type": "TextQuoteSelector", "exact": "p.V600E"}]
                }]}
            }]
        });
        let identity = verify_ldh_annotation(
            &requested(),
            "CA1",
            &["p.V600E".into()],
            "PMC1",
            "https://ldh.genome.network/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/PMC1/data",
            &response,
        );
        assert_eq!(identity.status, "unverified");
        assert!(!identity.incomplete);
    }

    #[test]
    fn ldh_wrong_pmcid_is_incomplete_without_a_contradiction() {
        let response = serde_json::json!({
            "annotations": [{
                "id": "annotation-1",
                "publicationId": "PMC2",
                "articleData": {"articleIDs": {"PMCID": "PMC2"}},
                "variantMatch": "p.V600E",
                "body": {"items": [
                    {"type": "TextualBody", "value": "CA1"},
                    {"type": "TextualBody", "value": "GeneData", "geneSymbol": ["BRAF"]}
                ]},
                "target": {"type": "List", "items": [{
                    "source": "https://www.ncbi.nlm.nih.gov/pmc/articles/PMC1",
                    "selector": [{"type": "TextQuoteSelector", "exact": "p.V600E"}]
                }]}
            }]
        });
        let identity = verify_ldh_annotation(
            &requested(),
            "CA1",
            &["p.V600E".into()],
            "PMC1",
            "https://ldh.genome.network/ldh/dss/cg/ns/ldh/set/variants_in_literature/id/PMC1/data",
            &response,
        );
        assert_eq!(identity.status, "unverified");
        assert!(identity.incomplete);
        assert!(identity.contradictions.is_empty());
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
