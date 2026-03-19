use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;

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

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    async fn post_json<T: DeserializeOwned, B: Serialize>(
        &self,
        req: reqwest_middleware::RequestBuilder,
        body: &B,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req.json(body))
            .send()
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, DGIDB_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: DGIDB_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(DGIDB_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: DGIDB_API.to_string(),
            source,
        })
    }

    pub async fn gene_interactions(
        &self,
        gene_name: &str,
    ) -> Result<GeneDruggability, BioMcpError> {
        let gene_name = normalize_gene_symbol(gene_name)?;
        let url = self.endpoint("graphql");
        let body = GraphQlRequest {
            query: DGIDB_GENE_QUERY,
            variables: serde_json::json!({
                "gene": gene_name,
                "first": 1,
            }),
        };
        let resp: GraphQlResponse<DgidbGeneData> =
            self.post_json(self.client.post(&url), &body).await?;

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

#[derive(Debug, Clone, Serialize)]
struct GraphQlRequest<'a> {
    query: &'a str,
    variables: serde_json::Value,
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
mod tests {
    use super::*;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn gene_interactions_aggregates_categories_and_interactions() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains("DgidbGeneDruggability"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {
                    "genes": {
                        "nodes": [{
                            "name": "BRAF",
                            "geneCategories": [
                                {"name": "KINASE"},
                                {"name": "kinase"},
                                {"name": "PROTEIN KINASE"}
                            ],
                            "interactions": [
                                {
                                    "drug": {"name": "DABRAFENIB", "approved": true},
                                    "interactionScore": 0.8,
                                    "interactionTypes": [{"type": "inhibitor"}],
                                    "sources": [{"sourceDbName": "SourceA"}]
                                },
                                {
                                    "drug": {"name": "DABRAFENIB", "approved": false},
                                    "interactionScore": 1.2,
                                    "interactionTypes": [{"type": "antagonist"}, {"type": "inhibitor"}],
                                    "sources": [{"sourceDbName": "SourceB"}]
                                },
                                {
                                    "drug": {"name": "SORAFENIB", "approved": true},
                                    "interactionScore": 0.4,
                                    "interactionTypes": [{"type": "inhibitor"}],
                                    "sources": [{"sourceDbName": "SourceC"}]
                                }
                            ]
                        }]
                    }
                }
            })))
            .mount(&server)
            .await;

        let client = DgidbClient::new_for_test(server.uri()).expect("client");
        let out = client
            .gene_interactions("BRAF")
            .await
            .expect("druggability");

        assert_eq!(out.categories, vec!["Kinase", "Protein Kinase"]);
        assert_eq!(out.interactions.len(), 2);

        let first = &out.interactions[0];
        assert_eq!(first.drug, "DABRAFENIB");
        assert_eq!(first.score, Some(1.2));
        assert_eq!(first.approved, Some(true));
        assert_eq!(first.source_count, 2);
        assert_eq!(first.interaction_types, vec!["antagonist", "inhibitor"]);
    }

    #[tokio::test]
    async fn gene_interactions_returns_graphql_errors() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errors": [
                    {"message": "GraphQL validation failed"}
                ]
            })))
            .mount(&server)
            .await;

        let client = DgidbClient::new_for_test(server.uri()).expect("client");
        let err = client
            .gene_interactions("BRAF")
            .await
            .expect_err("graphql error should propagate");
        assert!(matches!(err, BioMcpError::Api { .. }));
        assert!(err.to_string().contains("GraphQL validation failed"));
    }

    #[tokio::test]
    async fn gene_interactions_rejects_invalid_symbol() {
        let client = DgidbClient::new_for_test("http://127.0.0.1".into()).expect("client");
        let err = client
            .gene_interactions("BRAF!")
            .await
            .expect_err("invalid symbol should fail");
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
    }
}
