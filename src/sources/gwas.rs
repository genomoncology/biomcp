use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use http_cache_reqwest::CacheMode;
use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Deserializer;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const GWAS_BASE: &str = "https://www.ebi.ac.uk/gwas/rest/api";
const GWAS_API: &str = "gwas";
const GWAS_BASE_ENV: &str = "BIOMCP_GWAS_BASE";

pub struct GwasClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl GwasClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(GWAS_BASE, GWAS_BASE_ENV),
        })
    }

    pub(crate) fn associations_by_rsid_plan(
        rsid: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let rsid = normalize_rsid(rsid)?;
        Ok(
            RequestPlan::get(format!("singleNucleotidePolymorphisms/{rsid}/associations"))
                .query("projection", "associationByStudy")
                .query("page", "0")
                .query("size", limit.clamp(1, 200).to_string()),
        )
    }

    pub(crate) fn association_search_plan(
        gene: Option<&str>,
        trait_query: Option<&str>,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        if gene.is_some() == trait_query.is_some() {
            return Err(BioMcpError::InvalidArgument(
                "GWAS association search requires exactly one gene or trait filter".into(),
            ));
        }
        if limit == 0 || limit > 50 {
            return Err(BioMcpError::InvalidArgument(
                "GWAS association search limit must be between 1 and 50".into(),
            ));
        }
        let mut plan = RequestPlan::get("v2/associations")
            .query("page", "0")
            .query("size", limit.to_string())
            .query("sort", "p_value")
            .query("direction", "asc");
        if let Some(gene) = gene {
            plan = plan.query("mapped_gene", normalize_gene_symbol(gene)?);
        }
        if let Some(trait_query) = trait_query {
            plan = plan.query("efo_trait", normalize_trait_query(trait_query)?);
        }
        Ok(plan)
    }

    fn request_no_store(&self, plan: &RequestPlan) -> reqwest_middleware::RequestBuilder {
        // GWAS responses occasionally produce cache decode failures when a stale
        // body entry is reused. Always bypass persistence for this source.
        request_from_plan(&self.client, self.base.as_ref(), plan).with_extension(CacheMode::NoStore)
    }

    pub(crate) fn decode_json_optional<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<Option<T>, BioMcpError> {
        if status == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::GWAS),
            status,
            content_type,
            bytes,
            true,
        )
        .map(Some)
    }

    async fn get_json_optional<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<Option<T>, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GWAS,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::GWAS),
        )
        .await?;

        if status == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }
        Self::decode_json_optional(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GWAS,
            ))
        })
    }

    pub async fn associations_by_rsid(
        &self,
        rsid: &str,
        limit: usize,
    ) -> Result<Vec<GwasAssociation>, BioMcpError> {
        let plan = Self::associations_by_rsid_plan(rsid, limit)?;
        let req = self.request_no_store(&plan);

        let Some(resp): Option<GwasAssociationsResponse> = self
            .get_json_optional(req)
            .await
            .map_err(remap_gwas_error)?
        else {
            return Ok(Vec::new());
        };

        Ok(resp.embedded.associations)
    }

    pub async fn search_associations(
        &self,
        gene: Option<&str>,
        trait_query: Option<&str>,
        limit: usize,
    ) -> Result<GwasAssociationSearchPage, BioMcpError> {
        let plan = Self::association_search_plan(gene, trait_query, limit)?;
        let req = self.request_no_store(&plan);
        let Some(resp): Option<GwasV2AssociationsResponse> = self
            .get_json_optional(req)
            .await
            .map_err(remap_gwas_error)?
        else {
            return Ok(GwasAssociationSearchPage::default());
        };
        Ok(GwasAssociationSearchPage {
            associations: resp.embedded.associations,
            total: resp.page.total_elements,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct GwasAssociationSearchPage {
    pub associations: Vec<GwasAssociationSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasV2AssociationsResponse {
    #[serde(default, rename = "_embedded")]
    embedded: GwasV2AssociationsEmbedded,
    #[serde(default)]
    page: GwasV2Page,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasV2AssociationsEmbedded {
    #[serde(default)]
    associations: Vec<GwasAssociationSummary>,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasV2Page {
    #[serde(default, rename = "totalElements")]
    total_elements: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasAssociationSummary {
    #[serde(default)]
    pub snp_allele: Vec<GwasAlleleSummary>,
    #[serde(default)]
    pub snp_effect_allele: Vec<String>,
    #[serde(default)]
    pub efo_traits: Vec<GwasV2Trait>,
    #[serde(default)]
    pub reported_trait: Vec<String>,
    #[serde(default)]
    pub mapped_genes: Vec<String>,
    pub p_value: Option<f64>,
    pub or_per_copy_num: Option<f64>,
    pub beta_num: Option<f64>,
    pub range: Option<String>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub risk_frequency: Option<f64>,
    pub accession_id: Option<String>,
    pub pubmed_id: Option<String>,
    pub first_author: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasAlleleSummary {
    pub rs_id: Option<String>,
    pub effect_allele: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasV2Trait {
    pub efo_trait: Option<String>,
}

fn remap_gwas_error(err: BioMcpError) -> BioMcpError {
    match err {
        BioMcpError::WithSourceContext { context, source } => {
            remap_gwas_error(*source).with_source_context(context)
        }
        BioMcpError::ApiJson { api, .. }
            if api == GWAS_API || api == crate::error::SourceProvider::GWAS.label() =>
        {
            BioMcpError::SourceUnavailable {
                source_name: "GWAS Catalog".to_string(),
                reason: "GWAS Catalog returned a response BioMCP could not decode.".to_string(),
                suggestion: "Retry later: biomcp get variant <id> gwas".to_string(),
            }
        }
        BioMcpError::Http(source) if source.is_timeout() || source.is_connect() => {
            BioMcpError::SourceUnavailable {
                source_name: "GWAS Catalog".to_string(),
                reason: "GWAS Catalog is temporarily unavailable.".to_string(),
                suggestion: "Retry later: biomcp get variant <id> gwas".to_string(),
            }
        }
        BioMcpError::Api { api, message }
            if (api == GWAS_API || api == crate::error::SourceProvider::GWAS.label())
                && gwas_status_is_transient(&message) =>
        {
            BioMcpError::SourceUnavailable {
                source_name: "GWAS Catalog".to_string(),
                reason: "GWAS Catalog is temporarily unavailable.".to_string(),
                suggestion: "Retry later: biomcp get variant <id> gwas".to_string(),
            }
        }
        other => other,
    }
}

fn gwas_status_is_transient(message: &str) -> bool {
    let Some(status) = message
        .strip_prefix("HTTP ")
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|code| code.parse::<u16>().ok())
    else {
        return false;
    };

    status == 408 || status == 429 || (500..=599).contains(&status)
}

fn normalize_rsid(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() || !normalized.starts_with("rs") {
        return Err(BioMcpError::InvalidArgument(
            "GWAS lookup requires an rsID (e.g., rs7903146).".into(),
        ));
    }
    if !normalized.chars().skip(2).all(|c| c.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid rsID: {value}"
        )));
    }
    Ok(normalized)
}

fn normalize_gene_symbol(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_ascii_uppercase();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Gene symbol is required. Example: biomcp search gwas -g TCF7L2".into(),
        ));
    }
    if !crate::sources::is_valid_gene_symbol(&normalized) {
        return Err(BioMcpError::InvalidArgument(format!(
            "Invalid gene symbol: {value}"
        )));
    }
    Ok(normalized)
}

fn normalize_trait_query(value: &str) -> Result<String, BioMcpError> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(BioMcpError::InvalidArgument(
            "Trait query is required. Example: biomcp search gwas --trait \"type 2 diabetes\""
                .into(),
        ));
    }
    if normalized.len() > 256 {
        return Err(BioMcpError::InvalidArgument(
            "Trait query is too long.".into(),
        ));
    }
    Ok(normalized)
}

