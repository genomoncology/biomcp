use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashSet;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeOwned;

use crate::entities::disease::PhenotypeDirectSupportStatus;
use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};
use crate::utils::serde::StringOrVec;

const MONARCH_BASE: &str = "https://api-v3.monarchinitiative.org";
const MONARCH_API: &str = "monarch";
const MONARCH_BASE_ENV: &str = "BIOMCP_MONARCH_BASE";
pub(crate) const MONARCH_PHENOTYPE_WINDOW_LIMIT: usize = 50;

pub struct MonarchClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MonarchClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MONARCH_BASE, MONARCH_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        strict_content_type: bool,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::MONARCH,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::MONARCH),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes, strict_content_type)
            .map_err(|error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::MONARCH,
                ))
            })
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
        strict_content_type: bool,
    ) -> Result<T, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            if status.is_server_error() {
                return Err(BioMcpError::SourceUnavailable {
                    source_name: "Monarch Initiative".to_string(),
                    reason: format!(
                        "Monarch returned HTTP {status} for phenotype/disease evidence: {excerpt}"
                    ),
                    suggestion:
                        "Retry later or run the release verify lane again when Monarch is healthy."
                            .to_string(),
                });
            }
            return Err(BioMcpError::Api {
                api: MONARCH_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        let context = crate::error::SourceContext::retry(crate::error::SourceProvider::MONARCH);
        if strict_content_type {
            require_json_content_type(context, content_type, bytes)?;
        } else {
            crate::sources::ensure_json_content_type(context, content_type, bytes)?;
        }

        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: MONARCH_API.to_string(),
            source,
        })
    }

    pub(crate) fn disease_gene_associations_plan(
        disease_id: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let disease_id = normalize_disease_id(disease_id)?;
        let limit = limit.clamp(1, 200);
        Ok(RequestPlan::get("v3/api/association")
            .query("object", disease_id)
            .query("subject_category", "biolink:Gene")
            .query("limit", limit.to_string()))
    }

    fn map_gene_associations(
        resp: MonarchAssociationResponse,
        limit: usize,
    ) -> Vec<MonarchGeneAssociation> {
        let limit = limit.clamp(1, 200);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for item in resp.items {
            let Some(gene) = item
                .subject_label
                .clone()
                .filter(|v| !v.trim().is_empty())
                .or_else(|| {
                    item.subject
                        .as_deref()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(str::to_string)
                })
            else {
                continue;
            };

            let key = gene.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }

            out.push(MonarchGeneAssociation {
                gene,
                relationship: predicate_label(item.predicate.as_deref()),
                source: item
                    .primary_knowledge_source
                    .or(item.provided_by)
                    .filter(|v| !v.trim().is_empty()),
                disease_id: item.object,
                disease_name: item.object_label,
            });

            if out.len() >= limit {
                break;
            }
        }
        out
    }

    pub async fn disease_gene_associations(
        &self,
        disease_id: &str,
        limit: usize,
    ) -> Result<Vec<MonarchGeneAssociation>, BioMcpError> {
        let plan = Self::disease_gene_associations_plan(disease_id, limit)?;
        let resp: MonarchAssociationResponse = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan),
                false,
            )
            .await?;
        let out = Self::map_gene_associations(resp, limit);
        Ok(out)
    }

    pub(crate) fn disease_phenotypes_plan(
        disease_id: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let disease_id = normalize_disease_id(disease_id)?;
        let limit = limit.clamp(1, 200);
        Ok(RequestPlan::get("v3/api/association")
            .query("subject", disease_id)
            .query("object_category", "biolink:PhenotypicFeature")
            .query("limit", limit.to_string()))
    }

    fn map_phenotype_associations(
        resp: MonarchAssociationResponse,
        limit: usize,
    ) -> Vec<MonarchPhenotypeAssociation> {
        let limit = limit.clamp(1, 200);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for item in resp.items {
            let Some(hpo_id) = item
                .object
                .filter(|v| v.to_ascii_uppercase().starts_with("HP:"))
            else {
                continue;
            };

            let key = hpo_id.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }

            out.push(MonarchPhenotypeAssociation {
                hpo_id,
                label: item.object_label,
                relationship: predicate_label(item.predicate.as_deref()),
                frequency_qualifier: item.frequency_qualifier_label,
                onset_qualifier: item.onset_qualifier_label,
                sex_qualifier: item.sex_qualifier_label,
                stage_qualifier: item.stage_qualifier_label,
                qualifiers: item.qualifiers_label.into_vec(),
                source: item
                    .primary_knowledge_source
                    .or(item.provided_by)
                    .filter(|v| !v.trim().is_empty()),
                disease_id: item.subject,
                disease_name: item.subject_label,
            });

            if out.len() >= limit {
                break;
            }
        }
        out
    }

    pub async fn disease_phenotypes(
        &self,
        disease_id: &str,
        limit: usize,
    ) -> Result<Vec<MonarchPhenotypeAssociation>, BioMcpError> {
        let plan = Self::disease_phenotypes_plan(disease_id, limit)?;
        let resp: MonarchAssociationResponse = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan),
                false,
            )
            .await?;
        let out = Self::map_phenotype_associations(resp, limit);
        Ok(out)
    }

    pub(crate) fn disease_models_plan(
        disease_id: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let disease_id = normalize_disease_id(disease_id)?;
        let limit = limit.clamp(1, 200);
        Ok(RequestPlan::get("v3/api/association")
            .query("object", disease_id)
            .query("subject_category", "biolink:Genotype")
            .query("limit", limit.to_string()))
    }

    fn map_model_associations(
        resp: MonarchAssociationResponse,
        limit: usize,
    ) -> Vec<MonarchModelAssociation> {
        let limit = limit.clamp(1, 200);
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for item in resp.items {
            let Some(model) = item
                .subject_label
                .clone()
                .filter(|v| !v.trim().is_empty())
                .or(item.subject.clone())
            else {
                continue;
            };

            let key = model.to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }

            out.push(MonarchModelAssociation {
                model,
                model_id: item
                    .subject
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string),
                organism: item.subject_taxon_label,
                relationship: predicate_label(item.predicate.as_deref()),
                source: item
                    .primary_knowledge_source
                    .or(item.provided_by)
                    .filter(|v| !v.trim().is_empty()),
                evidence_count: item.evidence_count,
            });

            if out.len() >= limit {
                break;
            }
        }
        out
    }

    pub async fn disease_models(
        &self,
        disease_id: &str,
        limit: usize,
    ) -> Result<Vec<MonarchModelAssociation>, BioMcpError> {
        let plan = Self::disease_models_plan(disease_id, limit)?;
        let resp: MonarchAssociationResponse = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan),
                false,
            )
            .await?;
        let out = Self::map_model_associations(resp, limit);
        Ok(out)
    }

    pub(crate) fn phenotype_similarity_search_plan(
        hpo_terms: &[String],
    ) -> Result<RequestPlan, BioMcpError> {
        let normalized = normalize_hpo_terms(hpo_terms)?;
        let termset = normalized.join(",");
        Ok(
            RequestPlan::get(format!("v3/api/semsim/search/{termset}/Human%20Diseases"))
                .query("limit", MONARCH_PHENOTYPE_WINDOW_LIMIT.to_string()),
        )
    }

    fn map_phenotype_matches(rows: Vec<MonarchSemsimRow>) -> MonarchPhenotypeSearchResponse {
        let raw_row_count = rows.len();
        let provider_window_exhausted = raw_row_count == MONARCH_PHENOTYPE_WINDOW_LIMIT;
        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for row in rows {
            let Some(disease_id) = row
                .subject
                .id
                .as_deref()
                .map(str::trim)
                .filter(|v| v.starts_with("MONDO:"))
                .map(str::to_string)
            else {
                continue;
            };
            if !seen.insert(disease_id.clone()) {
                continue;
            }

            let disease_name = row
                .subject
                .name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| disease_id.clone());

            out.push(MonarchPhenotypeMatch {
                disease_id,
                disease_name,
                score: row.score,
            });
        }

        MonarchPhenotypeSearchResponse {
            matches: out,
            raw_row_count,
            provider_window_exhausted,
        }
    }

    pub(crate) async fn phenotype_similarity_search(
        &self,
        hpo_terms: &[String],
    ) -> Result<MonarchPhenotypeSearchResponse, BioMcpError> {
        let plan = Self::phenotype_similarity_search_plan(hpo_terms)?;
        let rows: Vec<MonarchSemsimRow> = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan),
                true,
            )
            .await?;
        let out = Self::map_phenotype_matches(rows);
        Ok(out)
    }

    pub(crate) fn phenotype_direct_support_plan(
        disease_ids: &[String],
        hpo_terms: &[String],
    ) -> Result<RequestPlan, BioMcpError> {
        if disease_ids.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "At least one sliced MONDO disease is required for direct support".into(),
            ));
        }
        let mut plan = RequestPlan::get("v3/api/association");
        let mut seen = HashSet::new();
        for id in disease_ids {
            let id = normalize_disease_id(id)?;
            if !id.starts_with("MONDO:") {
                return Err(BioMcpError::InvalidArgument(format!(
                    "Phenotype support requires a MONDO identifier. Received: {id}"
                )));
            }
            if seen.insert(id.clone()) {
                plan = plan.query("subject", id);
            }
        }
        for id in normalize_hpo_terms(hpo_terms)? {
            plan = plan.query("object", id);
        }
        Ok(plan
            .query("category", "biolink:DiseaseToPhenotypicFeatureAssociation")
            .query("predicate", "biolink:has_phenotype")
            .query("object_category", "biolink:PhenotypicFeature")
            .query("direct", "true")
            .query("limit", "500")
            .query("offset", "0"))
    }

    pub(crate) fn map_direct_support(
        response: MonarchDirectSupportResponse,
        disease_ids: &[String],
        hpo_terms: &[String],
    ) -> Result<MonarchDirectSupportLookup, BioMcpError> {
        let total = match response.total {
            Presence::Missing => None,
            Presence::Value(value) => Some(value),
        };
        let items = match response.items {
            Presence::Missing => None,
            Presence::Value(items) => Some(items),
        };
        let disease_set = disease_ids
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let hpo_set = hpo_terms.iter().map(String::as_str).collect::<HashSet<_>>();
        let mut supported = HashSet::new();
        let mut consistent = true;
        if let Some(items) = items.as_ref() {
            for item in items {
                let subject = item.subject.as_deref().unwrap_or_default();
                let object = item.object.as_deref().unwrap_or_default();
                let valid_filter = disease_set.contains(subject)
                    && hpo_set.contains(object)
                    && item.category.as_deref()
                        == Some("biolink:DiseaseToPhenotypicFeatureAssociation")
                    && item.predicate.as_deref() == Some("biolink:has_phenotype");
                if !valid_filter {
                    consistent = false;
                    continue;
                }
                if item.negated != Some(true) {
                    supported.insert((subject.to_string(), object.to_string()));
                }
            }
        }
        let complete = consistent
            && total.is_some_and(|total| total <= 500)
            && items
                .as_ref()
                .is_some_and(|items| total == Some(items.len()));
        Ok(MonarchDirectSupportLookup {
            supported,
            complete,
        })
    }

    pub(crate) async fn phenotype_direct_support(
        &self,
        disease_ids: &[String],
        hpo_terms: &[String],
    ) -> Result<MonarchDirectSupportLookup, BioMcpError> {
        let plan = Self::phenotype_direct_support_plan(disease_ids, hpo_terms)?;
        let response: MonarchDirectSupportResponse = self
            .get_json(
                request_from_plan(&self.client, self.base.as_ref(), &plan),
                true,
            )
            .await?;
        Self::map_direct_support(response, disease_ids, hpo_terms)
    }
}

