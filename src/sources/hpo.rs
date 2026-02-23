use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use futures::future::join_all;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const HPO_BASE: &str = "https://ontology.jax.org/api/hp";
const HPO_API: &str = "hpo";
const HPO_BASE_ENV: &str = "BIOMCP_HPO_BASE";

pub struct HpoClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl HpoClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(HPO_BASE, HPO_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, HPO_API).await?;
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(BioMcpError::NotFound {
                entity: "hpo".into(),
                id: "term".into(),
                suggestion: "Use an HPO ID like HP:0001653".into(),
            });
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: HPO_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: HPO_API.to_string(),
            source,
        })
    }

    pub async fn term(&self, hpo_id: &str) -> Result<HpoTerm, BioMcpError> {
        let hpo_id = normalize_hpo_id(hpo_id).ok_or_else(|| {
            BioMcpError::InvalidArgument("HPO term ID is required (e.g., HP:0001653)".into())
        })?;
        let url = self.endpoint(&format!("terms/{hpo_id}"));
        self.get_json(self.client.get(&url)).await
    }

    pub async fn resolve_terms(
        &self,
        ids: &[String],
        max_terms: usize,
    ) -> Result<HashMap<String, String>, BioMcpError> {
        let mut normalized: Vec<String> = ids
            .iter()
            .filter_map(|id| normalize_hpo_id(id))
            .collect::<Vec<_>>();
        normalized.sort();
        normalized.dedup();
        normalized.truncate(max_terms.clamp(1, 20));

        let lookups = normalized
            .iter()
            .map(|id| async move { (id.clone(), self.term(id).await) })
            .collect::<Vec<_>>();

        let mut out: HashMap<String, String> = HashMap::new();
        for (id, result) in join_all(lookups).await {
            match result {
                Ok(term) => {
                    let name = term.name.trim();
                    if !name.is_empty() {
                        out.insert(id, name.to_string());
                    }
                }
                Err(BioMcpError::NotFound { .. }) => {}
                Err(err) => return Err(err),
            }
        }
        Ok(out)
    }

    pub async fn search_term_ids(
        &self,
        query: &str,
        max_terms: usize,
    ) -> Result<Vec<String>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }
        let limit = max_terms.clamp(1, 20);
        let url = self.endpoint("search");
        let response: HpoSearchResponse = self
            .get_json(self.client.get(&url).query(&[("q", query)]))
            .await?;
        let mut out: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for row in response.terms {
            if let Some(id) = normalize_hpo_id(&row.id)
                && seen.insert(id.clone())
            {
                out.push(id);
                if out.len() >= limit {
                    break;
                }
            }
        }
        Ok(out)
    }
}

fn normalize_hpo_id(value: &str) -> Option<String> {
    let mut id = value.trim().to_ascii_uppercase();
    if id.is_empty() {
        return None;
    }
    id = id.replace('_', ":");
    if !id.starts_with("HP:") {
        return None;
    }
    let suffix = id.trim_start_matches("HP:");
    if suffix.is_empty() || !suffix.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!("HP:{suffix}"))
}

#[derive(Debug, Clone, Deserialize)]
pub struct HpoTerm {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct HpoSearchResponse {
    #[serde(default)]
    terms: Vec<HpoSearchTerm>,
}

#[derive(Debug, Clone, Deserialize)]
struct HpoSearchTerm {
    id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn normalize_hpo_id_accepts_standard_forms() {
        assert_eq!(
            normalize_hpo_id("HP:0001653").as_deref(),
            Some("HP:0001653")
        );
        assert_eq!(
            normalize_hpo_id("hp_0001653").as_deref(),
            Some("HP:0001653")
        );
        assert_eq!(normalize_hpo_id(""), None);
        assert_eq!(normalize_hpo_id("MP:0001653"), None);
    }

    #[tokio::test]
    async fn term_fetches_term_name() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/terms/HP:0001653"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "HP:0001653",
                "name": "Aortic root aneurysm"
            })))
            .mount(&server)
            .await;

        let client = HpoClient::new_for_test(server.uri()).expect("client");
        let term = client.term("HP:0001653").await.expect("term");
        assert_eq!(term.id, "HP:0001653");
        assert_eq!(term.name, "Aortic root aneurysm");
    }

    #[tokio::test]
    async fn resolve_terms_dedupes_and_limits() {
        let server = MockServer::start().await;
        for id in ["HP:0001653", "HP:0002097"] {
            Mock::given(method("GET"))
                .and(path(format!("/terms/{id}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "id": id,
                    "name": format!("Term {id}")
                })))
                .mount(&server)
                .await;
        }

        let client = HpoClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .resolve_terms(
                &[
                    "HP:0001653".into(),
                    "hp_0001653".into(),
                    "HP:0002097".into(),
                ],
                20,
            )
            .await
            .expect("resolved");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows.get("HP:0001653").map(String::as_str),
            Some("Term HP:0001653")
        );
    }

    #[tokio::test]
    async fn search_term_ids_maps_search_results() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "seizure"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "terms": [
                    {"id": "HP:0001250", "name": "Seizure"},
                    {"id": "hp_0001263", "name": "Developmental delay"},
                    {"id": "NOT_AN_HPO", "name": "Ignore me"},
                    {"id": "HP:0001250", "name": "Seizure duplicate"}
                ]
            })))
            .mount(&server)
            .await;

        let client = HpoClient::new_for_test(server.uri()).expect("client");
        let ids = client
            .search_term_ids("seizure", 5)
            .await
            .expect("search results");
        assert_eq!(
            ids,
            vec!["HP:0001250".to_string(), "HP:0001263".to_string()]
        );
    }
}