fn de_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum NumberLike {
        Float(f64),
        Integer(i64),
        String(String),
    }

    let value = Option::<NumberLike>::deserialize(deserializer)?;
    Ok(match value {
        Some(NumberLike::Float(v)) => Some(v),
        Some(NumberLike::Integer(v)) => Some(v as f64),
        Some(NumberLike::String(v)) => v.trim().parse::<f64>().ok(),
        None => None,
    })
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasAssociationsResponse {
    #[serde(default, rename = "_embedded")]
    embedded: GwasAssociationsEmbedded,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct GwasAssociationsEmbedded {
    #[serde(default)]
    associations: Vec<GwasAssociation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasAssociation {
    #[serde(default)]
    pub snps: Vec<GwasSnp>,
    #[serde(default)]
    pub loci: Vec<GwasLocus>,
    #[serde(default, rename = "efoTraits")]
    pub efo_traits: Vec<GwasTrait>,
    #[serde(default)]
    pub study: Option<GwasStudy>,
    #[serde(default, deserialize_with = "de_opt_f64")]
    pub pvalue: Option<f64>,
    #[serde(default, rename = "orPerCopyNum", deserialize_with = "de_opt_f64")]
    pub or_per_copy_num: Option<f64>,
    #[serde(default, rename = "betaNum", deserialize_with = "de_opt_f64")]
    pub beta_num: Option<f64>,
    #[serde(default)]
    pub range: Option<String>,
    #[serde(default, rename = "riskFrequency", deserialize_with = "de_opt_f64")]
    pub risk_frequency: Option<f64>,
    #[serde(default)]
    // dead-code reason: gwas::description preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasSnp {
    #[serde(default, rename = "rsId")]
    pub rs_id: Option<String>,
    #[serde(default, rename = "genomicContexts")]
    // dead-code reason: gwas::genomic_contexts preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub genomic_contexts: Vec<GwasGenomicContext>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasGenomicContext {
    #[serde(default)]
    // dead-code reason: gwas::gene preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub gene: Option<GwasGene>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasLocus {
    #[serde(default, rename = "strongestRiskAlleles")]
    pub strongest_risk_alleles: Vec<GwasRiskAllele>,
    #[serde(default, rename = "authorReportedGenes")]
    pub author_reported_genes: Vec<GwasGene>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasRiskAllele {
    #[serde(default, rename = "riskAlleleName")]
    pub risk_allele_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasGene {
    #[serde(default, rename = "geneName")]
    pub gene_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasTrait {
    #[serde(default, rename = "trait")]
    pub trait_field: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasStudy {
    #[serde(default, rename = "accessionId")]
    pub accession_id: Option<String>,
    #[serde(default, rename = "diseaseTrait")]
    pub disease_trait: Option<GwasDiseaseTrait>,
    #[serde(default, rename = "initialSampleSize")]
    pub initial_sample_size: Option<String>,
    #[serde(default, rename = "replicationSampleSize")]
    pub replication_sample_size: Option<String>,
    #[serde(default, rename = "publicationInfo")]
    pub publication_info: Option<GwasPublicationInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasDiseaseTrait {
    #[serde(default, rename = "trait")]
    pub trait_field: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasPublicationInfo {
    #[serde(default, rename = "pubmedId")]
    pub pubmed_id: Option<String>,
    #[serde(default)]
    pub author: Option<GwasAuthor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GwasAuthor {
    #[serde(default, rename = "fullname")]
    pub fullname: Option<String>,
}

#[cfg(test)]
mod tests;