fn require_json_content_type(
    context: crate::error::SourceContext,
    content_type: Option<&HeaderValue>,
    body: &[u8],
) -> Result<(), BioMcpError> {
    crate::sources::ensure_json_content_type(context, content_type, body)?;
    let media_type = content_type
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim);
    if media_type.is_some_and(|value| {
        value.eq_ignore_ascii_case("application/json")
            || value.eq_ignore_ascii_case("text/json")
            || value.to_ascii_lowercase().ends_with("+json")
    }) {
        return Ok(());
    }
    Err(BioMcpError::Api {
        api: context.provider().label().into(),
        message: "Provider response did not declare a JSON content type".into(),
    }
    .with_source_context(context))
}

fn normalize_disease_id(value: &str) -> Result<String, BioMcpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Disease ID is required (e.g., MONDO:0007739).".into(),
        ));
    }

    if trimmed.starts_with("MONDO:") || trimmed.starts_with("DOID:") {
        return Ok(trimmed.to_string());
    }

    Err(BioMcpError::InvalidArgument(format!(
        "Monarch requires MONDO/DOID identifiers. Received: {value}"
    )))
}

fn normalize_hpo_terms(values: &[String]) -> Result<Vec<String>, BioMcpError> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    for raw in values {
        let mut term = raw.trim().to_ascii_uppercase();
        if term.is_empty() {
            continue;
        }
        term = term.replace('_', ":");
        if !term.starts_with("HP:") {
            return Err(BioMcpError::InvalidArgument(format!(
                "Invalid HPO term: {raw}. Expected format HP:0001250"
            )));
        }

        let suffix = term.trim_start_matches("HP:");
        if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(format!(
                "Invalid HPO term: {raw}. Expected format HP:0001250"
            )));
        }

        let normalized = format!("HP:{suffix}");
        if seen.insert(normalized.clone()) {
            out.push(normalized);
        }
    }

    if out.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "At least one HPO term is required. Example: HP:0001250 HP:0001263".into(),
        ));
    }

    Ok(out)
}

