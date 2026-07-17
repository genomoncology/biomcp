use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, is_valid_gene_symbol, request_from_plan};
use crate::utils::serde::StringOrVec;

const MYGENE_BASE: &str = "https://mygene.info/v3";
const MYGENE_API: &str = "mygene.info";
const MYGENE_BASE_ENV: &str = "BIOMCP_MYGENE_BASE";
const MYGENE_MAX_RESULT_WINDOW: usize = 10_000;
const MYGENE_BATCH_GENE_LIMIT: usize = 200;

pub struct MyGeneClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

impl MyGeneClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(MYGENE_BASE, MYGENE_BASE_ENV),
        })
    }

    pub(crate) fn escape_query_value(value: &str) -> String {
        crate::utils::query::escape_lucene_value(value)
    }

    fn validate_search_window(limit: usize, offset: usize) -> Result<(), BioMcpError> {
        if offset >= MYGENE_MAX_RESULT_WINDOW {
            return Err(BioMcpError::InvalidArgument(format!(
                "--offset must be less than {MYGENE_MAX_RESULT_WINDOW} for MyGene search"
            )));
        }

        if offset.saturating_add(limit) > MYGENE_MAX_RESULT_WINDOW {
            return Err(BioMcpError::InvalidArgument(format!(
                "--offset + --limit must be <= {MYGENE_MAX_RESULT_WINDOW} for MyGene search"
            )));
        }

        Ok(())
    }

    async fn get_json<T: DeserializeOwned>(
        &self,
        req: reqwest_middleware::RequestBuilder,
    ) -> Result<T, BioMcpError> {
        let resp = crate::sources::apply_cache_mode(req).send().await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_body(resp, MYGENE_API).await?;
        crate::sources::decode_json(MYGENE_API, status, content_type.as_ref(), &bytes, true)
    }

    /// Build the outbound search request (pure — Tier-2 testable, never sent).
    pub(crate) fn search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        chromosome: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        Self::validate_search_window(limit, offset)?;
        let mut plan = RequestPlan::get("query")
            .query("q", query)
            .query("species", "human")
            .query(
                "fields",
                "symbol,name,entrezgene,alias,type_of_gene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,MIM,uniprot,pathway.kegg.id,pathway.reactome.id,go.BP.id,go.CC.id,go.MF.id",
            )
            .query("size", limit.to_string())
            .query("from", offset.to_string());

        if let Some(chr) = chromosome.map(str::trim).filter(|v| !v.is_empty()) {
            // MyGene supports `chr` query param filtering for `/query`.
            plan = plan.query("chr", chr);
        }
        Ok(plan)
    }

    /// Search genes by query
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        chromosome: Option<&str>,
    ) -> Result<MyGeneSearchResponse, BioMcpError> {
        let plan = Self::search_plan(query, limit, offset, chromosome)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        self.get_json(req).await
    }

    /// Build the outbound single-gene query request (pure — Tier-2 testable).
    pub(crate) fn get_plan(
        symbol: &str,
        include_transcripts: bool,
    ) -> Result<RequestPlan, BioMcpError> {
        let symbol = symbol.trim();
        if symbol.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Gene symbol is required. Example: biomcp get gene BRAF".into(),
            ));
        }
        if symbol.len() > 128 {
            return Err(BioMcpError::InvalidArgument(
                "Gene symbol is too long. Example: biomcp get gene BRAF".into(),
            ));
        }
        if !is_valid_gene_symbol(symbol) {
            return Err(BioMcpError::InvalidArgument(
                "Gene symbol must contain only letters, numbers, '_' or '-'. Example: biomcp get gene BRAF".into(),
            ));
        }

        let fields = if include_transcripts {
            "symbol,name,summary,alias,type_of_gene,ensembl.gene,ensembl.transcript,ensembl.protein,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg"
        } else {
            "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg"
        };

        let q = format!("symbol:\"{}\"", Self::escape_query_value(symbol));
        Ok(RequestPlan::get("query")
            .query("q", q)
            .query("species", "human")
            .query("fields", fields)
            .query("size", "1"))
    }

    /// Get gene by symbol (single query for fields needed by the caller)
    pub async fn get(
        &self,
        symbol: &str,
        include_transcripts: bool,
    ) -> Result<MyGeneGetResponse, BioMcpError> {
        let symbol = symbol.trim();
        let plan = Self::get_plan(symbol, include_transcripts)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let query_resp: MyGeneGetQueryResponse = self.get_json(req).await?;

        query_resp
            .hits
            .into_iter()
            .next()
            .ok_or_else(|| BioMcpError::NotFound {
                entity: "gene".into(),
                id: symbol.into(),
                suggestion: format!("Try searching: biomcp search gene -q {symbol}"),
            })
    }

    pub async fn resolve_uniprot_accession(&self, symbol: &str) -> Result<String, BioMcpError> {
        let symbol = symbol.trim();
        let hit = self.get(symbol, false).await?;
        hit.uniprot
            .as_ref()
            .and_then(extract_uniprot_accession)
            .ok_or_else(|| BioMcpError::NotFound {
                entity: "protein".into(),
                id: symbol.to_string(),
                suggestion: format!(
                    "No UniProt accession found for {symbol}. Try: biomcp search protein -q {symbol}"
                ),
            })
    }

    /// Build the outbound batch-symbol request and return the cleaned id list (pure).
    pub(crate) fn batch_symbols_plan(
        ids: &[String],
    ) -> Result<(RequestPlan, Vec<String>), BioMcpError> {
        let ids = ids
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "MyGene Entrez ID batch must include at least one ID".into(),
            ));
        }
        if ids.len() > MYGENE_BATCH_GENE_LIMIT {
            return Err(BioMcpError::InvalidArgument(format!(
                "MyGene Entrez ID batch supports at most {MYGENE_BATCH_GENE_LIMIT} IDs per request"
            )));
        }

        let ids_csv = ids.join(",");
        let plan = RequestPlan::post("gene").form(vec![
            ("ids".to_string(), ids_csv),
            ("fields".to_string(), "symbol".to_string()),
            ("species".to_string(), "human".to_string()),
        ]);
        Ok((plan, ids))
    }

    /// Map batch rows back to input order with de-duplicated symbols (pure — Tier-3).
    pub(crate) fn dedupe_symbols_in_order(
        rows: Vec<MyGeneBatchGeneHit>,
        ids: &[String],
    ) -> Vec<String> {
        let mut symbol_by_id = HashMap::new();
        for row in rows {
            let symbol = row
                .symbol
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let key = row
                .query
                .or(row.id)
                .map(|value| value.as_string())
                .filter(|value| !value.is_empty());
            let (Some(symbol), Some(key)) = (symbol, key) else {
                continue;
            };
            symbol_by_id.entry(key).or_insert(symbol);
        }

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        for id in ids {
            let Some(symbol) = symbol_by_id.get(id.as_str()) else {
                continue;
            };
            if !seen.insert(symbol.clone()) {
                continue;
            }
            out.push(symbol.clone());
        }
        out
    }

    pub async fn symbols_for_entrez_ids(&self, ids: &[String]) -> Result<Vec<String>, BioMcpError> {
        let (plan, ids) = Self::batch_symbols_plan(ids)?;
        let req = request_from_plan(&self.client, self.base.as_ref(), &plan);
        let rows: Vec<MyGeneBatchGeneHit> = self.get_json(req).await?;
        Ok(Self::dedupe_symbols_in_order(rows, &ids))
    }
}

