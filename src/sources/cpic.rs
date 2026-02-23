use std::borrow::Cow;
use std::collections::HashMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;

const CPIC_BASE: &str = "https://api.cpicpgx.org/v1";
const CPIC_API: &str = "cpic";
const CPIC_BASE_ENV: &str = "BIOMCP_CPIC_BASE";

#[derive(Debug, Clone)]
pub struct CpicPage<T> {
    pub rows: T,
    pub total: Option<usize>,
}

pub struct CpicClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl CpicClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CPIC_BASE, CPIC_BASE_ENV),
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
        let bytes = crate::sources::read_limited_body(resp, CPIC_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: CPIC_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(CPIC_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: CPIC_API.to_string(),
            source,
        })
    }

    async fn get_json_with_total<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<CpicPage<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let total = parse_content_range_total(resp.headers());
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, CPIC_API).await?;

        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: CPIC_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }

        crate::sources::ensure_json_content_type(CPIC_API, content_type.as_ref(), &bytes)?;
        let rows = serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: CPIC_API.to_string(),
            source,
        })?;
        Ok(CpicPage { rows, total })
    }

    pub async fn pairs_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<CpicPairRow>, BioMcpError> {
        Ok(self.pairs_by_gene_page(gene_symbol, limit, 0).await?.rows)
    }

    pub async fn pairs_by_gene_page(
        &self,
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicPairRow>>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("pair_view");
        let offset = offset.to_string();

        let req = self.client.get(&url).query(&[
            ("genesymbol", format!("eq.{gene_symbol}")),
            ("select", "*".to_string()),
            ("limit", limit.to_string()),
            ("offset", offset),
            ("order", "cpiclevel.asc,drugname.asc".to_string()),
        ]);

        self.get_json_with_total(req).await
    }

    pub async fn pairs_by_drug(
        &self,
        drug_name: &str,
        limit: usize,
    ) -> Result<Vec<CpicPairRow>, BioMcpError> {
        Ok(self.pairs_by_drug_page(drug_name, limit, 0).await?.rows)
    }

    pub async fn pairs_by_drug_page(
        &self,
        drug_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicPairRow>>, BioMcpError> {
        let drug_name = normalize_drug_name(drug_name)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("pair_view");
        let like = format!("ilike.*{}*", sanitize_like_value(&drug_name));
        let offset = offset.to_string();

        let req = self.client.get(&url).query(&[
            ("drugname", like),
            ("select", "*".to_string()),
            ("limit", limit.to_string()),
            ("offset", offset),
            ("order", "cpiclevel.asc,genesymbol.asc".to_string()),
        ]);

        self.get_json_with_total(req).await
    }

    pub async fn recommendations_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<CpicRecommendationRow>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("recommendation_view");
        let lookup_filter = "not.is.null".to_string();
        let lookup_key = format!("lookupkey->>{gene_symbol}");

        let req = self.client.get(&url).query(&[
            (lookup_key, lookup_filter),
            ("select".to_string(), "*".to_string()),
            ("limit".to_string(), limit.to_string()),
        ]);

        self.get_json(req).await
    }

    pub async fn recommendations_by_drug(
        &self,
        drug_name: &str,
        limit: usize,
    ) -> Result<Vec<CpicRecommendationRow>, BioMcpError> {
        let drug_name = normalize_drug_name(drug_name)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("recommendation_view");
        let like = format!("ilike.*{}*", sanitize_like_value(&drug_name));

        let req = self.client.get(&url).query(&[
            ("drugname", like),
            ("select", "*".to_string()),
            ("limit", limit.to_string()),
        ]);

        self.get_json(req).await
    }

    pub async fn frequencies_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<CpicFrequencyRow>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("population_frequency_view");

        let req = self.client.get(&url).query(&[
            ("genesymbol", format!("eq.{gene_symbol}")),
            ("select", "*".to_string()),
            ("limit", limit.to_string()),
        ]);

        self.get_json(req).await
    }

    pub async fn guidelines_by_gene(
        &self,
        gene_symbol: &str,
        limit: usize,
    ) -> Result<Vec<CpicGuidelineSummaryRow>, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        let url = self.endpoint("guideline_summary_view");
        let filter = format!("cs.[{{\"symbol\":\"{gene_symbol}\"}}]");

        let req = self.client.get(&url).query(&[
            ("genes", filter),
            ("select", "*".to_string()),
            ("limit", limit.to_string()),
        ]);

        self.get_json(req).await
    }
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

