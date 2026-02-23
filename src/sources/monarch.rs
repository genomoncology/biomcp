use std::borrow::Cow;
use std::collections::HashSet;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::utils::serde::StringOrVec;

const MONARCH_BASE: &str = "https://api-v3.monarchinitiative.org";
const MONARCH_API: &str = "monarch";
const MONARCH_BASE_ENV: &str = "BIOMCP_MONARCH_BASE";

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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MONARCH_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: MONARCH_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(MONARCH_API, content_type.as_ref(), &bytes)?;

        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: MONARCH_API.to_string(),
            source,
        })
    }

    pub async fn disease_gene_associations(
        &self,
        disease_id: &str,
        limit: usize,
    ) -> Result<Vec<MonarchGeneAssociation>, BioMcpError> {
        let disease_id = normalize_disease_id(disease_id)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("v3/api/association");
        let req = self.client.get(&url).query(&[
            ("object", disease_id.as_str()),
            ("subject_category", "biolink:Gene"),
            ("limit", &limit.to_string()),
        ]);

        let resp: MonarchAssociationResponse = self.get_json(req).await?;
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
        Ok(out)
    }

    pub async fn disease_phenotypes(
        &self,
        disease_id: &str,
        limit: usize,
    ) -> Result<Vec<MonarchPhenotypeAssociation>, BioMcpError> {
        let disease_id = normalize_disease_id(disease_id)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("v3/api/association");
        let req = self.client.get(&url).query(&[
            ("subject", disease_id.as_str()),
            ("object_category", "biolink:PhenotypicFeature"),
            ("limit", &limit.to_string()),
        ]);

        let resp: MonarchAssociationResponse = self.get_json(req).await?;
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
        Ok(out)
    }

    pub async fn disease_models(
        &self,
        disease_id: &str,
        limit: usize,
    ) -> Result<Vec<MonarchModelAssociation>, BioMcpError> {
        let disease_id = normalize_disease_id(disease_id)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("v3/api/association");
        let req = self.client.get(&url).query(&[
            ("object", disease_id.as_str()),
            ("subject_category", "biolink:Genotype"),
            ("limit", &limit.to_string()),
        ]);

        let resp: MonarchAssociationResponse = self.get_json(req).await?;
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
        Ok(out)
    }

    pub async fn phenotype_similarity_search(
        &self,
        hpo_terms: &[String],
        limit: usize,
    ) -> Result<Vec<MonarchPhenotypeMatch>, BioMcpError> {
        let normalized = normalize_hpo_terms(hpo_terms)?;
        let limit = limit.clamp(1, 50);
        let termset = normalized.join(",");
        let url = self.endpoint(&format!("v3/api/semsim/search/{termset}/Human%20Diseases"));

        let req = self
            .client
            .get(&url)
            .query(&[("limit", &limit.to_string())]);

        let rows: Vec<MonarchSemsimRow> = self.get_json(req).await?;
        let mut out = Vec::new();
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

            if out.len() >= limit {
                break;
            }
        }

        Ok(out)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, path_regex, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn disease_gene_associations_maps_rows() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/api/association"))
            .and(query_param("object", "MONDO:0007739"))
            .and(query_param("subject_category", "biolink:Gene"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 1,
                "items": [
                    {
                        "subject": "HGNC:4851",
                        "subject_label": "HTT",
                        "predicate": "biolink:gene_associated_with_condition",
                        "primary_knowledge_source": "infores:orphanet",
                        "object": "MONDO:0016621",
                        "object_label": "juvenile Huntington disease"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MonarchClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .disease_gene_associations("MONDO:0007739", 5)
            .await
            .expect("rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].gene, "HTT");
        assert_eq!(
            rows[0].relationship.as_deref(),
            Some("gene associated with condition")
        );
    }

    #[tokio::test]
    async fn disease_models_maps_genotype_rows() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/api/association"))
            .and(query_param("object", "MONDO:0007739"))
            .and(query_param("subject_category", "biolink:Genotype"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "total": 1,
                "items": [
                    {
                        "subject": "MGI:3698752",
                        "subject_label": "Htt tm1.1",
                        "subject_taxon_label": "Mus musculus",
                        "predicate": "biolink:model_of",
                        "provided_by": "alliance_disease_edges"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = MonarchClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .disease_models("MONDO:0007739", 5)
            .await
            .expect("rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].organism.as_deref(), Some("Mus musculus"));
    }

    #[tokio::test]
    async fn phenotype_similarity_search_maps_scores() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex(r"/v3/api/semsim/search/.+/Human(%20| )Diseases"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "subject": {
                        "id": "MONDO:0010450",
                        "name": "intellectual disability, X-linked 89"
                    },
                    "score": 13.302
                }
            ])))
            .mount(&server)
            .await;

        let client = MonarchClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .phenotype_similarity_search(&["HP:0001250".into(), "HP:0001263".into()], 5)
            .await
            .expect("rows");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].disease_id, "MONDO:0010450");
        assert!(rows[0].score > 0.0);
    }
}
