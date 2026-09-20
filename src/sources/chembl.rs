use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CHEMBL_BASE: &str = "https://www.ebi.ac.uk/chembl/api/data";
const CHEMBL_BASE_ENV: &str = "BIOMCP_CHEMBL_BASE";
const CHEMBL_API: &str = "chembl";

pub struct ChemblClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl ChemblClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CHEMBL_BASE, CHEMBL_BASE_ENV),
        })
    }

    pub(crate) fn drug_targets_plan(
        chembl_id: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let chembl_id = chembl_id.trim();
        if chembl_id.is_empty() {
            return Err(BioMcpError::InvalidArgument("ChEMBL ID is required".into()));
        }
        Ok(RequestPlan::get("mechanism.json")
            .query("molecule_chembl_id", chembl_id)
            .query("limit", limit.clamp(1, 25).to_string()))
    }

    pub(crate) fn target_summary_plan(target_chembl_id: &str) -> Result<RequestPlan, BioMcpError> {
        let target_chembl_id = target_chembl_id.trim();
        if target_chembl_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "ChEMBL target ID is required".into(),
            ));
        }
        Ok(RequestPlan::get(format!("target/{target_chembl_id}.json")))
    }

    /// The cell line record that names one Cellosaurus accession. The join
    /// goes through `cellosaurus_id` alone, because ChEMBL spells several cell
    /// line names differently from Cellosaurus.
    pub(crate) fn cell_line_by_cellosaurus_plan(
        accession: &str,
    ) -> Result<RequestPlan, BioMcpError> {
        let accession = accession.trim();
        if accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Cellosaurus accession is required".into(),
            ));
        }
        Ok(RequestPlan::get("cell_line.json").query("cellosaurus_id", accession))
    }

    /// One assay row, asked for only so `page_meta.total_count` reports how many
    /// ChEMBL assays name the cell line.
    pub(crate) fn cell_line_assay_count_plan(chembl_id: &str) -> Result<RequestPlan, BioMcpError> {
        let chembl_id = chembl_id.trim();
        if chembl_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "ChEMBL cell line ID is required".into(),
            ));
        }
        Ok(RequestPlan::get("assay.json")
            .query("cell_chembl_id", chembl_id)
            .query("limit", "1"))
    }

    /// The ChEMBL release behind a cell line section.
    pub(crate) fn status_plan() -> RequestPlan {
        RequestPlan::get("status.json")
    }

    /// Decode one ChEMBL JSON body.
    ///
    /// An HTTP 200 carrying an HTML human-verification page where JSON was
    /// expected is a provider error that names the URL and stops the command.
    pub(crate) fn decode_body<T: DeserializeOwned>(
        url: &str,
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        if status.is_success() && looks_like_html(bytes) {
            return Err(BioMcpError::Api {
                api: CHEMBL_API.to_string(),
                message: format!(
                    "ChEMBL returned an HTML page where JSON was expected at {url}: {}",
                    crate::sources::body_excerpt(bytes)
                ),
            });
        }
        Self::decode_json_response(status, bytes)
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::CHEMBL),
            status,
            None,
            bytes,
            false,
        )
    }

    async fn get_plan<T: DeserializeOwned>(&self, plan: &RequestPlan) -> Result<T, BioMcpError> {
        let url = crate::sources::join_base_path(self.base.as_ref(), &plan.path);
        let req = request_from_plan(&self.client, self.base.as_ref(), plan);
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CHEMBL,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CHEMBL),
        )
        .await?;
        Self::decode_body(&url, status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CHEMBL,
            ))
        })
    }

    pub async fn drug_targets(
        &self,
        chembl_id: &str,
        limit: usize,
    ) -> Result<Vec<ChemblTarget>, BioMcpError> {
        let plan = Self::drug_targets_plan(chembl_id, limit)?;
        let resp: ChemblMechanismResponse = self.get_plan(&plan).await?;
        Ok(Self::targets_from_response(resp))
    }

    pub async fn target_summary(
        &self,
        target_chembl_id: &str,
    ) -> Result<ChemblTargetSummary, BioMcpError> {
        let plan = Self::target_summary_plan(target_chembl_id)?;
        let resp: ChemblTargetSummaryResponse = self.get_plan(&plan).await?;
        Ok(Self::summary_from_response(resp))
    }

    /// Every ChEMBL cell line record that names one Cellosaurus accession, in
    /// ChEMBL order. An empty list means ChEMBL lists no such cell line.
    pub(crate) async fn cell_line_by_cellosaurus(
        &self,
        accession: &str,
    ) -> Result<Vec<ChemblCellLine>, BioMcpError> {
        let plan = Self::cell_line_by_cellosaurus_plan(accession)?;
        let resp: ChemblCellLineResponse = self.get_plan(&plan).await?;
        Ok(Self::cell_lines_from_response(resp))
    }

    /// How many ChEMBL assays name one cell line.
    pub(crate) async fn cell_line_assay_count(&self, chembl_id: &str) -> Result<u64, BioMcpError> {
        let plan = Self::cell_line_assay_count_plan(chembl_id)?;
        let resp: ChemblPageResponse = self.get_plan(&plan).await?;
        Ok(Self::total_count_from_response(resp))
    }

    /// The ChEMBL release name and date.
    pub(crate) async fn status(&self) -> Result<ChemblRelease, BioMcpError> {
        let plan = Self::status_plan();
        let resp: ChemblStatusResponse = self.get_plan(&plan).await?;
        Self::release_from_response(resp)
    }

    fn targets_from_response(resp: ChemblMechanismResponse) -> Vec<ChemblTarget> {
        let mut out = Vec::new();
        for row in resp.mechanisms {
            let target = row
                .target_pref_name
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("Unknown target");
            let action = row
                .action_type
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("Mechanism");
            let mechanism = row
                .mechanism_of_action
                .as_deref()
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string);
            out.push(ChemblTarget {
                target: target.to_string(),
                action: action.to_string(),
                mechanism,
                target_chembl_id: row.target_chembl_id,
            });
        }
        out
    }

    /// Decode recorded bytes into cell line records, for fixture-backed tests.
    #[cfg(test)]
    pub(crate) fn cell_lines_from_bytes(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<Vec<ChemblCellLine>, BioMcpError> {
        Ok(Self::cell_lines_from_response(Self::decode_json_response(
            status, bytes,
        )?))
    }

    /// Decode recorded bytes into the release, for fixture-backed tests.
    #[cfg(test)]
    pub(crate) fn release_from_bytes(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<ChemblRelease, BioMcpError> {
        Self::release_from_response(Self::decode_json_response(status, bytes)?)
    }

    fn cell_lines_from_response(resp: ChemblCellLineResponse) -> Vec<ChemblCellLine> {
        resp.cell_lines
            .into_iter()
            .filter_map(|row| {
                let chembl_id = row.cell_chembl_id?;
                Some(ChemblCellLine {
                    chembl_id,
                    name: row.cell_name.unwrap_or_default(),
                    tax_id: row.cell_source_tax_id,
                    efo_id: row.efo_id,
                    clo_id: row.clo_id,
                })
            })
            .collect()
    }

    fn total_count_from_response(resp: ChemblPageResponse) -> u64 {
        resp.page_meta
            .and_then(|meta| meta.total_count)
            .unwrap_or_default()
    }

    fn release_from_response(resp: ChemblStatusResponse) -> Result<ChemblRelease, BioMcpError> {
        match (resp.chembl_db_version, resp.chembl_release_date) {
            (Some(version), Some(released)) if !version.trim().is_empty() => Ok(ChemblRelease {
                version: version.trim().to_string(),
                released: released.trim().to_string(),
            }),
            _ => Err(BioMcpError::Api {
                api: CHEMBL_API.to_string(),
                message: "ChEMBL status carried no release name and date".to_string(),
            }),
        }
    }

    fn summary_from_response(resp: ChemblTargetSummaryResponse) -> ChemblTargetSummary {
        ChemblTargetSummary {
            pref_name: resp.pref_name.unwrap_or_default().trim().to_string(),
            target_type: resp.target_type.unwrap_or_default().trim().to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblMechanismResponse {
    #[serde(default)]
    mechanisms: Vec<ChemblMechanism>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblMechanism {
    target_pref_name: Option<String>,
    action_type: Option<String>,
    mechanism_of_action: Option<String>,
    target_chembl_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblTargetSummaryResponse {
    pref_name: Option<String>,
    target_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblCellLineResponse {
    #[serde(default)]
    cell_lines: Vec<ChemblCellLineRow>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblCellLineRow {
    cell_chembl_id: Option<String>,
    cell_name: Option<String>,
    cell_source_tax_id: Option<i64>,
    efo_id: Option<String>,
    clo_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblPageResponse {
    #[serde(default)]
    page_meta: Option<ChemblPageMeta>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblPageMeta {
    #[serde(default)]
    total_count: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ChemblStatusResponse {
    chembl_db_version: Option<String>,
    chembl_release_date: Option<String>,
}

/// One ChEMBL cell line record, as ChEMBL published it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChemblCellLine {
    pub chembl_id: String,
    pub name: String,
    pub tax_id: Option<i64>,
    pub efo_id: Option<String>,
    pub clo_id: Option<String>,
}

/// The ChEMBL release behind a cell line section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ChemblRelease {
    pub version: String,
    pub released: String,
}

/// An HTTP 200 body that is an HTML page where JSON was expected.
fn looks_like_html(bytes: &[u8]) -> bool {
    let sniff = String::from_utf8_lossy(&bytes[..bytes.len().min(128)])
        .trim_start()
        .to_ascii_lowercase();
    sniff.starts_with("<!doctype html") || sniff.starts_with("<html")
}

#[derive(Debug, Clone)]
pub struct ChemblTarget {
    pub target: String,
    pub action: String,
    pub mechanism: Option<String>,
    pub target_chembl_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChemblTargetSummary {
    pub pref_name: String,
    pub target_type: String,
}

#[cfg(test)]
mod tests {
    mod construction;
    mod parsing;
}
