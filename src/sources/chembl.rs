use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const CHEMBL_BASE: &str = "https://www.ebi.ac.uk/chembl/api/data";
const CHEMBL_BASE_ENV: &str = "BIOMCP_CHEMBL_BASE";

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

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
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
        Self::decode_json_response(status, &bytes).map_err(|error| {
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
        let resp: ChemblMechanismResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::targets_from_response(resp))
    }

    pub async fn target_summary(
        &self,
        target_chembl_id: &str,
    ) -> Result<ChemblTargetSummary, BioMcpError> {
        let plan = Self::target_summary_plan(target_chembl_id)?;
        let resp: ChemblTargetSummaryResponse = self
            .get_json(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await?;
        Ok(Self::summary_from_response(resp))
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
mod tests;
