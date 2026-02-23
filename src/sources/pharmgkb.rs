use std::borrow::Cow;
use std::collections::HashSet;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const PHARMGKB_BASE: &str = "https://api.pharmgkb.org/v1";
const PHARMGKB_API: &str = "pharmgkb";
const PHARMGKB_BASE_ENV: &str = "BIOMCP_PHARMGKB_BASE";

pub struct PharmGkbClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl PharmGkbClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(PHARMGKB_BASE, PHARMGKB_BASE_ENV),
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

    async fn get_json_optional<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<Option<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, PHARMGKB_API).await?;

        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: PHARMGKB_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(PHARMGKB_API, content_type.as_ref(), &bytes)?;

        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|source| BioMcpError::ApiJson {
                api: PHARMGKB_API.to_string(),
                source,
            })
    }

    pub async fn annotations_by_drug(
        &self,
        drug_name: &str,
        limit: usize,
    ) -> Result<Vec<PharmGkbAnnotation>, BioMcpError> {
        let drug_name = normalize_drug_name(drug_name)?;
        let limit = limit.clamp(1, 100);

        let mut out = Vec::new();
        out.extend(
            self.fetch_annotations(
                "clinicalAnnotation",
                "relatedChemicals.name",
                &drug_name,
                "Clinical Annotation",
                limit,
            )
            .await?,
        );
        out.extend(
            self.fetch_annotations(
                "guidelineAnnotation",
                "relatedChemicals.name",
                &drug_name,
                "Guideline Annotation",
                limit,
            )
            .await?,
        );
        out.extend(
            self.fetch_annotations(
                "labelAnnotation",
                "relatedChemicals.name",
                &drug_name,
                "Label Annotation",
                limit,
            )
            .await?,
        );

        Ok(dedupe_and_limit(out, limit))
    }

    pub async fn annotations_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<PharmGkbAnnotation>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 100);

        let mut out = Vec::new();
        out.extend(
            self.fetch_annotations(
                "clinicalAnnotation",
                "location.genes.symbol",
                &gene_symbol,
                "Clinical Annotation",
                limit,
            )
            .await?,
        );
        out.extend(
            self.fetch_annotations(
                "guidelineAnnotation",
                "relatedGenes.symbol",
                &gene_symbol,
                "Guideline Annotation",
                limit,
            )
            .await?,
        );
        out.extend(
            self.fetch_annotations(
                "labelAnnotation",
                "relatedGenes.symbol",
                &gene_symbol,
                "Label Annotation",
                limit,
            )
            .await?,
        );

        Ok(dedupe_and_limit(out, limit))
    }

    async fn fetch_annotations(
        &self,
        endpoint: &str,
        criteria_key: &str,
        criteria_value: &str,
        fallback_kind: &str,
        limit: usize,
    ) -> Result<Vec<PharmGkbAnnotation>, BioMcpError> {
        let url = self.endpoint(&format!("data/{endpoint}"));
        let req = self
            .client
            .get(&url)
            .query(&[(criteria_key, criteria_value), ("view", "min")]);

        let Some(resp): Option<PharmGkbDataResponse> = self.get_json_optional(req).await? else {
            return Ok(Vec::new());
        };

        let mut out = Vec::new();
        for row in resp.data {
            if let Some(annotation) = map_annotation(&row, fallback_kind) {
                out.push(annotation);
            }
            if out.len() >= limit {
                break;
            }
        }

        Ok(out)
    }
}

fn normalize_drug_name(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "PGx drug is required. Example: biomcp get pgx warfarin".into(),
        ));
    }
    if normalized.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Drug name is too long.".into(),
        ));
    }
    Ok(normalized)
}

fn normalize_gene_symbol(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "PGx gene is required. Example: biomcp get pgx CYP2D6".into(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(&normalized) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {value}"
        )));
    }
    Ok(normalized)
}

fn dedupe_and_limit(rows: Vec<PharmGkbAnnotation>, limit: usize) -> Vec<PharmGkbAnnotation> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for row in rows {
        let key = format!("{}|{}|{}", row.kind, row.id, row.title.to_ascii_lowercase());
        if !seen.insert(key) {
            continue;
        }
        out.push(row);
        if out.len() >= limit {
            break;
        }
    }
    out
}

fn map_annotation(row: &serde_json::Value, fallback_kind: &str) -> Option<PharmGkbAnnotation> {
    let obj = row.as_object()?;

    let id = obj
        .get("id")
        .and_then(to_string_value)
        .filter(|v| !v.trim().is_empty())?;

    let kind = obj
        .get("objCls")
        .and_then(to_string_value)
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| fallback_kind.to_string());

    let title = obj
        .get("name")
        .and_then(to_string_value)
        .or_else(|| obj.get("accessionId").and_then(to_string_value))
        .or_else(|| {
            obj.get("location")
                .and_then(|v| v.get("displayName"))
                .and_then(to_string_value)
        })
        .unwrap_or_else(|| id.clone());

    let level = obj
        .get("levelOfEvidence")
        .and_then(|v| v.get("term"))
        .and_then(to_string_value)
        .filter(|v| !v.trim().is_empty());

    let url = obj
        .get("crossReferences")
        .and_then(|v| v.as_array())
        .and_then(|rows| {
            rows.iter().find_map(|xref| {
                xref.get("_url")
                    .and_then(to_string_value)
                    .or_else(|| xref.get("resourceId").and_then(to_string_value))
            })
        })
        .filter(|v| v.starts_with("http"));

    Some(PharmGkbAnnotation {
        source: "PharmGKB".to_string(),
        kind,
        id,
        title,
        level,
        url,
    })
}

fn to_string_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(v) => Some(v.trim().to_string()),
        serde_json::Value::Number(v) => Some(v.to_string()),
        _ => None,
    }
}

#[derive(Debug, Clone, Deserialize)]
struct PharmGkbDataResponse {
    #[serde(default)]
    data: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PharmGkbAnnotation {
    pub source: String,
    pub kind: String,
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn annotations_by_drug_collects_multiple_kinds() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/data/clinicalAnnotation"))
            .and(query_param("relatedChemicals.name", "warfarin"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "id": 981239556,
                        "accessionId": "PA166134613",
                        "levelOfEvidence": {"term": "3"}
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/guidelineAnnotation"))
            .and(query_param("relatedChemicals.name", "warfarin"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "objCls": "Guideline Annotation",
                        "id": "PA166104949",
                        "name": "Annotation of CPIC Guideline for warfarin"
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/labelAnnotation"))
            .and(query_param("relatedChemicals.name", "warfarin"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "objCls": "Label Annotation",
                        "id": "PA166104776",
                        "name": "Annotation of FDA Label for warfarin"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = PharmGkbClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .annotations_by_drug("warfarin", 10)
            .await
            .expect("annotations");

        assert_eq!(rows.len(), 3);
        assert!(rows.iter().any(|row| row.kind.contains("Guideline")));
        assert!(rows.iter().any(|row| row.kind.contains("Label")));
    }

    #[tokio::test]
    async fn annotations_by_gene_uses_expected_properties() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/data/clinicalAnnotation"))
            .and(query_param("location.genes.symbol", "CYP2D6"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {
                        "id": 1,
                        "accessionId": "PA1"
                    }
                ]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/guidelineAnnotation"))
            .and(query_param("relatedGenes.symbol", "CYP2D6"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/data/labelAnnotation"))
            .and(query_param("relatedGenes.symbol", "CYP2D6"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": []
            })))
            .mount(&server)
            .await;

        let client = PharmGkbClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .annotations_by_gene("cyp2d6", 5)
            .await
            .expect("annotations");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "1");
    }
}