fn predicate_label(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(|v| v.strip_prefix("biolink:").unwrap_or(v))
        .map(|v| v.replace('_', " "))
}

#[derive(Debug, Clone, Deserialize)]
struct MonarchAssociationResponse {
    // dead-code reason: monarch::total preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    #[serde(default)]
    total: usize,
    #[serde(default)]
    items: Vec<MonarchAssociationItem>,
}

#[derive(Debug, Clone, Deserialize)]
struct MonarchAssociationItem {
    #[serde(default)]
    subject: Option<String>,
    #[serde(default)]
    subject_label: Option<String>,
    #[serde(default)]
    subject_taxon_label: Option<String>,
    #[serde(default)]
    predicate: Option<String>,
    #[serde(default)]
    object: Option<String>,
    #[serde(default)]
    object_label: Option<String>,
    #[serde(default)]
    primary_knowledge_source: Option<String>,
    #[serde(default)]
    provided_by: Option<String>,
    #[serde(default)]
    evidence_count: Option<u32>,
    #[serde(default)]
    qualifiers_label: StringOrVec,
    #[serde(default)]
    frequency_qualifier_label: Option<String>,
    #[serde(default)]
    onset_qualifier_label: Option<String>,
    #[serde(default)]
    sex_qualifier_label: Option<String>,
    #[serde(default)]
    stage_qualifier_label: Option<String>,
}

