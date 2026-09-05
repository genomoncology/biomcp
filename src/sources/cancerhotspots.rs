use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::HashMap;

use reqwest::StatusCode;
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CANCERHOTSPOTS_BASE: &str = "https://www.cancerhotspots.org";
const CANCERHOTSPOTS_API: &str = "cancerhotspots.org";
const CANCERHOTSPOTS_BASE_ENV: &str = "BIOMCP_CANCERHOTSPOTS_BASE";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancerHotspotRecurrence {
    pub source: String,
    pub position_count: Option<u32>,
    pub same_aa_count: Option<u32>,
    pub matched_transcript: Option<String>,
}

impl CancerHotspotRecurrence {
    fn checked_absent() -> Self {
        Self {
            source: CANCERHOTSPOTS_API.to_string(),
            position_count: None,
            same_aa_count: None,
            matched_transcript: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancerHotspotRow {
    // dead-code reason: cancerhotspots::hugo_symbol preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub hugo_symbol: Option<String>,
    pub residue: Option<String>,
    pub tumor_count: Option<u32>,
    pub transcript_id: Option<String>,
    // dead-code reason: cancerhotspots::amino_acid_position preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub amino_acid_position: Option<serde_json::Value>,
    #[serde(default)]
    pub variant_amino_acid: HashMap<String, u32>,
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
    ) -> Result<Vec<CancerHotspotRow>, BioMcpError> {
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
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: CANCERHOTSPOTS_API.to_string(),
            source,
        })
    }

    pub async fn by_gene(&self, gene: &str) -> Result<Vec<CancerHotspotRow>, BioMcpError> {
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

pub(crate) fn recurrence_for_change(
    rows: &[CancerHotspotRow],
    normalized_change: &str,
) -> CancerHotspotRecurrence {
    let Some((requested_residue, requested_alt)) = residue_and_alt(normalized_change) else {
        return CancerHotspotRecurrence::checked_absent();
    };

    rows.iter()
        .filter(|row| {
            row.residue
                .as_deref()
                .map(normalize_residue)
                .is_some_and(|residue| residue == requested_residue)
        })
        .find_map(|row| {
            let same_aa_count = same_aa_count(row, &requested_alt)?;
            Some(CancerHotspotRecurrence {
                source: CANCERHOTSPOTS_API.to_string(),
                position_count: row.tumor_count,
                same_aa_count: Some(same_aa_count),
                matched_transcript: row
                    .transcript_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(str::to_string),
            })
        })
        .unwrap_or_else(CancerHotspotRecurrence::checked_absent)
}

fn same_aa_count(row: &CancerHotspotRow, requested_alt: &str) -> Option<u32> {
    row.variant_amino_acid
        .get(requested_alt)
        .copied()
        .or_else(|| {
            row.variant_amino_acid
                .get(&requested_alt.to_ascii_uppercase())
                .copied()
        })
}

fn normalize_residue(value: &str) -> String {
    value.trim().to_ascii_uppercase()
}

fn residue_and_alt(normalized_change: &str) -> Option<(String, String)> {
    let change = normalized_change.trim().trim_start_matches("p.");
    let alt = change.chars().last()?;
    if !(alt.is_ascii_alphabetic() || alt == '*') {
        return None;
    }
    let residue = &change[..change.len() - alt.len_utf8()];
    if residue.is_empty() || !residue.chars().any(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some((normalize_residue(residue), alt.to_string()))
}

#[cfg(test)]
mod tests {
    mod construction;
    mod parsing;
}