fn sanitize_like_value(value: &str) -> String {
    value.replace(['*', '%'], "").trim().to_string()
}

fn parse_content_range_total(headers: &reqwest::header::HeaderMap) -> Option<usize> {
    let raw = headers
        .get("content-range")
        .or_else(|| headers.get("Content-Range"))?
        .to_str()
        .ok()?;
    let (_, tail) = raw.rsplit_once('/')?;
    if tail.trim() == "*" {
        return None;
    }
    tail.trim().parse().ok()
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpicPairRow {
    #[allow(dead_code)]
    pub pairid: Option<u64>,
    #[serde(default)]
    pub genesymbol: String,
    #[serde(default)]
    pub drugname: String,
    #[serde(default)]
    pub cpiclevel: Option<String>,
    #[serde(default)]
    pub pgxtesting: Option<String>,
    #[serde(default)]
    pub guidelinename: Option<String>,
    #[serde(default)]
    pub guidelineurl: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub usedforrecommendation: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub provisional: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpicRecommendationRow {
    #[allow(dead_code)]
    pub recommendationid: Option<u64>,
    #[serde(default)]
    #[allow(dead_code)]
    pub lookupkey: HashMap<String, String>,
    #[serde(default)]
    pub drugname: String,
    #[serde(default)]
    pub guidelinename: Option<String>,
    #[serde(default)]
    pub guidelineurl: Option<String>,
    #[serde(default)]
    pub implications: HashMap<String, String>,
    #[serde(default)]
    pub drugrecommendation: Option<String>,
    #[serde(default)]
    pub classification: Option<String>,
    #[serde(default)]
    pub phenotypes: HashMap<String, String>,
    #[serde(default)]
    pub activityscore: HashMap<String, String>,
    #[serde(default)]
    pub population: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpicFrequencyRow {
    #[serde(default)]
    pub genesymbol: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub population_group: Option<String>,
    #[serde(default)]
    pub subjectcount: Option<u64>,
    #[serde(default)]
    pub freq_weighted_avg: Option<f64>,
    #[serde(default)]
    pub freq_avg: Option<f64>,
    #[serde(default)]
    pub freq_max: Option<f64>,
    #[serde(default)]
    pub freq_min: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpicGuidelineSummaryRow {
    #[serde(default)]
    pub guideline_name: String,
    #[serde(default)]
    pub guideline_url: Option<String>,
    #[serde(default)]
    pub drugs: Vec<String>,
    #[serde(default)]
    pub genes: Vec<CpicGuidelineGene>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpicGuidelineGene {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn pairs_by_gene_builds_expected_query() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/pair_view"))
            .and(query_param("genesymbol", "eq.CYP2D6"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "pairid": 1,
                    "genesymbol": "CYP2D6",
                    "drugname": "codeine",
                    "cpiclevel": "A"
                }
            ])))
            .mount(&server)
            .await;

        let client = CpicClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .pairs_by_gene("cyp2d6", 5)
            .await
            .expect("rows should parse");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].drugname, "codeine");
    }

    #[tokio::test]
    async fn recommendations_by_drug_parses_payload() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/recommendation_view"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "recommendationid": 1,
                    "drugname": "codeine",
                    "phenotypes": {"CYP2D6": "Poor Metabolizer"},
                    "activityscore": {"CYP2D6": "0.0"},
                    "drugrecommendation": "Avoid codeine",
                    "classification": "Strong"
                }
            ])))
            .mount(&server)
            .await;

        let client = CpicClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .recommendations_by_drug("codeine", 3)
            .await
            .expect("rows should parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].drugname, "codeine");
        assert_eq!(rows[0].drugrecommendation.as_deref(), Some("Avoid codeine"));
    }

    #[tokio::test]
    async fn guidelines_by_gene_parses_guideline_rows() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/guideline_summary_view"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {
                    "guideline_name": "CYP2D6 and Opioids",
                    "guideline_url": "https://cpicpgx.org/guidelines/guideline-for-codeine-and-cyp2d6/",
                    "drugs": ["codeine"],
                    "genes": [{"symbol": "CYP2D6"}]
                }
            ])))
            .mount(&server)
            .await;

        let client = CpicClient::new_for_test(server.uri()).expect("client");
        let rows = client
            .guidelines_by_gene("CYP2D6", 5)
            .await
            .expect("rows should parse");

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].guideline_name, "CYP2D6 and Opioids");
        assert_eq!(rows[0].genes[0].symbol, "CYP2D6");
    }
}
