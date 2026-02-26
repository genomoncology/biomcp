use std::borrow::Cow;
use std::cmp::Ordering;
use std::io::Read;

use flate2::read::GzDecoder;
use reqwest::header::ACCEPT;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const UNIPROT_BASE: &str = "https://rest.uniprot.org";
const UNIPROT_API: &str = "uniprot";
const UNIPROT_BASE_ENV: &str = "BIOMCP_UNIPROT_BASE";

pub struct UniProtClient {
    client: reqwest::Client,
    base: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct UniProtSearchPage {
    pub results: Vec<UniProtRecord>,
    pub total: Option<usize>,
    pub next_page_token: Option<String>,
}

impl UniProtClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::streaming_http_client()?,
            base: crate::sources::env_base(UNIPROT_BASE, UNIPROT_BASE_ENV),
        })
    }

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::streaming_http_client()?,
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

    async fn get_json<T, F>(&self, build_request: F) -> Result<T, BioMcpError>
    where
        T: DeserializeOwned,
        F: Fn() -> reqwest::RequestBuilder,
    {
        let resp =
            crate::sources::retry_send(UNIPROT_API, 3, || async { build_request().send().await })
                .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, UNIPROT_API).await?;
        let mut payload = bytes.to_vec();
        if payload.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = GzDecoder::new(payload.as_slice());
            let mut decoded = Vec::new();
            decoder
                .read_to_end(&mut decoded)
                .map_err(|err| BioMcpError::Api {
                    api: UNIPROT_API.to_string(),
                    message: format!("Failed to decode gzip response: {err}"),
                })?;
            payload = decoded;
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&payload);
            return Err(BioMcpError::Api {
                api: UNIPROT_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&payload).map_err(|source| {
            let excerpt = crate::sources::body_excerpt(&payload);
            BioMcpError::Api {
                api: UNIPROT_API.to_string(),
                message: format!("Invalid JSON response: {excerpt} ({source})"),
            }
        })
    }

    pub async fn get_record(&self, accession: &str) -> Result<UniProtRecord, BioMcpError> {
        let accession = accession.trim();
        if accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "UniProt accession is required".into(),
            ));
        }

        let url = self.endpoint(&format!("uniprotkb/{accession}.json"));
        crate::sources::rate_limit::wait_for_url_str(&url).await;
        self.get_json(|| self.client.get(&url).header(ACCEPT, "application/json"))
            .await
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        next_page: Option<&str>,
    ) -> Result<UniProtSearchPage, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "UniProt query is required".into(),
            ));
        }

        let url = self.endpoint("uniprotkb/search");
        let size = limit.clamp(1, 25).to_string();
        let offset = offset.to_string();
        crate::sources::rate_limit::wait_for_url_str(&url).await;
        let token = normalize_next_page_token(next_page)?;
        let token_for_request = token.clone();
        let resp = crate::sources::retry_send(UNIPROT_API, 3, || async {
            if let Some(token) = token_for_request.as_deref() {
                if token.starts_with("http://") || token.starts_with("https://") {
                    return self.client.get(token).header(ACCEPT, "application/json").send().await;
                }

                return self
                    .client
                    .get(&url)
                    .header(ACCEPT, "application/json")
                    .query(&[
                        ("query", query),
                        ("format", "json"),
                        ("size", size.as_str()),
                        ("cursor", token),
                        (
                            "fields",
                            "accession,id,protein_name,gene_names,organism_name,length,cc_function,xref_pdb,xref_alphafolddb",
                        ),
                    ])
                    .send()
                    .await;
            }

            self.client
                .get(&url)
                .header(ACCEPT, "application/json")
                .query(&[
                    ("query", query),
                    ("format", "json"),
                    ("size", size.as_str()),
                    ("offset", offset.as_str()),
                    (
                        "fields",
                        "accession,id,protein_name,gene_names,organism_name,length,cc_function,xref_pdb,xref_alphafolddb",
                    ),
                ])
                .send()
                .await
        })
        .await?;
        let status = resp.status();
        let total = resp
            .headers()
            .get("x-total-results")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok());
        let next_page_token = parse_uniprot_next_link(resp.headers().get("link"));
        let bytes = crate::sources::read_limited_body(resp, UNIPROT_API).await?;
        let mut payload = bytes.to_vec();
        if payload.starts_with(&[0x1f, 0x8b]) {
            let mut decoder = GzDecoder::new(payload.as_slice());
            let mut decoded = Vec::new();
            decoder
                .read_to_end(&mut decoded)
                .map_err(|err| BioMcpError::Api {
                    api: UNIPROT_API.to_string(),
                    message: format!("Failed to decode gzip response: {err}"),
                })?;
            payload = decoded;
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&payload);
            return Err(BioMcpError::Api {
                api: UNIPROT_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        let parsed: UniProtSearchResponse =
            serde_json::from_slice(&payload).map_err(|source| BioMcpError::Api {
                api: UNIPROT_API.to_string(),
                message: format!(
                    "Invalid JSON response: {} ({source})",
                    crate::sources::body_excerpt(&payload)
                ),
            })?;
        Ok(UniProtSearchPage {
            results: parsed.results,
            total,
            next_page_token,
        })
    }
}

