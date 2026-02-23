use std::borrow::Cow;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::utils::serde::StringOrVec;

const REACTOME_BASE: &str = "https://reactome.org/ContentService";
const REACTOME_API: &str = "reactome";
const REACTOME_BASE_ENV: &str = "BIOMCP_REACTOME_BASE";

pub struct ReactomeClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ReactomeClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(REACTOME_BASE, REACTOME_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, REACTOME_API).await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: REACTOME_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: REACTOME_API.to_string(),
            source,
        })
    }

    pub async fn search_pathways(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<(Vec<ReactomePathwayHit>, Option<usize>), BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Reactome query is required".into(),
            ));
        }

        let url = self.endpoint("search/query");
        let page_size = limit.clamp(1, 25).to_string();
        let resp: ReactomeSearchResponse = self
            .get_json(self.client.get(&url).query(&[
                ("query", query),
                ("species", "Homo sapiens"),
                ("pageSize", page_size.as_str()),
            ]))
            .await?;
        let total_results = resp.total_results;

        let mut out = Vec::new();
        for row in resp.results {
            for entry in row.entries {
                let id = entry
                    .st_id
                    .or(entry.id)
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty());
                let Some(id) = id else { continue };
                let Some(name) = entry
                    .name
                    .map(|v| strip_html(v.trim()))
                    .filter(|v| !v.is_empty())
                else {
                    continue;
                };
                out.push(ReactomePathwayHit { id, name });
                if out.len() >= limit {
                    return Ok((out, total_results));
                }
            }
        }

        Ok((out, total_results))
    }

    pub async fn top_level_pathways(
        &self,
        limit: usize,
    ) -> Result<Vec<ReactomePathwayHit>, BioMcpError> {
        let url = self.endpoint("data/pathways/top/Homo%20sapiens");
        let rows: Vec<ReactomeTopLevelPathway> = self.get_json(self.client.get(&url)).await?;
        let mut out = Vec::new();
        for row in rows.into_iter().take(limit.clamp(1, 200)) {
            let Some(id) = row
                .st_id
                .or(row.id)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            let Some(name) = row
                .display_name
                .map(|v| strip_html(v.trim()))
                .filter(|v| !v.is_empty())
                .or_else(|| {
                    row.name
                        .first()
                        .map(str::trim)
                        .filter(|v| !v.is_empty())
                        .map(strip_html)
                })
            else {
                continue;
            };
            out.push(ReactomePathwayHit { id, name });
        }
        Ok(out)
    }

    pub async fn get_pathway(&self, st_id: &str) -> Result<ReactomePathwayRecord, BioMcpError> {
        let st_id = st_id.trim();
        if st_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Reactome stable ID is required".into(),
            ));
        }

        let url = self.endpoint(&format!("data/query/{st_id}"));
        let resp: ReactomePathwayRecordRaw = self.get_json(self.client.get(&url)).await?;

        Ok(ReactomePathwayRecord {
            id: resp.st_id.unwrap_or_else(|| st_id.to_string()),
            name: resp.display_name.unwrap_or_else(|| st_id.to_string()),
            species: resp.species_name,
            summary: resp
                .summation
                .and_then(|v| v.into_iter().next())
                .and_then(|v| v.text)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty()),
        })
    }

    pub async fn participants(
        &self,
        st_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, BioMcpError> {
        let st_id = st_id.trim();
        if st_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Reactome stable ID is required".into(),
            ));
        }

        let url = self.endpoint(&format!("data/participants/{st_id}"));
        let resp: Vec<ReactomeParticipant> = self.get_json(self.client.get(&url)).await?;

        let mut out = Vec::new();
        for row in resp.into_iter().take(limit.clamp(1, 200)) {
            let Some(name) = row
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            out.push(name.to_string());
        }

        Ok(out)
    }

    pub async fn contained_events(
        &self,
        st_id: &str,
        limit: usize,
    ) -> Result<Vec<String>, BioMcpError> {
        let st_id = st_id.trim();
        if st_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Reactome stable ID is required".into(),
            ));
        }

        let url = self.endpoint(&format!("data/pathway/{st_id}/containedEvents"));
        let resp: Vec<ReactomeContainedEvent> = self.get_json(self.client.get(&url)).await?;

        let mut out = Vec::new();
        for row in resp.into_iter().take(limit.clamp(1, 200)) {
            let ReactomeContainedEvent::Event(row) = row else {
                continue;
            };
            let Some(name) = row
                .display_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
            else {
                continue;
            };
            out.push(name.to_string());
        }
        Ok(out)
    }
}

fn strip_html(value: &str) -> String {
    let mut out = String::new();
    let mut inside = false;
    for ch in value.chars() {
        match ch {
            '<' => inside = true,
            '>' => inside = false,
            _ if !inside => out.push(ch),
            _ => {}
        }
    }
    out.replace("  ", " ").trim().to_string()
}

#[derive(Debug, Clone)]
pub struct ReactomePathwayHit {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ReactomePathwayRecord {
    pub id: String,
    pub name: String,
    pub species: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReactomeSearchResponse {
    #[serde(rename = "totalResults")]
    total_results: Option<usize>,
    #[serde(default)]
    results: Vec<ReactomeSearchResult>,
}

#[derive(Debug, Deserialize)]
struct ReactomeSearchResult {
    #[serde(default)]
    entries: Vec<ReactomeSearchEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeSearchEntry {
    st_id: Option<String>,
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeTopLevelPathway {
    st_id: Option<String>,
    id: Option<String>,
    display_name: Option<String>,
    #[serde(default)]
    name: StringOrVec,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomePathwayRecordRaw {
    st_id: Option<String>,
    display_name: Option<String>,
    species_name: Option<String>,
    summation: Option<Vec<ReactomeSummation>>,
}

#[derive(Debug, Deserialize)]
struct ReactomeSummation {
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeParticipant {
    display_name: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReactomeEvent {
    display_name: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ReactomeContainedEvent {
    Event(ReactomeEvent),
    Id(i64),
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn strip_html_removes_tags_and_extra_spaces() {
        assert_eq!(strip_html("RAF <b>MAPK</b> cascade"), "RAF MAPK cascade");
    }

    #[tokio::test]
    async fn search_pathways_extracts_entries_and_limits_results() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search/query"))
            .and(query_param("query", "MAPK"))
            .and(query_param("species", "Homo sapiens"))
            .and(query_param("pageSize", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "entries": [
                        {"stId": "R-HSA-1", "name": "A <b>pathway</b>"},
                        {"id": "R-HSA-2", "name": "B pathway"}
                    ]
                }]
            })))
            .mount(&server)
            .await;

        let client = ReactomeClient::new_for_test(server.uri()).unwrap();
        let (rows, total) = client.search_pathways("MAPK", 2).await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(total, None);
        assert_eq!(rows[0].id, "R-HSA-1");
        assert_eq!(rows[0].name, "A pathway");
    }

    #[tokio::test]
    async fn contained_events_maps_display_names() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/data/pathway/R-HSA-5673001/containedEvents"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"displayName": "RAS activates RAF"},
                9652817,
                {"displayName": " "}
            ])))
            .mount(&server)
            .await;

        let client = ReactomeClient::new_for_test(server.uri()).unwrap();
        let rows = client.contained_events("R-HSA-5673001", 10).await.unwrap();
        assert_eq!(rows, vec!["RAS activates RAF".to_string()]);
    }
}
