use std::borrow::Cow;
use std::collections::HashSet;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const WIKIPATHWAYS_BASE: &str = "https://webservice.wikipathways.org";
const WIKIPATHWAYS_API: &str = "wikipathways";
const WIKIPATHWAYS_BASE_ENV: &str = "BIOMCP_WIKIPATHWAYS_BASE";

pub struct WikiPathwaysClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl WikiPathwaysClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(WIKIPATHWAYS_BASE, WIKIPATHWAYS_BASE_ENV),
        })
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(base: String) -> Result<Self, BioMcpError> {
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
        let bytes = crate::sources::read_limited_body(resp, WIKIPATHWAYS_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: WIKIPATHWAYS_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(WIKIPATHWAYS_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: WIKIPATHWAYS_API.to_string(),
            source,
        })
    }

    pub async fn search_pathways(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<WikiPathwaysHit>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "WikiPathways query is required".into(),
            ));
        }

        let url = self.endpoint("findPathwaysByText");
        let resp: WikiPathwaysSearchResponse = self
            .get_json(self.client.get(&url).query(&[
                ("query", query),
                ("organism", "Homo sapiens"),
                ("format", "json"),
            ]))
            .await?;

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for row in resp.result.unwrap_or_default() {
            let is_human = row.species_is_human();
            let Some(id) = row
                .id
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty() && is_wikipathways_id(value) && is_human)
            else {
                continue;
            };
            let Some(name) = row
                .name
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
            else {
                continue;
            };
            if !seen.insert(id.clone()) {
                continue;
            }
            out.push(WikiPathwaysHit { id, name });
            if out.len() >= limit.clamp(1, 25) {
                break;
            }
        }

        Ok(out)
    }

    pub async fn get_pathway(&self, pw_id: &str) -> Result<WikiPathwaysRecord, BioMcpError> {
        let pw_id = validate_wikipathways_id(pw_id)?;
        let url = self.endpoint("getPathwayInfo");
        let resp: WikiPathwaysGetResponse = self
            .get_json(
                self.client
                    .get(&url)
                    .query(&[("pwId", pw_id.as_str()), ("format", "json")]),
            )
            .await?;
        let row = resp.pathway_info;
        let id = row
            .id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty() && is_wikipathways_id(value))
            .ok_or_else(|| BioMcpError::Api {
                api: WIKIPATHWAYS_API.to_string(),
                message: "WikiPathways detail response missing pathwayInfo.id".to_string(),
            })?;
        let name = row
            .name
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| BioMcpError::Api {
                api: WIKIPATHWAYS_API.to_string(),
                message: "WikiPathways detail response missing pathwayInfo.name".to_string(),
            })?;

        Ok(WikiPathwaysRecord {
            id,
            name,
            species: row
                .species
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        })
    }

    pub async fn pathway_entrez_gene_ids(&self, pw_id: &str) -> Result<Vec<String>, BioMcpError> {
        let pw_id = validate_wikipathways_id(pw_id)?;
        let url = self.endpoint("getXrefList");
        let resp: WikiPathwaysXrefResponse = self
            .get_json(self.client.get(&url).query(&[
                ("pwId", pw_id.as_str()),
                ("code", "L"),
                ("format", "json"),
            ]))
            .await?;

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for xref in resp.xrefs.unwrap_or_default() {
            let xref = xref.trim();
            if xref.is_empty() || !xref.chars().all(|ch| ch.is_ascii_digit()) {
                continue;
            }
            if !seen.insert(xref.to_string()) {
                continue;
            }
            out.push(xref.to_string());
        }
        Ok(out)
    }
}

pub(crate) fn is_wikipathways_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3 && bytes.starts_with(b"WP") && bytes[2..].iter().all(u8::is_ascii_digit)
}

fn validate_wikipathways_id(value: &str) -> Result<String, BioMcpError> {
    let value = value.trim();
    if !is_wikipathways_id(value) {
        return Err(BioMcpError::InvalidArgument(
            "WikiPathways ID must look like WP254. Example: biomcp get pathway WP254".into(),
        ));
    }
    Ok(value.to_string())
}

