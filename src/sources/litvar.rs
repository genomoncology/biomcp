//! NCBI LitVar2 client: variant-to-literature lookup for article search.
//!
//! LitVar2 links variants to the papers they appear in. The autocomplete
//! endpoint resolves a gene-plus-protein-change or rsID query to one LitVar2
//! variant id (`litvar@<rsid>##`), and the publications endpoint lists the
//! PMIDs linked to that variant. The client sends no key.

use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;

use reqwest_middleware::ClientWithMiddleware;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, request_from_plan};

const LITVAR_BASE: &str = "https://www.ncbi.nlm.nih.gov/research/litvar2-api";
const LITVAR_BASE_ENV: &str = "BIOMCP_LITVAR_BASE";

#[derive(Clone)]
pub struct LitVarClient {
    client: ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl LitVarClient {
    pub(crate) async fn new_with_deadline(
        deadline: &crate::sources::VariantArticleDeadline,
    ) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client_with_deadline("litvar2", deadline).await?,
            base: crate::sources::env_base(LITVAR_BASE, LITVAR_BASE_ENV),
        })
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::LITVAR2,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::LITVAR2),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::LITVAR2),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
    }

    pub(crate) fn autocomplete_plan(query: &str) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Query is required for LitVar2 autocomplete".into(),
            ));
        }
        if query.len() > 256 {
            return Err(BioMcpError::InvalidArgument(
                "Query is too long for LitVar2 autocomplete".into(),
            ));
        }
        Ok(RequestPlan::get("variant/autocomplete/").query("query", query))
    }

    pub async fn autocomplete(
        &self,
        query: &str,
    ) -> Result<Vec<LitVarAutocompleteEntry>, BioMcpError> {
        let plan = Self::autocomplete_plan(query)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    pub(crate) fn publications_plan(variant_id: &str) -> Result<RequestPlan, BioMcpError> {
        let variant_id = variant_id.trim();
        if variant_id.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Variant id is required for LitVar2 publications".into(),
            ));
        }
        if variant_id.len() > 256 {
            return Err(BioMcpError::InvalidArgument(
                "Variant id is too long for LitVar2 publications".into(),
            ));
        }
        // LitVar2 variant ids carry `@` and `#` (for example
        // `litvar@rs113488022##`); a raw `#` would read as a URL fragment,
        // so the id travels percent-encoded, as the provider's own examples
        // spell it.
        Ok(RequestPlan::get(format!(
            "variant/get/{}/publications",
            encode_path_segment(variant_id)
        )))
    }

    pub async fn publications(&self, variant_id: &str) -> Result<LitVarPublications, BioMcpError> {
        let plan = Self::publications_plan(variant_id)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char)
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

/// One LitVar2 autocomplete row: the variant id, the rsID it normalizes to,
/// the genes and protein or coding change it names, and how many papers
/// LitVar2 links to it.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LitVarAutocompleteEntry {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    #[serde(default)]
    pub rsid: Option<String>,
    #[serde(default)]
    pub gene: Vec<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub hgvs: Option<String>,
    #[serde(default)]
    pub pmids_count: Option<u64>,
}

impl LitVarAutocompleteEntry {
    /// The variant's own identity as the existing variant identity
    /// comparison sees it: genes from `gene`, the protein or coding change
    /// from `name` (LitVar2 repeats it in `hgvs`), and the rsID it
    /// normalizes to. An entry without a usable id resolves no publications.
    pub(crate) fn source_identity(
        &self,
    ) -> Option<crate::entities::variant::SourceVariantIdentity> {
        let id = self
            .id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())?;
        let mut identity = crate::entities::variant::SourceVariantIdentity {
            genomic_id: id.to_string(),
            genome_build: "unknown".into(),
            genes: self
                .gene
                .iter()
                .map(|gene| gene.trim().to_string())
                .filter(|gene| !gene.is_empty())
                .collect(),
            protein_changes: Vec::new(),
            coding_changes: Vec::new(),
            rsids: self
                .rsid
                .as_deref()
                .map(|rsid| rsid.trim())
                .filter(|rsid| crate::entities::variant::is_rsid(rsid))
                .map(|rsid| vec![rsid.to_string()])
                .unwrap_or_default(),
        };
        for change in [self.name.as_deref(), self.hgvs.as_deref()]
            .into_iter()
            .flatten()
            .map(|change| change.trim())
            .filter(|change| !change.is_empty())
        {
            if crate::entities::variant::normalize_protein_change(change).is_some() {
                if !identity.protein_changes.iter().any(|kept| kept == change) {
                    identity.protein_changes.push(change.to_string());
                }
            } else if crate::entities::variant::coding_change_segment(change)
                .to_ascii_lowercase()
                .starts_with("c.")
                && !identity.coding_changes.iter().any(|kept| kept == change)
            {
                identity.coding_changes.push(change.to_string());
            }
        }
        Some(identity)
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct LitVarPublications {
    #[serde(default)]
    pub pmids: Vec<u64>,
    /// PMCIDs LitVar2 links to the variant; the provider sends them as one
    /// unkeyed list, so they carry no PMID correspondence and stay unused
    /// for row merging.
    #[serde(default)]
    pub pmcids: Vec<String>,
    #[serde(default)]
    pub pmids_count: Option<u64>,
}

#[cfg(test)]
mod tests;
