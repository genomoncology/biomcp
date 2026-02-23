use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

use crate::error::BioMcpError;

const MYDISEASE_BASE: &str = "https://mydisease.info/v1";
const MYDISEASE_API: &str = "mydisease.info";
const MYDISEASE_BASE_ENV: &str = "BIOMCP_MYDISEASE_BASE";

const MYDISEASE_SEARCH_FIELDS: &str = "_id,mondo.name,mondo.synonym,disease_ontology.name,disease_ontology.synonyms,hpo.inheritance.hpo_id,hpo.inheritance.hpo_name,hpo.phenotype_related_to_disease.hpo_id,hpo.clinical_course.hpo_name";
const MYDISEASE_GET_FIELDS: &str = "_id,mondo.name,mondo.definition,mondo.parents,mondo.synonym,mondo.xrefs,disease_ontology.name,disease_ontology.doid,disease_ontology.def,disease_ontology.parents,disease_ontology.synonyms,disease_ontology.xrefs,umls.mesh,umls.nci,umls.snomed,umls.icd10am,disgenet.genes_related_to_disease,hpo.phenotype_related_to_disease.hpo_id,hpo.phenotype_related_to_disease.evidence,hpo.phenotype_related_to_disease.hp_freq,hpo.inheritance.hpo_id";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

fn de_vec_or_single<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<OneOrMany<T>>::deserialize(deserializer)?;
    Ok(match value {
        Some(OneOrMany::One(v)) => vec![v],
        Some(OneOrMany::Many(v)) => v,
        None => Vec::new(),
    })
}