#[derive(Debug, Clone, Default)]
enum Presence<T> {
    #[default]
    Missing,
    Value(T),
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Presence<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self::Value)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MonarchDirectSupportResponse {
    #[serde(default)]
    total: Presence<usize>,
    #[serde(default)]
    items: Presence<Vec<MonarchDirectSupportItem>>,
}

#[derive(Debug, Clone, Deserialize)]
struct MonarchDirectSupportItem {
    subject: Option<String>,
    object: Option<String>,
    category: Option<String>,
    predicate: Option<String>,
    #[serde(default)]
    negated: Option<bool>,
}

#[derive(Debug, Clone)]
pub(crate) struct MonarchDirectSupportLookup {
    supported: HashSet<(String, String)>,
    complete: bool,
}

impl MonarchDirectSupportLookup {
    pub(crate) fn status(&self, disease_id: &str, hpo_id: &str) -> PhenotypeDirectSupportStatus {
        if self
            .supported
            .contains(&(disease_id.to_string(), hpo_id.to_string()))
        {
            PhenotypeDirectSupportStatus::Supported
        } else if self.complete {
            PhenotypeDirectSupportStatus::NotSupported
        } else {
            PhenotypeDirectSupportStatus::Indeterminate
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct MonarchSemsimRow {
    subject: MonarchSemsimSubject,
    score: f64,
}

#[derive(Debug, Clone, Deserialize)]
struct MonarchSemsimSubject {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonarchGeneAssociation {
    pub gene: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease_name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonarchPhenotypeAssociation {
    pub hpo_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onset_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex_qualifier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stage_qualifier: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub qualifiers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disease_name: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonarchModelAssociation {
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organism: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_count: Option<u32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonarchPhenotypeMatch {
    pub disease_id: String,
    pub disease_name: String,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct MonarchPhenotypeSearchResponse {
    pub(crate) matches: Vec<MonarchPhenotypeMatch>,
    pub(crate) raw_row_count: usize,
    pub(crate) provider_window_exhausted: bool,
}

#[cfg(test)]
mod tests;
