use std::borrow::Cow;

use http_cache_reqwest::CacheMode;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};
use crate::xml::{ARTICLE_XML_NODE_LIMIT, parse_external_xml};

const NCBI_EFETCH_BASE: &str = "https://eutils.ncbi.nlm.nih.gov/entrez/eutils";
const NCBI_EFETCH_API: &str = "pubmed-eutils";
const NCBI_EFETCH_BASE_ENV: &str = "BIOMCP_PUBMED_BASE";

#[derive(Clone)]
pub struct NcbiEfetchClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
    api_key: Option<String>,
}

impl NcbiEfetchClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(NCBI_EFETCH_BASE, NCBI_EFETCH_BASE_ENV),
            api_key: crate::sources::ncbi_api_key(),
        })
    }

    pub(crate) fn normalize_pmcid(pmcid: &str) -> Result<Option<String>, BioMcpError> {
        let pmcid = pmcid.trim();
        if pmcid.is_empty() {
            return Ok(None);
        }
        if pmcid.len() > 64 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        let numeric = if pmcid
            .get(..3)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("PMC"))
        {
            &pmcid[3..]
        } else {
            pmcid
        };

        if numeric.is_empty() || !numeric.chars().all(|ch| ch.is_ascii_digit()) {
            return Err(BioMcpError::InvalidArgument(
                "PMCID must start with PMC and contain only digits after.".into(),
            ));
        }
        if numeric.len() > 32 {
            return Err(BioMcpError::InvalidArgument("PMCID is too long.".into()));
        }

        Ok(Some(numeric.to_string()))
    }

    pub(crate) fn full_text_xml_plan(
        pmcid: &str,
        api_key: Option<&str>,
    ) -> Result<Option<RequestPlan>, BioMcpError> {
        let Some(numeric_pmcid) = Self::normalize_pmcid(pmcid)? else {
            return Ok(None);
        };

        let mut plan = RequestPlan::get("efetch.fcgi")
            .query("db", "pmc")
            .query("id", numeric_pmcid)
            .query("rettype", "xml");
        if let Some(key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
            plan = plan.query("api_key", key);
        }
        Ok(Some(plan))
    }

    pub(crate) fn decode_text(
        status: reqwest::StatusCode,
        bytes: &[u8],
    ) -> Result<String, BioMcpError> {
        if matches!(
            status,
            reqwest::StatusCode::NOT_FOUND | reqwest::StatusCode::NO_CONTENT
        ) {
            return Ok(String::new());
        }
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(bytes);
            return Err(BioMcpError::Api {
                api: NCBI_EFETCH_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        Ok(String::from_utf8_lossy(bytes).to_string())
    }

    async fn get_text(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<String, BioMcpError> {
        let resp = req.with_extension(CacheMode::NoStore).send().await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_body(resp, NCBI_EFETCH_API).await?;
        Self::decode_text(status, &bytes)
    }

    pub async fn get_full_text_xml(&self, pmcid: &str) -> Result<Option<String>, BioMcpError> {
        let Some(plan) = Self::full_text_xml_plan(pmcid, self.api_key.as_deref())? else {
            return Ok(None);
        };

        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let xml = self.get_text(req).await?;
        normalize_article_xml(&xml)
    }
}

pub(crate) fn normalize_article_xml(xml: &str) -> Result<Option<String>, BioMcpError> {
    let trimmed = xml.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let doc = match parse_external_xml(trimmed, ARTICLE_XML_NODE_LIMIT) {
        Ok(doc) => doc,
        Err(_) => return Ok(Some(trimmed.to_string())),
    };

    let article = doc
        .descendants()
        .find(|node| node.is_element() && node.has_tag_name("article"));
    let Some(article) = article else {
        return Ok(Some(trimmed.to_string()));
    };

    Ok(Some(trimmed[article.range()].to_string()))
}

#[cfg(test)]
mod tests;
