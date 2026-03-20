use std::borrow::Cow;

use futures::future::join_all;
use serde::Deserialize;

use crate::error::BioMcpError;

const UMLS_BASE: &str = "https://uts-ws.nlm.nih.gov";
const UMLS_API: &str = "umls";
const UMLS_BASE_ENV: &str = "BIOMCP_UMLS_BASE";
const UMLS_API_KEY_ENV: &str = "UMLS_API_KEY";
const UMLS_ATOM_PAGE_SIZE: &str = "200";

pub struct UmlsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: String,
}

impl UmlsClient {
    pub fn new() -> Result<Option<Self>, BioMcpError> {
        let Some(api_key) = std::env::var(UMLS_API_KEY_ENV)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        Ok(Some(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(UMLS_BASE, UMLS_BASE_ENV),
            api_key,
        }))
    }

    #[cfg(test)]
    fn new_for_test(base: String, api_key: &str) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
            api_key: api_key.to_string(),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    pub async fn search(&self, query: &str) -> Result<Vec<UmlsConcept>, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok(Vec::new());
        }

        let resp = crate::sources::apply_cache_mode_with_auth(
            self.client
                .get(self.endpoint("rest/search/current"))
                .query(&[
                    ("string", query),
                    ("pageSize", "5"),
                    ("apiKey", self.api_key.as_str()),
                ]),
            true,
        )
        .send()
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, UMLS_API).await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: UMLS_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }
        crate::sources::ensure_json_content_type(UMLS_API, content_type.as_ref(), &bytes)?;

        let search: UmlsSearchEnvelope =
            serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
                api: UMLS_API.to_string(),
                source,
            })?;

        let tasks = search
            .result
            .results
            .into_iter()
            .filter(|hit| hit.ui != "NONE")
            .take(5)
            .map(|hit| async move {
                let xrefs = self.fetch_atoms(&hit.ui).await?;
                Ok::<_, BioMcpError>(UmlsConcept {
                    cui: hit.ui,
                    name: hit.name,
                    semantic_types: hit.semantic_types,
                    xrefs,
                    uri: hit.uri,
                })
            })
            .collect::<Vec<_>>();

        let mut out = Vec::new();
        for result in join_all(tasks).await {
            out.push(result?);
        }
        Ok(out)
    }

    async fn fetch_atoms(&self, cui: &str) -> Result<Vec<UmlsXref>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode_with_auth(
            self.client
                .get(self.endpoint(&format!("rest/content/current/CUI/{cui}/atoms")))
                .query(&[
                    ("apiKey", self.api_key.as_str()),
                    ("pageSize", UMLS_ATOM_PAGE_SIZE),
                    ("language", "ENG"),
                ]),
            true,
        )
        .send()
        .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, UMLS_API).await?;
        if !status.is_success() {
            return Err(BioMcpError::Api {
                api: UMLS_API.to_string(),
                message: format!("HTTP {status}: {}", crate::sources::body_excerpt(&bytes)),
            });
        }
        crate::sources::ensure_json_content_type(UMLS_API, content_type.as_ref(), &bytes)?;

        let atoms: UmlsAtomsEnvelope =
            serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
                api: UMLS_API.to_string(),
                source,
            })?;

        let mut out = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for atom in atoms.result {
            if !atom.language.eq_ignore_ascii_case("ENG") {
                continue;
            }
            let id = atom
                .code
                .rsplit('/')
                .next()
                .map(str::trim)
                .unwrap_or_default();
            if id.is_empty() {
                continue;
            }
            let key = format!("{}:{id}", atom.root_source.to_ascii_uppercase());
            if seen.insert(key) {
                out.push(UmlsXref {
                    vocab: atom.root_source,
                    id: id.to_string(),
                    label: atom.name,
                });
            }
        }
        Ok(out)
    }
}

#[derive(Debug, Clone)]
pub struct UmlsConcept {
    pub cui: String,
    pub name: String,
    pub semantic_types: Vec<String>,
    pub xrefs: Vec<UmlsXref>,
    #[allow(dead_code)]
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UmlsXref {
    pub vocab: String,
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsSearchEnvelope {
    result: UmlsSearchResult,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsSearchResult {
    #[serde(default)]
    results: Vec<UmlsHit>,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsHit {
    ui: String,
    name: String,
    #[serde(default, rename = "semanticTypes")]
    semantic_types: Vec<String>,
    uri: String,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsAtomsEnvelope {
    #[serde(default)]
    result: Vec<UmlsAtom>,
}

#[derive(Debug, Clone, Deserialize)]
struct UmlsAtom {
    #[serde(default, rename = "rootSource")]
    root_source: String,
    #[serde(default)]
    code: String,
    #[serde(default)]
    language: String,
    #[serde(default)]
    name: String,
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::UmlsClient;

    #[tokio::test]
    async fn search_uses_query_param_auth_and_atoms_lookup() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/rest/search/current"))
            .and(query_param("string", "cystic fibrosis"))
            .and(query_param("pageSize", "5"))
            .and(query_param("apiKey", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": {
                    "results": [
                        {
                            "ui": "C0010674",
                            "name": "Cystic Fibrosis",
                            "uri": "https://example.org/C0010674",
                            "semanticTypes": ["Disease or Syndrome"]
                        }
                    ]
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/rest/content/current/CUI/C0010674/atoms"))
            .and(query_param("apiKey", "test-key"))
            .and(query_param("pageSize", "200"))
            .and(query_param("language", "ENG"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": [
                    {
                        "rootSource": "ICD10CM",
                        "code": "https://example.org/source/ICD10CM/E84",
                        "language": "ENG",
                        "name": "Cystic fibrosis"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = UmlsClient::new_for_test(server.uri(), "test-key").expect("client");
        let rows = client.search("cystic fibrosis").await.expect("search");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cui, "C0010674");
        assert_eq!(rows[0].xrefs[0].vocab, "ICD10CM");
        assert_eq!(rows[0].xrefs[0].id, "E84");
    }
}
