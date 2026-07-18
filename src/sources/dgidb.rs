use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

const DGIDB_BASE: &str = "https://dgidb.org/api";
const DGIDB_API: &str = "dgidb";
const DGIDB_BASE_ENV: &str = "BIOMCP_DGIDB_BASE";
const DGIDB_MAX_INTERACTIONS: usize = 15;

const DGIDB_GENE_QUERY: &str = r#"
query DgidbGeneDruggability($gene: String!, $first: Int!) {
  genes(names: [$gene], first: $first) {
    nodes {
      name
      geneCategories {
        name
      }
      interactions {
        drug {
          name
          approved
        }
        interactionScore
        interactionTypes {
          type
        }
        sources {
          sourceDbName
        }
      }
    }
  }
}
"#;

pub struct DgidbClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl DgidbClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(DGIDB_BASE, DGIDB_BASE_ENV),
        })
    }

    pub(crate) fn gene_interactions_plan(gene_name: &str) -> Result<RequestPlan, BioMcpError> {
        let gene_name = normalize_gene_symbol(gene_name)?;
        let mut plan = RequestPlan::post("graphql");
        plan.body = RequestBody::Json(serde_json::json!({
            "query": DGIDB_GENE_QUERY,
            "variables": {
                "gene": gene_name,
                "first": 1,
            },
        }));
        Ok(plan)
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::DGIDB),
            status,
            content_type,
            bytes,
            true,
        )
    }

    async fn post_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::DGIDB,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::DGIDB),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::DGIDB,
            ))
        })
    }

    pub async fn gene_interactions(
        &self,
        gene_name: &str,
    ) -> Result<GeneDruggability, BioMcpError> {
        let plan = Self::gene_interactions_plan(gene_name)?;
        let resp: GraphQlResponse<DgidbGeneData> = self
            .post_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Self::druggability_from_response(resp)
    }

    fn druggability_from_response(
        resp: GraphQlResponse<DgidbGeneData>,
    ) -> Result<GeneDruggability, BioMcpError> {
        if let Some(errors) = resp.errors {
            let message = errors
                .into_iter()
                .filter_map(|row| clean_optional(row.message))
                .collect::<Vec<_>>()
                .join("; ");
            if !message.is_empty() {
                return Err(BioMcpError::Api {
                    api: DGIDB_API.to_string(),
                    message,
                });
            }
        }

        let Some(node) = resp
            .data
            .and_then(|row| row.genes)
            .and_then(|conn| conn.nodes.into_iter().next())
        else {
            return Ok(GeneDruggability::default());
        };

        let mut categories = BTreeSet::new();
        for category in node.gene_categories {
            let Some(name) = clean_optional(category.name) else {
                continue;
            };
            categories.insert(normalize_label(&name));
        }

        let mut by_drug: HashMap<String, InteractionAccumulator> = HashMap::new();
        for row in node.interactions {
            let Some(drug_name) = row
                .drug
                .as_ref()
                .and_then(|drug| clean_optional(drug.name.clone()))
            else {
                continue;
            };
            let key = drug_name.to_ascii_lowercase();
            let entry = by_drug
                .entry(key)
                .or_insert_with(|| InteractionAccumulator {
                    drug: drug_name.clone(),
                    approved: row.drug.as_ref().and_then(|drug| drug.approved),
                    score: row.interaction_score,
                    ..InteractionAccumulator::default()
                });

            if entry.drug.trim().is_empty() {
                entry.drug = drug_name;
            }

            if let Some(score) = row.interaction_score
                && entry.score.is_none_or(|current| score > current)
            {
                entry.score = Some(score);
            }

            if let Some(approved) = row.drug.as_ref().and_then(|drug| drug.approved) {
                entry.approved = match entry.approved {
                    Some(true) => Some(true),
                    Some(false) => Some(approved),
                    None => Some(approved),
                };
            }

            for kind in row.interaction_types {
                let Some(kind) = clean_optional(kind.kind) else {
                    continue;
                };
                entry
                    .interaction_types
                    .insert(kind.trim().to_ascii_lowercase());
            }

            for source in row.sources {
                let Some(name) = clean_optional(source.source_db_name) else {
                    continue;
                };
                entry.sources.insert(name);
            }
        }

        let mut interactions = by_drug
            .into_values()
            .map(|acc| DrugInteraction {
                drug: acc.drug,
                interaction_types: acc.interaction_types.into_iter().collect(),
                score: acc.score,
                approved: acc.approved,
                source_count: acc.sources.len(),
            })
            .collect::<Vec<_>>();

        interactions.sort_by(|a, b| {
            match (a.score, b.score) {
                (Some(a_score), Some(b_score)) => {
                    b_score.partial_cmp(&a_score).unwrap_or(Ordering::Equal)
                }
                (None, Some(_)) => Ordering::Greater,
                (Some(_), None) => Ordering::Less,
                (None, None) => Ordering::Equal,
            }
            .then_with(|| a.drug.cmp(&b.drug))
        });
        interactions.truncate(DGIDB_MAX_INTERACTIONS);

        Ok(GeneDruggability {
            categories: categories.into_iter().collect(),
            interactions,
            tractability: Vec::new(),
            safety_liabilities: Vec::new(),
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeneDruggability {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interactions: Vec<DrugInteraction>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tractability: Vec<GeneTractabilityModality>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub safety_liabilities: Vec<GeneSafetyLiability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneTractabilityModality {
    pub modality: String,
    pub tractable: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_labels: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneSafetyLiability {
    pub event: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datasource: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_direction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub biosample: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugInteraction {
    pub drug: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub interaction_types: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    pub source_count: usize,
}

#[derive(Debug, Default)]
struct InteractionAccumulator {
    drug: String,
    interaction_types: BTreeSet<String>,
    score: Option<f64>,
    approved: Option<bool>,
    sources: BTreeSet<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Clone, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbGeneData {
    genes: Option<DgidbGeneConnection>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbGeneConnection {
    #[serde(default)]
    nodes: Vec<DgidbGeneNode>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbGeneNode {
    #[serde(default, rename = "geneCategories")]
    gene_categories: Vec<DgidbCategoryRow>,
    #[serde(default)]
    interactions: Vec<DgidbInteractionRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbCategoryRow {
    name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbInteractionRow {
    drug: Option<DgidbDrugRow>,
    #[serde(rename = "interactionScore")]
    interaction_score: Option<f64>,
    #[serde(default, rename = "interactionTypes")]
    interaction_types: Vec<DgidbInteractionTypeRow>,
    #[serde(default)]
    sources: Vec<DgidbSourceRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbDrugRow {
    name: Option<String>,
    approved: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbInteractionTypeRow {
    #[serde(rename = "type")]
    kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct DgidbSourceRow {
    #[serde(rename = "sourceDbName")]
    source_db_name: Option<String>,
}

fn normalize_gene_symbol(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene symbol is required for DGIdb".into(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(&normalized) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {value}"
        )));
    }
    Ok(normalized)
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn normalize_label(value: &str) -> String {
    let value = value.trim().replace('_', " ");
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            let first = chars.next().unwrap_or_default();
            let rest = chars.as_str().to_ascii_lowercase();
            format!("{}{}", first.to_ascii_uppercase(), rest)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests;
