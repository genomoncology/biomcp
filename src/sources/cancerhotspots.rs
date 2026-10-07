use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CANCERHOTSPOTS_BASE: &str = "https://www.cancerhotspots.org";
const CANCERHOTSPOTS_API: &str = "cancerhotspots.org";
const CANCERHOTSPOTS_BASE_ENV: &str = "BIOMCP_CANCERHOTSPOTS_BASE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancerHotspotSection {
    pub source: String,
    #[serde(
        flatten,
        serialize_with = "serialize_recurrence",
        deserialize_with = "biodata::CancerHotspotRecurrenceProjection::deserialize_recurrence_target"
    )]
    recurrence: biodata::CancerHotspotRecurrenceProjection,
}

impl CancerHotspotSection {
    pub(crate) fn new(recurrence: biodata::CancerHotspotRecurrenceProjection) -> Self {
        Self {
            source: CANCERHOTSPOTS_API.to_owned(),
            recurrence,
        }
    }

    pub fn recurrence(&self) -> &biodata::CancerHotspotRecurrenceProjection {
        &self.recurrence
    }
}

fn serialize_recurrence<S: serde::Serializer>(
    recurrence: &biodata::CancerHotspotRecurrenceProjection,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    recurrence.as_recurrence_target().serialize(serializer)
}

pub struct CancerHotspotsClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl CancerHotspotsClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CANCERHOTSPOTS_BASE, CANCERHOTSPOTS_BASE_ENV),
        })
    }

    pub(crate) fn by_gene_plan(gene: &str) -> RequestPlan {
        RequestPlan::get(format!(
            "api/hotspots/single/byGene/{}",
            encode_path_segment(gene.trim())
        ))
    }

    pub(crate) fn decode_by_gene_response(
        status: StatusCode,
        content_type: Option<&HeaderValue>,
        bytes: &[u8],
    ) -> Result<biodata::CancerHotspotsResponse, BioMcpError> {
        if !status.is_success() {
            let excerpt = crate::sources::summarize_http_error_body(content_type, bytes);
            return Err(BioMcpError::Api {
                api: CANCERHOTSPOTS_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CANCER_HOTSPOTS),
            content_type,
            bytes,
        )?;
        biodata::CancerHotspotsResponse::decode_json(bytes).map_err(|_| BioMcpError::Api {
            api: CANCERHOTSPOTS_API.to_owned(),
            message: "Invalid Cancer Hotspots response.".to_owned(),
        })
    }

    pub async fn by_gene(
        &self,
        gene: &str,
    ) -> Result<biodata::CancerHotspotsResponse, BioMcpError> {
        let plan = Self::by_gene_plan(gene);
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CANCER_HOTSPOTS,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CANCER_HOTSPOTS),
        )
        .await?;
        Self::decode_by_gene_response(status, content_type.as_ref(), &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CANCER_HOTSPOTS,
            ))
        })
    }
}

fn encode_path_segment(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    mod construction;
    mod parsing;
}
