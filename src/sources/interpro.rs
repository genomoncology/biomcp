use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use biodata::InterProResponse;

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const INTERPRO_BASE: &str = "https://www.ebi.ac.uk/interpro/api";
const INTERPRO_API: &str = "interpro";
const INTERPRO_BASE_ENV: &str = "BIOMCP_INTERPRO_BASE";

pub struct InterProClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl InterProClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(INTERPRO_BASE, INTERPRO_BASE_ENV),
        })
    }

    async fn get_response(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<InterProResponse, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::INTERPRO,
            ))
            .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::INTERPRO),
        )
        .await?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: INTERPRO_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::INTERPRO,
            )));
        }
        InterProResponse::decode_json(&bytes).map_err(|_| {
            BioMcpError::Api {
                api: INTERPRO_API.to_string(),
                message: "Invalid InterPro response.".to_string(),
            }
            .with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::INTERPRO,
            ))
        })
    }

    pub(crate) fn domains_plan(
        uniprot_accession: &str,
        limit: usize,
    ) -> Result<RequestPlan, BioMcpError> {
        let uniprot_accession = uniprot_accession.trim();
        if uniprot_accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "InterPro requires a UniProt accession".into(),
            ));
        }

        let page_size = limit.clamp(1, 25).to_string();
        Ok(RequestPlan::get(format!(
            "entry/interpro/protein/uniprot/{uniprot_accession}/"
        ))
        .query("page_size", page_size))
    }

    pub async fn domains(
        &self,
        uniprot_accession: &str,
        limit: usize,
    ) -> Result<InterProResponse, BioMcpError> {
        let plan = Self::domains_plan(uniprot_accession, limit)?;
        self.get_response(request_from_plan(&self.client, self.base.as_ref(), &plan))
            .await
    }
}

#[cfg(test)]
mod tests;