fn first_string_value(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => {
            let s = s.trim();
            (!s.is_empty()).then(|| s.to_string())
        }
        serde_json::Value::Array(values) => values.iter().find_map(first_string_value),
        serde_json::Value::Object(values) => {
            if let Some(id) = values.get("id").and_then(first_string_value) {
                return Some(id);
            }
            values.values().find_map(first_string_value)
        }
        _ => None,
    }
}

pub(crate) fn extract_uniprot_accession(value: &serde_json::Value) -> Option<String> {
    if let Some(obj) = value.as_object() {
        if let Some(swiss_prot) = obj.get("Swiss-Prot").and_then(first_string_value) {
            return Some(swiss_prot);
        }
        if let Some(swiss_prot) = obj.get("SwissProt").and_then(first_string_value) {
            return Some(swiss_prot);
        }
        if let Some(trembl) = obj.get("TrEMBL").and_then(first_string_value) {
            return Some(trembl);
        }
    }

    first_string_value(value)
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyGeneSearchResponse {
    // dead-code reason: mygene::total preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyGeneHit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyGeneGetQueryResponse {
    // dead-code reason: mygene::total preserves the provider shape used by source contract fixtures
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyGeneGetResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyGeneHit {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub entrezgene: Option<StringOrU64>,
    #[serde(default)]
    pub alias: StringOrVec,
    pub type_of_gene: Option<String>,
    pub genomic_pos: Option<GenomicPosField>,
    #[serde(rename = "MIM")]
    pub mim: Option<serde_json::Value>,
    pub uniprot: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyGeneGetResponse {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub entrezgene: Option<StringOrU64>,
    pub summary: Option<String>,
    #[serde(default)]
    pub alias: StringOrVec,
    pub type_of_gene: Option<String>,
    pub ensembl: Option<EnsemblField>,
    pub genomic_pos: Option<GenomicPosField>,
    #[serde(rename = "MIM")]
    pub mim: Option<serde_json::Value>,
    pub uniprot: Option<serde_json::Value>,
    pub pathway: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct MyGeneBatchGeneHit {
    query: Option<StringOrU64>,
    #[serde(rename = "_id")]
    id: Option<StringOrU64>,
    symbol: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StringOrU64 {
    String(String),
    Number(u64),
}

impl StringOrU64 {
    pub fn as_string(&self) -> String {
        match self {
            StringOrU64::String(s) => s.clone(),
            StringOrU64::Number(n) => n.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EnsemblInfo {
    pub gene: Option<String>,
    pub protein: Option<Vec<String>>,
    pub transcript: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EnsemblField {
    Single(EnsemblInfo),
    Multiple(Vec<EnsemblInfo>),
}

impl EnsemblField {
    fn first(&self) -> Option<&EnsemblInfo> {
        match self {
            EnsemblField::Single(v) => Some(v),
            EnsemblField::Multiple(v) => v.first(),
        }
    }

    pub fn gene(&self) -> Option<&String> {
        self.first().and_then(|v| v.gene.as_ref())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GenomicPos {
    pub chr: Option<String>,
    pub start: Option<i64>,
    pub end: Option<i64>,
    pub strand: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum GenomicPosField {
    Single(GenomicPos),
    Multiple(Vec<GenomicPos>),
}

impl GenomicPosField {
    fn first(&self) -> Option<&GenomicPos> {
        match self {
            GenomicPosField::Single(v) => Some(v),
            GenomicPosField::Multiple(v) => v.first(),
        }
    }

    pub fn chr(&self) -> Option<&String> {
        self.first().and_then(|v| v.chr.as_ref())
    }

    pub fn start(&self) -> Option<i64> {
        self.first().and_then(|v| v.start)
    }

    pub fn end(&self) -> Option<i64> {
        self.first().and_then(|v| v.end)
    }

    pub fn strand(&self) -> Option<i32> {
        self.first().and_then(|v| v.strand)
    }
}

#[cfg(test)]
mod tests;
