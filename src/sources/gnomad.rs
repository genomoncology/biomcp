use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestBody, RequestPlan, request_from_plan};

pub(crate) const GNOMAD_BASE: &str = "https://gnomad.broadinstitute.org/api";
pub(crate) const GNOMAD_API: &str = "gnomAD";
pub(crate) const GNOMAD_BASE_ENV: &str = "BIOMCP_GNOMAD_BASE";
pub(crate) const GNOMAD_CONSTRAINT_VERSION: &str = "v4";
pub(crate) const GNOMAD_CONSTRAINT_REFERENCE_GENOME: &str = "GRCh38";
pub(crate) const GNOMAD_VARIANT_MAX_BODY_BYTES: usize = 512 * 1024;
const GENE_CONSTRAINT_QUERY: &str = r#"
query GeneConstraint($symbol: String!) {
  gene(gene_symbol: $symbol, reference_genome: GRCh38) {
    canonical_transcript_id
    gnomad_constraint {
      pLI
      oe_lof_upper
      mis_z
      syn_z
    }
  }
}
"#;
const VARIANT_POPULATION_QUERY: &str = r#"
query VariantPopulation($variantId: String!) {
  variant(variantId: $variantId, dataset: gnomad_r4) {
    variant_id
    exome { ac an homozygote_count hemizygote_count filters faf95 { popmax popmax_population } populations { id ac an homozygote_count hemizygote_count } }
    genome { ac an homozygote_count hemizygote_count filters faf95 { popmax popmax_population } populations { id ac an homozygote_count hemizygote_count } }
  }
}
"#;

pub struct GnomadClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GnomadConstraintData {
    pub pli: Option<f64>,
    pub loeuf: Option<f64>,
    pub mis_z: Option<f64>,
    pub syn_z: Option<f64>,
    pub transcript: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GraphQlResponse<T> {
    data: Option<T>,
    #[serde(default)]
    errors: Option<Vec<GraphQlError>>,
}

#[derive(Debug, Deserialize)]
struct GraphQlError {
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GeneConstraintResponse {
    gene: Option<GeneConstraintGene>,
}

#[derive(Debug, Deserialize)]
struct GeneConstraintGene {
    canonical_transcript_id: Option<String>,
    gnomad_constraint: Option<ConstraintPayload>,
}

#[derive(Debug, Deserialize)]
struct ConstraintPayload {
    #[serde(rename = "pLI", alias = "pli")]
    pli: Option<f64>,
    #[serde(rename = "oe_lof_upper")]
    oe_lof_upper: Option<f64>,
    mis_z: Option<f64>,
    syn_z: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GnomadVariantPopulation {
    pub variant_id: String,
    pub exome: Option<GnomadSequencingPopulation>,
    pub genome: Option<GnomadSequencingPopulation>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct GnomadSequencingPopulation {
    #[serde(default)]
    pub allele_frequency: Option<f64>,
    pub ac: u64,
    pub an: u64,
    pub homozygote_count: u64,
    pub hemizygote_count: u64,
    pub filters: Vec<String>,
    pub faf95: Option<GnomadFaf95>,
    pub populations: Vec<GnomadAncestryPopulation>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct GnomadFaf95 {
    pub popmax: Option<f64>,
    pub popmax_population: Option<String>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct GnomadAncestryPopulation {
    pub id: String,
    #[serde(default)]
    pub allele_frequency: Option<f64>,
    pub ac: u64,
    pub an: u64,
    pub homozygote_count: u64,
    pub hemizygote_count: u64,
}

#[derive(Debug, Deserialize)]
struct VariantPopulationResponse {
    variant: Option<GnomadVariantPopulation>,
}

impl GnomadClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(GNOMAD_BASE, GNOMAD_BASE_ENV),
        })
    }

    pub(crate) fn gene_constraint_plan(symbol: &str) -> Result<RequestPlan, BioMcpError> {
        let symbol = symbol.trim();
        if !crate::sources::is_valid_gene_symbol(symbol) {
            return Err(BioMcpError::InvalidArgument(
                "gnomAD requires a valid gene symbol".into(),
            ));
        }

        let mut plan = RequestPlan::post("");
        plan.body = RequestBody::Json(serde_json::json!({
            "query": GENE_CONSTRAINT_QUERY,
            "variables": { "symbol": symbol },
        }));
        Ok(plan)
    }