fn parse_uniprot_next_link(value: Option<&reqwest::header::HeaderValue>) -> Option<String> {
    let raw = value?.to_str().ok()?;
    for part in raw.split(',') {
        let piece = part.trim();
        if !piece.contains("rel=\"next\"") {
            continue;
        }
        let start = piece.find('<')?;
        let end = piece[start + 1..].find('>')?;
        let url = piece[start + 1..start + 1 + end].trim();
        if !url.is_empty() {
            return Some(url.to_string());
        }
    }
    None
}

fn normalize_next_page_token(next_page: Option<&str>) -> Result<Option<String>, BioMcpError> {
    let Some(token) = next_page
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Ok(None);
    };

    if token.len() > 2048 {
        return Err(BioMcpError::InvalidArgument(
            "--next-page token is too long".into(),
        ));
    }
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(
            "--next-page token is invalid. Use pagination.next_page_token from the previous result."
                .into(),
        ));
    }
    if token.chars().any(|ch| ch.is_whitespace()) {
        return Err(BioMcpError::InvalidArgument(
            "--next-page token must not contain whitespace".into(),
        ));
    }
    if token.starts_with("http://") || token.starts_with("https://") {
        let parsed = reqwest::Url::parse(&token).map_err(|_| {
            BioMcpError::InvalidArgument(
                "--next-page token URL is invalid. Use pagination.next_page_token from the previous result."
                    .into(),
            )
        })?;
        if parsed.host_str() != Some("rest.uniprot.org") {
            return Err(BioMcpError::InvalidArgument(
                "--next-page token must be a rest.uniprot.org URL. Use pagination.next_page_token from the previous result.".into(),
            ));
        }
    }

    Ok(Some(token))
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtSearchResponse {
    #[serde(default)]
    pub results: Vec<UniProtRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtRecord {
    #[serde(default)]
    pub primary_accession: String,
    #[serde(rename = "uniProtkbId")]
    pub uni_prot_kb_id: Option<String>,
    pub protein_description: Option<UniProtProteinDescription>,
    #[serde(default)]
    pub genes: Vec<UniProtGene>,
    pub organism: Option<UniProtOrganism>,
    pub sequence: Option<UniProtSequence>,
    #[serde(default)]
    pub comments: Vec<UniProtComment>,
    #[serde(rename = "uniProtKBCrossReferences", default)]
    pub uni_prot_kb_cross_references: Vec<UniProtCrossReference>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtProteinDescription {
    pub recommended_name: Option<UniProtNameContainer>,
    pub submission_names: Option<Vec<UniProtNameContainer>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtNameContainer {
    pub full_name: Option<UniProtTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtTextValue {
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtGene {
    pub gene_name: Option<UniProtTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtOrganism {
    pub scientific_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtSequence {
    pub length: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtComment {
    pub comment_type: Option<String>,
    #[serde(default)]
    pub texts: Vec<UniProtTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtCrossReference {
    pub database: Option<String>,
    pub id: Option<String>,
    #[serde(default)]
    pub properties: Vec<UniProtCrossReferenceProperty>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtCrossReferenceProperty {
    pub key: Option<String>,
    pub value: Option<String>,
}

impl UniProtRecord {
    pub fn display_name(&self) -> String {
        if let Some(desc) = self.protein_description.as_ref() {
            if let Some(value) = desc
                .recommended_name
                .as_ref()
                .and_then(|v| v.full_name.as_ref())
                .map(|v| v.value.trim())
                .filter(|v| !v.is_empty())
            {
                return value.to_string();
            }

            if let Some(value) = desc
                .submission_names
                .as_ref()
                .and_then(|v| v.first())
                .and_then(|v| v.full_name.as_ref())
                .map(|v| v.value.trim())
                .filter(|v| !v.is_empty())
            {
                return value.to_string();
            }
        }

        self.primary_accession.clone()
    }

    pub fn primary_gene_symbol(&self) -> Option<String> {
        self.genes
            .first()
            .and_then(|g| g.gene_name.as_ref())
            .map(|g| g.value.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    pub fn function_summary(&self) -> Option<String> {
        self.comments
            .iter()
            .find(|c| {
                c.comment_type
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.eq_ignore_ascii_case("function"))
            })
            .and_then(|c| c.texts.first())
            .map(|v| v.value.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    pub fn structure_ids(&self) -> Vec<String> {
        let mut out = Vec::new();
        for x in &self.uni_prot_kb_cross_references {
            let Some(db) = x.database.as_deref().map(str::trim) else {
                continue;
            };
            let Some(id) = x.id.as_deref().map(str::trim) else {
                continue;
            };
            if id.is_empty() {
                continue;
            }
            if !matches!(db, "PDB" | "AlphaFoldDB") {
                continue;
            }
            if out.iter().any(|v: &String| v == id) {
                continue;
            }
            out.push(id.to_string());
        }
        out
    }

    pub fn structure_count(&self) -> usize {
        self.structure_ids().len()
    }

    pub fn structure_summaries(&self, limit: usize) -> Vec<String> {
        #[derive(Debug)]
        struct PdbRow {
            id: String,
            method: Option<String>,
            resolution_text: Option<String>,
            resolution_value: Option<f64>,
        }

        let limit = limit.max(1);
        let mut seen: Vec<String> = Vec::new();
        let mut pdb_rows: Vec<PdbRow> = Vec::new();
        let mut other_rows: Vec<String> = Vec::new();

        for x in &self.uni_prot_kb_cross_references {
            let Some(db) = x.database.as_deref().map(str::trim) else {
                continue;
            };
            let Some(id) = x.id.as_deref().map(str::trim) else {
                continue;
            };
            if id.is_empty() {
                continue;
            }
            if !matches!(db, "PDB" | "AlphaFoldDB") {
                continue;
            }
            if seen.iter().any(|v| v == id) {
                continue;
            }
            seen.push(id.to_string());

            if db == "PDB" {
                let method = cross_ref_property(x, "Method");
                let resolution_text = cross_ref_property(x, "Resolution")
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty() && v != "-");
                let resolution_value = resolution_text
                    .as_deref()
                    .and_then(parse_resolution_angstrom);

                pdb_rows.push(PdbRow {
                    id: id.to_string(),
                    method,
                    resolution_text,
                    resolution_value,
                });
            } else {
                other_rows.push(format!("{id} (AlphaFold model)"));
            }
        }

        pdb_rows.sort_by(|a, b| match (a.resolution_value, b.resolution_value) {
            (Some(lhs), Some(rhs)) => lhs.partial_cmp(&rhs).unwrap_or(Ordering::Equal),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        });

        let mut out: Vec<String> = Vec::new();
        for row in pdb_rows {
            let line = match (row.method.as_deref(), row.resolution_text.as_deref()) {
                (Some(method), Some(resolution)) => format!("{} ({method}, {resolution})", row.id),
                (Some(method), None) => format!("{} ({method})", row.id),
                (None, Some(resolution)) => format!("{} ({resolution})", row.id),
                (None, None) => row.id,
            };
            out.push(line);
            if out.len() >= limit {
                return out;
            }
        }

        for row in other_rows {
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }

        out
    }
}

fn cross_ref_property(row: &UniProtCrossReference, key: &str) -> Option<String> {
    row.properties.iter().find_map(|p| {
        let matches = p
            .key
            .as_deref()
            .map(str::trim)
            .is_some_and(|k| k.eq_ignore_ascii_case(key));
        if !matches {
            return None;
        }
        p.value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    })
}

fn parse_resolution_angstrom(value: &str) -> Option<f64> {
    let token = value
        .trim()
        .trim_end_matches('A')
        .trim_end_matches('a')
        .trim();
    let token = token.split_whitespace().next()?;
    token.parse::<f64>().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn search_sets_expected_query_params() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/uniprotkb/search"))
            .and(query_param("query", "BRAF"))
            .and(query_param("format", "json"))
            .and(query_param("size", "3"))
            .and(query_param("offset", "0"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "results": [{
                    "primaryAccession": "P15056",
                    "uniProtkbId": "BRAF_HUMAN",
                    "proteinDescription": {
                        "recommendedName": {"fullName": {"value": "Serine/threonine-protein kinase B-raf"}}
                    },
                    "genes": [{"geneName": {"value": "BRAF"}}]
                }]
            })))
            .mount(&server)
            .await;

        let client = UniProtClient::new_for_test(server.uri()).unwrap();
        let page = client.search("BRAF", 3, 0, None).await.unwrap();
        assert_eq!(page.results.len(), 1);
        assert_eq!(page.results[0].primary_accession, "P15056");
        assert_eq!(
            page.results[0].primary_gene_symbol().as_deref(),
            Some("BRAF")
        );
    }

    #[test]
    fn record_helpers_extract_display_function_and_structures() {
        let record: UniProtRecord = serde_json::from_value(serde_json::json!({
            "primaryAccession": "P15056",
            "proteinDescription": {
                "recommendedName": {"fullName": {"value": " Kinase X "}}
            },
            "comments": [
                {"commentType": "FUNCTION", "texts": [{"value": " Signal transduction. "}]}
            ],
            "uniProtKBCrossReferences": [
                {
                    "database": "PDB",
                    "id": "1UWH",
                    "properties": [
                        {"key": "Method", "value": "X-ray"},
                        {"key": "Resolution", "value": "2.95 A"}
                    ]
                },
                {"database": "PDB", "id": "1UWH"},
                {"database": "AlphaFoldDB", "id": "AF-P15056-F1"},
                {"database": "GO", "id": "GO:0004672"}
            ]
        }))
        .unwrap();

        assert_eq!(record.display_name(), "Kinase X");
        assert_eq!(
            record.function_summary().as_deref(),
            Some("Signal transduction.")
        );
        assert_eq!(
            record.structure_ids(),
            vec!["1UWH".to_string(), "AF-P15056-F1".to_string()]
        );
        assert_eq!(record.structure_count(), 2);
        assert_eq!(
            record.structure_summaries(10),
            vec![
                "1UWH (X-ray, 2.95 A)".to_string(),
                "AF-P15056-F1 (AlphaFold model)".to_string()
            ]
        );
    }

    #[test]
    fn normalize_next_page_token_rejects_numeric_only_tokens() {
        let err = normalize_next_page_token(Some("12345")).expect_err("numeric token should fail");
        assert!(err.to_string().contains("--next-page token is invalid"));
    }

    #[test]
    fn normalize_next_page_token_accepts_cursor_url() {
        let token =
            normalize_next_page_token(Some("https://rest.uniprot.org/uniprotkb/search?cursor=abc"))
                .expect("valid URL token");
        assert!(token.is_some());
    }
}