#[derive(Debug, Clone)]
pub struct WikiPathwaysHit {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct WikiPathwaysRecord {
    pub id: String,
    pub name: String,
    pub species: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysSearchResponse {
    result: Option<Vec<WikiPathwaysSearchEntry>>,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysSearchEntry {
    id: Option<String>,
    name: Option<String>,
    species: Option<String>,
}

impl WikiPathwaysSearchEntry {
    fn species_is_human(&self) -> bool {
        self.species
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| value.eq_ignore_ascii_case("Homo sapiens"))
    }
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysGetResponse {
    #[serde(rename = "pathwayInfo")]
    pathway_info: WikiPathwaysGetEntry,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysGetEntry {
    id: Option<String>,
    name: Option<String>,
    species: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WikiPathwaysXrefResponse {
    xrefs: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn validates_wikipathways_id_shape() {
        assert!(is_wikipathways_id("WP254"));
        assert!(!is_wikipathways_id("wp254"));
        assert!(!is_wikipathways_id("R-HSA-5673001"));
        assert!(!is_wikipathways_id("WP25A"));
    }

    #[tokio::test]
    async fn search_pathways_filters_non_human_invalid_and_duplicate_rows() {
        let server = MockServer::start().await;
        let client = WikiPathwaysClient::new_for_test(server.uri()).unwrap();

        let body = r#"{
          "result": [
            {"id": "WP111", "name": "Alpha", "species": "Homo sapiens"},
            {"id": "WP111", "name": "Alpha duplicate", "species": "Homo sapiens"},
            {"id": "WP222", "name": "Mouse only", "species": "Mus musculus"},
            {"id": "BAD", "name": "Bad", "species": "Homo sapiens"},
            {"id": "WP333", "name": "", "species": "Homo sapiens"},
            {"id": "WP444", "name": "Beta", "species": "Homo sapiens"}
          ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/findPathwaysByText"))
            .and(query_param("query", "apoptosis"))
            .and(query_param("organism", "Homo sapiens"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let hits = client.search_pathways("apoptosis", 10).await.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].id, "WP111");
        assert_eq!(hits[1].id, "WP444");
    }

    #[tokio::test]
    async fn get_pathway_parses_minimal_detail_payload() {
        let server = MockServer::start().await;
        let client = WikiPathwaysClient::new_for_test(server.uri()).unwrap();

        Mock::given(method("GET"))
            .and(path("/getPathwayInfo"))
            .and(query_param("pwId", "WP254"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"pathwayInfo":{"id":"WP254","name":"Apoptosis","species":"Homo sapiens","revision":"140926"}}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&server)
            .await;

        let record = client.get_pathway("WP254").await.unwrap();
        assert_eq!(record.id, "WP254");
        assert_eq!(record.name, "Apoptosis");
        assert_eq!(record.species.as_deref(), Some("Homo sapiens"));
    }

    #[tokio::test]
    async fn pathway_entrez_gene_ids_dedupes_and_filters_non_numeric_rows() {
        let server = MockServer::start().await;
        let client = WikiPathwaysClient::new_for_test(server.uri()).unwrap();

        Mock::given(method("GET"))
            .and(path("/getXrefList"))
            .and(query_param("pwId", "WP254"))
            .and(query_param("code", "L"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"xrefs":["7157","1956","7157","BAD","","672"]}"#,
                "application/json",
            ))
            .expect(1)
            .mount(&server)
            .await;

        let ids = client.pathway_entrez_gene_ids("WP254").await.unwrap();
        assert_eq!(ids, vec!["7157", "1956", "672"]);
    }

    #[tokio::test]
    async fn search_rejects_html_content_type_before_json_parse() {
        let server = MockServer::start().await;
        let client = WikiPathwaysClient::new_for_test(server.uri()).unwrap();

        Mock::given(method("GET"))
            .and(path("/findPathwaysByText"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw("<html><body>error page</body></html>", "text/html"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let err = client.search_pathways("apoptosis", 1).await.unwrap_err();
        assert!(err.to_string().contains("Unexpected HTML response"));
    }

    #[tokio::test]
    async fn get_pathway_rejects_invalid_ids_before_request() {
        let client = WikiPathwaysClient::new_for_test("http://127.0.0.1".into()).unwrap();
        let err = client.get_pathway("not-a-pathway").await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("WP254"));
    }
}