    pub(crate) fn variant_population_plan(variant_id: &str) -> Result<RequestPlan, BioMcpError> {
        let variant_id = variant_id.trim();
        if variant_id.is_empty() || variant_id.chars().count() > 256 {
            return Err(BioMcpError::InvalidArgument(
                "gnomAD variant ID is invalid".into(),
            ));
        }
        Ok(RequestPlan::post("").json(serde_json::json!({
            "query": VARIANT_POPULATION_QUERY,
            "variables": { "variantId": variant_id },
        })))
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::GNOMAD),
            status,
            content_type,
            bytes,
            true,
        )
    }

    async fn send_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GNOMAD,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::GNOMAD),
        )
        .await?;
        Self::decode_json_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GNOMAD,
            ))
        })
    }

    fn parse_gene_constraint_response(
        resp: GraphQlResponse<GeneConstraintResponse>,
    ) -> Result<Option<GnomadConstraintData>, BioMcpError> {
        let errors = resp.errors.unwrap_or_default();
        let gene = resp.data.and_then(|data| data.gene);

        if !errors.is_empty() {
            let messages = errors
                .iter()
                .filter_map(|error| error.message.as_deref())
                .map(str::trim)
                .filter(|message| !message.is_empty())
                .collect::<Vec<_>>();

            if gene.is_none()
                && !messages.is_empty()
                && messages
                    .iter()
                    .all(|message| message.eq_ignore_ascii_case("Gene not found"))
            {
                return Ok(None);
            }

            let message = if messages.is_empty() {
                "GraphQL request failed".to_string()
            } else {
                messages.join("; ")
            };

            return Err(BioMcpError::Api {
                api: GNOMAD_API.to_string(),
                message,
            });
        }

        let Some(gene) = gene else {
            return Ok(None);
        };

        let transcript = gene
            .canonical_transcript_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        let Some(metrics) = gene.gnomad_constraint else {
            return Ok(Some(GnomadConstraintData {
                pli: None,
                loeuf: None,
                mis_z: None,
                syn_z: None,
                transcript,
            }));
        };

        Ok(Some(GnomadConstraintData {
            pli: metrics.pli,
            loeuf: metrics.oe_lof_upper,
            mis_z: metrics.mis_z,
            syn_z: metrics.syn_z,
            transcript,
        }))
    }

    fn parse_variant_population_response(
        resp: GraphQlResponse<VariantPopulationResponse>,
    ) -> Result<Option<GnomadVariantPopulation>, BioMcpError> {
        if let Some(errors) = resp.errors.filter(|errors| !errors.is_empty()) {
            let messages = errors
                .into_iter()
                .filter_map(|error| error.message)
                .map(|message| message.trim().to_string())
                .filter(|message| !message.is_empty())
                .collect::<Vec<_>>();
            if resp.data.as_ref().is_none_or(|data| data.variant.is_none())
                && !messages.is_empty()
                && messages
                    .iter()
                    .all(|message| message.eq_ignore_ascii_case("Variant not found"))
            {
                return Ok(None);
            }
            let message = messages.join("; ");
            return Err(BioMcpError::Api {
                api: GNOMAD_API.into(),
                message: if message.is_empty() {
                    "GraphQL request failed".into()
                } else {
                    message
                },
            });
        }

        let mut variant = resp.data.and_then(|data| data.variant);
        if let Some(variant) = variant.as_mut() {
            for sequencing in [&mut variant.exome, &mut variant.genome]
                .into_iter()
                .flatten()
            {
                sequencing.allele_frequency = frequency(sequencing.ac, sequencing.an);
                for ancestry in &mut sequencing.populations {
                    ancestry.allele_frequency = frequency(ancestry.ac, ancestry.an);
                }
            }
        }
        Ok(variant)
    }

    pub async fn gene_constraint(
        &self,
        symbol: &str,
    ) -> Result<Option<GnomadConstraintData>, BioMcpError> {
        let plan = Self::gene_constraint_plan(symbol)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let resp: GraphQlResponse<GeneConstraintResponse> = self.send_json(req).await?;
        Self::parse_gene_constraint_response(resp).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GNOMAD,
            ))
        })
    }

    pub async fn variant_population(
        &self,
        variant_id: &str,
    ) -> Result<Option<GnomadVariantPopulation>, BioMcpError> {
        let plan = Self::variant_population_plan(variant_id)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let response = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::GNOMAD,
            ))
            .await?;
        let status = response.status();
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned();
        let bytes = crate::sources::read_limited_source_body_with_limit(
            response,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::GNOMAD),
            GNOMAD_VARIANT_MAX_BODY_BYTES,
        )
        .await?;
        let response = Self::decode_json_response(status, content_type.as_ref(), &bytes)?;
        Self::parse_variant_population_response(response)
    }
}

fn frequency(ac: u64, an: u64) -> Option<f64> {
    (an > 0).then_some(ac as f64 / an as f64)
}

#[cfg(test)]
mod tests;