#[derive(Clone)]
pub struct MyDiseaseClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MyDiseaseClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MYDISEASE_BASE, MYDISEASE_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, MYDISEASE_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: MYDISEASE_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: MYDISEASE_API.to_string(),
            source,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn query(
        &self,
        q: &str,
        size: usize,
        offset: usize,
        source: Option<&str>,
        inheritance: Option<&str>,
        phenotype: Option<&str>,
        onset: Option<&str>,
    ) -> Result<MyDiseaseQueryResponse, BioMcpError> {
        let q = q.trim();
        if q.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required. Example: biomcp search disease -q melanoma".into(),
            ));
        }
        if q.len() > 512 {
            return Err(BioMcpError::InvalidArgument("Query is too long.".into()));
        }
        crate::sources::validate_biothings_result_window("MyDisease search", size, offset)?;

        let url = self.endpoint("query");
        let size = size.to_string();
        let from = offset.to_string();
        let escaped = crate::utils::query::escape_lucene_value(q);
        let mut scoped_query = if q.contains(':') && !q.chars().any(|c| c.is_whitespace()) {
            format!("(_id:\"{escaped}\" OR disease_ontology.doid:\"{escaped}\")")
        } else {
            // Keep the legacy name search semantics (tokenized by backend) to avoid
            // over-constraining common disease names like "lung cancer".
            format!("(disease_ontology.name:{escaped} OR mondo.name:{escaped})")
        };
        if let Some(source) = source.map(str::trim).filter(|v| !v.is_empty()) {
            let source_clause = match source.to_ascii_lowercase().as_str() {
                "mondo" => "(mondo.parents:* OR mondo.xrefs:*)",
                "doid" => "(disease_ontology.doid:* OR mondo.xrefs.doid:*)",
                "mesh" => "(disease_ontology.xrefs.mesh:* OR mondo.xrefs.mesh:* OR umls.mesh:*)",
                other => {
                    return Err(BioMcpError::InvalidArgument(format!(
                        "Unknown --source '{other}'. Expected one of: mondo, doid, mesh"
                    )));
                }
            };
            scoped_query = format!("{scoped_query} AND {source_clause}");
        }
        if let Some(inheritance) = inheritance.map(str::trim).filter(|v| !v.is_empty()) {
            let escaped = crate::utils::query::escape_lucene_value(inheritance);
            scoped_query = format!(
                "{scoped_query} AND (hpo.inheritance.hpo_name:*{escaped}* OR hpo.inheritance.hpo_id:*{escaped}*)"
            );
        }
        if let Some(phenotype) = phenotype.map(str::trim).filter(|v| !v.is_empty()) {
            let escaped = crate::utils::query::escape_lucene_value(phenotype);
            scoped_query =
                format!("{scoped_query} AND hpo.phenotype_related_to_disease.hpo_id:*{escaped}*");
        }
        if let Some(onset) = onset.map(str::trim).filter(|v| !v.is_empty()) {
            let escaped = crate::utils::query::escape_lucene_value(onset);
            scoped_query = format!("{scoped_query} AND hpo.clinical_course.hpo_name:*{escaped}*");
        }
        self.get_json(self.client.get(&url).query(&[
            ("q", scoped_query.as_str()),
            ("size", size.as_str()),
            ("from", from.as_str()),
            ("fields", MYDISEASE_SEARCH_FIELDS),
        ]))
        .await
    }

    pub async fn get(&self, id: &str) -> Result<MyDiseaseHit, BioMcpError> {
        let id = id.trim();
        if id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Disease ID is required. Example: biomcp get disease MONDO:0005105".into(),
            ));
        }
        if id.len() > 128 {
            return Err(BioMcpError::InvalidArgument(
                "Disease ID is too long.".into(),
            ));
        }

        let url = self.endpoint(&format!("disease/{id}"));
        let resp = self
            .client
            .get(&url)
            .query(&[("fields", MYDISEASE_GET_FIELDS)])
            .send()
            .await?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(BioMcpError::NotFound {
                entity: "disease".into(),
                id: id.into(),
                suggestion: format!("Try searching: biomcp search disease -q \"{id}\""),
            });
        }

        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, MYDISEASE_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: MYDISEASE_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: MYDISEASE_API.to_string(),
            source,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyDiseaseQueryResponse {
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyDiseaseHit>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MyDiseaseHit {
    #[serde(rename = "_id")]
    pub id: String,
    #[serde(default)]
    pub mondo: Option<serde_json::Value>,
    #[serde(default, rename = "disease_ontology")]
    pub disease_ontology: Option<serde_json::Value>,
    #[serde(default)]
    pub umls: Option<serde_json::Value>,
    #[serde(default)]
    pub disgenet: Option<serde_json::Value>,
    #[serde(default)]
    pub hpo: Option<MyDiseaseHpo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseaseHpo {
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub phenotype_related_to_disease: Vec<MyDiseasePhenotypeRelatedToDisease>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub inheritance: Vec<MyDiseaseInheritance>,
    #[serde(default, deserialize_with = "de_vec_or_single")]
    pub clinical_course: Vec<MyDiseaseClinicalCourse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseasePhenotypeRelatedToDisease {
    pub hpo_id: Option<String>,
    pub evidence: Option<String>,
    #[serde(rename = "hp_freq")]
    pub hp_freq: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseaseInheritance {
    pub hpo_id: Option<String>,
    pub hpo_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyDiseaseClinicalCourse {
    pub hpo_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn query_sets_fields_and_size() {
        let server = MockServer::start().await;
        let client = MyDiseaseClient::new_for_test(format!("{}/v1", server.uri())).unwrap();

        let body = r#"{
          "took": 1,
          "total": 1,
          "hits": [{"_id": "MONDO:0005105", "disease_ontology": {"name": "melanoma"}}]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/query"))
            .and(query_param(
                "q",
                "(disease_ontology.name:melanoma OR mondo.name:melanoma)",
            ))
            .and(query_param("size", "10"))
            .and(query_param("from", "0"))
            .and(query_param("fields", MYDISEASE_SEARCH_FIELDS))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client
            .query("melanoma", 10, 0, None, None, None, None)
            .await
            .unwrap();
        assert_eq!(resp.hits.len(), 1);
        assert_eq!(resp.hits[0].id, "MONDO:0005105");
    }

    #[tokio::test]
    async fn get_sets_fields_and_path() {
        let server = MockServer::start().await;
        let client = MyDiseaseClient::new_for_test(format!("{}/v1", server.uri())).unwrap();

        let body = r#"{
          "_id": "MONDO:0005105",
          "disease_ontology": {"name": "melanoma"},
          "mondo": {"definition": "example"}
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/disease/MONDO:0005105"))
            .and(query_param("fields", MYDISEASE_GET_FIELDS))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client.get("MONDO:0005105").await.unwrap();
        assert_eq!(resp.id, "MONDO:0005105");
    }

    #[test]
    fn hpo_fields_deserialize_from_hit() {
        let hit: MyDiseaseHit = serde_json::from_value(serde_json::json!({
            "_id": "MONDO:0017309",
            "hpo": {
                "phenotype_related_to_disease": [
                    {"hpo_id": "HP:0001653", "evidence": "TAS", "hp_freq": "HP:0040280"}
                ],
                "inheritance": {"hpo_id": "HP:0000006"}
            }
        }))
        .expect("hpo payload should deserialize");

        let hpo = hit.hpo.expect("hpo field should exist");
        assert_eq!(hpo.phenotype_related_to_disease.len(), 1);
        assert_eq!(
            hpo.phenotype_related_to_disease[0].hpo_id.as_deref(),
            Some("HP:0001653")
        );
        assert_eq!(hpo.inheritance.len(), 1);
        assert_eq!(hpo.inheritance[0].hpo_id.as_deref(), Some("HP:0000006"));
    }

    #[tokio::test]
    async fn query_rejects_offset_at_biothings_window() {
        let client = MyDiseaseClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client
            .query("melanoma", 5, 10_000, None, None, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--offset must be less than 10000"));
    }

    #[tokio::test]
    async fn query_rejects_offset_limit_window_overflow() {
        let client = MyDiseaseClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client
            .query("melanoma", 40, 9_980, None, None, None, None)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(
            err.to_string()
                .contains("--offset + --limit must be <= 10000")
        );
    }
}
