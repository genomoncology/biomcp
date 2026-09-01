use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashMap;

use reqwest::StatusCode;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CPIC_BASE: &str = "https://api.cpicpgx.org/v1";
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
    pub(crate) fn with_test_client(
        client: reqwest_middleware::ClientWithMiddleware,
        base: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            client,
            base: base.into(),
        }
    }

    pub(crate) fn pairs_by_gene_plan(
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        Ok(RequestPlan::get("pair_view")
            .query("genesymbol", format!("eq.{gene_symbol}"))
            .query("select", "*")
            .query("limit", limit.to_string())
            .query("offset", offset.to_string())
            .query("order", "cpiclevel.asc,drugname.asc"))
    }

    pub(crate) fn pairs_by_drug_plan(
        drug_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let drug_name = normalize_drug_name(drug_name)?;
        let limit = limit.clamp(1, 200);
        let like = format!("ilike.*{}*", sanitize_like_value(&drug_name));
        Ok(RequestPlan::get("pair_view")
            .query("drugname", like)
            .query("select", "*")
            .query("limit", limit.to_string())
            .query("offset", offset.to_string())
            .query("order", "cpiclevel.asc,genesymbol.asc"))
    }

    #[cfg(test)]
    pub(crate) fn recommendations_by_gene_plan(
        gene_symbol: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        Self::recommendations_by_gene_page_plan(gene_symbol, limit, 0)
    }

    pub(crate) fn recommendations_by_gene_page_plan(
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        Ok(RequestPlan::get("recommendation_view")
            .query(format!("lookupkey->>{gene_symbol}"), "not.is.null")
            .query("select", "*")
            .query("limit", limit.to_string())
            .query("offset", offset.to_string())
            .query("order", "drugname.asc,recommendationid.asc"))
    }

    #[cfg(test)]
    pub(crate) fn recommendations_by_drug_plan(
        drug_name: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        Self::recommendations_by_drug_page_plan(drug_name, limit, 0)
    }

    fn recommendation_drugs_by_gene_plan(gene_symbol: &str) -> Result<RequestPlan, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        Ok(RequestPlan::get("recommendation_view")
            .query(format!("lookupkey->>{gene_symbol}"), "not.is.null")
            .query("select", "drugname")
            .query("limit", "200"))
    }

    pub(crate) fn recommendations_by_drug_page_plan(
        drug_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let drug_name = normalize_drug_name(drug_name)?;
        let limit = limit.clamp(1, 200);
        let like = format!("ilike.*{}*", sanitize_like_value(&drug_name));
        Ok(RequestPlan::get("recommendation_view")
            .query("drugname", like)
            .query("select", "*")
            .query("limit", limit.to_string())
            .query("offset", offset.to_string())
            .query("order", "drugname.asc,recommendationid.asc"))
    }

    #[cfg(test)]
    pub(crate) fn frequencies_by_gene_plan(
        gene_symbol: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        Self::frequencies_by_gene_page_plan(gene_symbol, limit, 0)
    }

    pub(crate) fn frequencies_by_gene_page_plan(
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        Ok(RequestPlan::get("population_frequency_view")
            .query("genesymbol", format!("eq.{gene_symbol}"))
            .query("select", "*")
            .query("limit", limit.to_string())
            .query("offset", offset.to_string())
            .query("order", "genesymbol.asc,allele.asc"))
    }

    #[cfg(test)]
    pub(crate) fn guidelines_by_gene_plan(
        gene_symbol: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        Self::guidelines_by_gene_page_plan(gene_symbol, limit, 0)
    }

    pub(crate) fn guidelines_by_gene_page_plan(
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let gene_symbol = normalize_gene_symbol(gene_symbol)?;
        let limit = limit.clamp(1, 200);
        let filter = format!("cs.[{{\"symbol\":\"{gene_symbol}\"}}]");
        Ok(RequestPlan::get("guideline_summary_view")
            .query("genes", filter)
            .query("select", "*")
            .query("limit", limit.to_string())
            .query("offset", offset.to_string())
            .query("order", "guidelinename.asc"))
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CPIC),
            status,
            content_type,
            bytes,
            true,
        )
    }

    pub(crate) fn decode_json_page_response<T: DeserializeOwned>(
        status: StatusCode,
        headers: &HeaderMap,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<CpicPage<T>, BioMcpError> {
        let total = parse_content_range_total(headers);
        let rows = Self::decode_json_response(status, content_type, bytes)?;
        Ok(CpicPage { rows, total })
    }

    async fn get_json_with_total<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<CpicPage<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CPIC,
            ))
            .await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CPIC),
        )
        .await?;
        Self::decode_json_page_response(status, &headers, content_type.as_ref(), &bytes).map_err(
            |error| {
                error.with_source_context(crate::error::SourceContext::retry(
                    crate::error::SourceProvider::CPIC,
                ))
            },
        )
    }

    pub async fn pairs_by_gene_page(
        &self,
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicPairRow>>, BioMcpError> {
        let plan = Self::pairs_by_gene_plan(gene_symbol, limit, offset)?;
        self.get_json_with_total(
            request_from_plan(&self.client, self.base.as_ref(), &plan)
                .header("Prefer", "count=exact"),
        )
        .await
    }

    pub async fn pairs_by_drug_page(
        &self,
        drug_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicPairRow>>, BioMcpError> {
        let plan = Self::pairs_by_drug_plan(drug_name, limit, offset)?;
        self.get_json_with_total(
            request_from_plan(&self.client, self.base.as_ref(), &plan)
                .header("Prefer", "count=exact"),
        )
        .await
    }

    pub async fn recommendations_by_gene_page(
        &self,
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicRecommendationRow>>, BioMcpError> {
        let plan = Self::recommendations_by_gene_page_plan(gene_symbol, limit, offset)?;
        self.get_json_with_total(
            request_from_plan(&self.client, self.base.as_ref(), &plan)
                .header("Prefer", "count=exact"),
        )
        .await
    }

    pub async fn recommendation_drugs_by_gene(
        &self,
        gene_symbol: &str,
    ) -> Result<Vec<String>, BioMcpError> {
        let plan = Self::recommendation_drugs_by_gene_plan(gene_symbol)?;
        let mut drugs: Vec<_> = self
            .get_json_with_total::<Vec<CpicRecommendationRow>>(request_from_plan(
                &self.client,
                self.base.as_ref(),
                &plan,
            ))
            .await?
            .rows
            .into_iter()
            .filter_map(|row| {
                let drug = row.drugname.trim();
                (!drug.is_empty()).then(|| drug.to_string())
            })
            .collect();
        drugs.sort();
        drugs.dedup();
        Ok(drugs)
    }

    pub async fn recommendations_by_drug_page(
        &self,
        drug_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicRecommendationRow>>, BioMcpError> {
        let plan = Self::recommendations_by_drug_page_plan(drug_name, limit, offset)?;
        self.get_json_with_total(
            request_from_plan(&self.client, self.base.as_ref(), &plan)
                .header("Prefer", "count=exact"),
        )
        .await
    }

    pub async fn frequencies_by_gene_page(
        &self,
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicFrequencyRow>>, BioMcpError> {
        let plan = Self::frequencies_by_gene_page_plan(gene_symbol, limit, offset)?;
        self.get_json_with_total(
            request_from_plan(&self.client, self.base.as_ref(), &plan)
                .header("Prefer", "count=exact"),
        )
        .await
    }

    pub async fn guidelines_by_gene_page(
        &self,
        gene_symbol: &str,
        limit: usize,
        offset: usize,
    ) -> Result<CpicPage<Vec<CpicGuidelineSummaryRow>>, BioMcpError> {
        let plan = Self::guidelines_by_gene_page_plan(gene_symbol, limit, offset)?;
        self.get_json_with_total(
            request_from_plan(&self.client, self.base.as_ref(), &plan)
                .header("Prefer", "count=exact"),
        )
        .await
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
    // dead-code reason: cpic::pairid preserves the provider shape used by source contract fixtures
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
    // dead-code reason: cpic::usedforrecommendation preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub usedforrecommendation: Option<String>,
    #[serde(default)]
    // dead-code reason: cpic::provisional preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub provisional: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CpicRecommendationRow {
    // dead-code reason: cpic::recommendationid preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub recommendationid: Option<u64>,
    #[serde(default)]
    // dead-code reason: cpic::lookupkey preserves the provider shape used by source contract fixtures
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
    // dead-code reason: cpic::url preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub url: Option<String>,
}

#[cfg(test)]
mod tests;
