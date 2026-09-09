use crate::sources::RequestBuilderSourceContextExt;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::{RequestPlan, is_valid_gene_symbol, request_from_plan};
use crate::utils::serde::StringOrVec;

const MYGENE_BASE: &str = "https://mygene.info/v3";
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
        let resp = crate::sources::apply_cache_mode(req)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::MYGENE,
            ))
            .await?;
        let status = resp.status();
        let content_type = resp.headers().get(reqwest::header::CONTENT_TYPE).cloned();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::MYGENE),
        )
        .await?;
        crate::sources::decode_json(
            crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
            status,
            content_type.as_ref(),
            &bytes,
            true,
        )
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
            "symbol,name,summary,alias,type_of_gene,ensembl.gene,ensembl.transcript,ensembl.protein,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg,HGNC"
        } else {
            "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg,HGNC"
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
    #[serde(rename = "HGNC")]
    pub hgnc: Option<serde_json::Value>,
}

impl MyGeneGetResponse {
    /// Decode MyGene's documented HGNC annotation while tolerating its observed
    /// identifier scalar/flat-array wire union. Any malformed or conflicting
    /// supplied value makes identity inconclusive.
    pub(crate) fn hgnc_ids(&self) -> Result<Vec<String>, ()> {
        fn parse_one(value: &serde_json::Value) -> Result<String, ()> {
            let digits = match value {
                serde_json::Value::String(value) => {
                    let value = value.trim();
                    let value = if value
                        .get(..5)
                        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("HGNC:"))
                    {
                        value.get(5..).ok_or(())?
                    } else {
                        value
                    };
                    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                        return Err(());
                    }
                    value
                }
                serde_json::Value::Number(value) if value.as_u64().is_some() => {
                    return u32::try_from(value.as_u64().ok_or(())?)
                        .ok()
                        .filter(|value| *value > 0)
                        .map(|value| format!("HGNC:{value}"))
                        .ok_or(());
                }
                _ => return Err(()),
            };
            let value = digits
                .parse::<u32>()
                .ok()
                .filter(|value| *value > 0)
                .ok_or(())?;
            Ok(format!("HGNC:{value}"))
        }

        let Some(value) = &self.hgnc else {
            return Ok(Vec::new());
        };
        let values: Vec<&serde_json::Value> = match value {
            serde_json::Value::Null => return Ok(Vec::new()),
            serde_json::Value::Array(values) if values.is_empty() => return Ok(Vec::new()),
            serde_json::Value::Array(values) => values.iter().collect(),
            value => vec![value],
        };
        let mut result = Vec::new();
        for value in values {
            let normalized = parse_one(value)?;
            if !result.contains(&normalized) {
                result.push(normalized);
            }
        }
        Ok(result)
    }
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
mod tests {
    mod construction {
        //! Tier 2 — request construction. Pure: builds `RequestPlan`s and asserts the exact
        //! method / path / query / body that would be sent. Nothing is sent.

        use crate::error::BioMcpError;
        use crate::sources::mygene::MyGeneClient;
        use crate::sources::{HttpMethod, RequestBody};

        #[test]
        fn search_plan_sets_path_and_core_query_params() {
            let plan = MyGeneClient::search_plan("symbol:EGFR", 5, 0, None).unwrap();
            assert_eq!(plan.method, HttpMethod::Get);
            assert_eq!(plan.path, "query");
            assert_eq!(plan.query_value("q"), Some("symbol:EGFR"));
            assert_eq!(plan.query_value("species"), Some("human"));
            assert_eq!(plan.query_value("size"), Some("5"));
            assert_eq!(plan.query_value("from"), Some("0"));
            assert!(!plan.has_query("chr"));
            let fields = plan.query_value("fields").expect("fields present");
            assert!(fields.contains("alias"));
            assert!(fields.contains("genomic_pos.chr"));
        }

        #[test]
        fn search_plan_adds_chr_filter_only_when_non_empty() {
            let plan = MyGeneClient::search_plan("symbol:EGFR", 5, 0, Some("7")).unwrap();
            assert_eq!(plan.query_value("chr"), Some("7"));

            let blank = MyGeneClient::search_plan("symbol:EGFR", 5, 0, Some("  ")).unwrap();
            assert!(!blank.has_query("chr"));
        }

        #[test]
        fn search_plan_rejects_offset_at_or_above_window() {
            let err = MyGeneClient::search_plan("symbol:EGFR", 5, 10_000, None).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("--offset"));
        }

        #[test]
        fn search_plan_rejects_offset_plus_limit_overflow() {
            let err = MyGeneClient::search_plan("symbol:EGFR", 2, 9_999, None).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("--limit"));
        }

        #[test]
        fn get_plan_default_uses_minimal_fields_quoted_symbol_and_size_one() {
            let plan = MyGeneClient::get_plan("BRAF", false).unwrap();
            assert_eq!(plan.path, "query");
            assert_eq!(plan.query_value("q"), Some("symbol:\"BRAF\""));
            assert_eq!(plan.query_value("species"), Some("human"));
            assert_eq!(plan.query_value("size"), Some("1"));
            let fields = plan.query_value("fields").expect("fields present");
            assert!(fields.contains("ensembl.gene"));
            assert!(fields.split(',').any(|field| field == "HGNC"));
            assert!(!fields.contains("ensembl.transcript"));
        }

        #[test]
        fn get_plan_with_transcripts_requests_transcript_and_protein_fields() {
            let plan = MyGeneClient::get_plan("BRAF", true).unwrap();
            let fields = plan.query_value("fields").expect("fields present");
            assert!(fields.contains("ensembl.transcript"));
            assert!(fields.contains("ensembl.protein"));
        }

        #[test]
        fn get_plan_rejects_empty_symbol() {
            let err = MyGeneClient::get_plan("   ", false).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("required"));
        }

        #[test]
        fn get_plan_rejects_overlong_symbol() {
            let err = MyGeneClient::get_plan(&"A".repeat(129), false).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("too long"));
        }

        #[test]
        fn get_plan_rejects_invalid_symbol_characters() {
            let err = MyGeneClient::get_plan("BRAF:V600E", false).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("letters, numbers"));
        }

        #[test]
        fn batch_symbols_plan_builds_post_form_preserving_input_order() {
            let (plan, ids) = MyGeneClient::batch_symbols_plan(&[
                " 1956 ".to_string(),
                "7157".to_string(),
                String::new(),
                "673".to_string(),
            ])
            .unwrap();
            assert_eq!(plan.method, HttpMethod::Post);
            assert_eq!(plan.path, "gene");
            assert_eq!(ids, vec!["1956", "7157", "673"]);
            match &plan.body {
                RequestBody::Form(form) => {
                    assert!(form.iter().any(|(k, v)| k == "ids" && v == "1956,7157,673"));
                    assert!(form.iter().any(|(k, v)| k == "fields" && v == "symbol"));
                    assert!(form.iter().any(|(k, v)| k == "species" && v == "human"));
                }
                other => panic!("expected form body, got {other:?}"),
            }
        }

        #[test]
        fn batch_symbols_plan_rejects_empty_input() {
            let err = MyGeneClient::batch_symbols_plan(&[]).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("at least one ID"));
        }

        #[test]
        fn batch_symbols_plan_rejects_oversized_batch() {
            let ids: Vec<String> = (1..=201).map(|n| n.to_string()).collect();
            let err = MyGeneClient::batch_symbols_plan(&ids).unwrap_err();
            assert!(matches!(err, BioMcpError::InvalidArgument(_)));
            assert!(err.to_string().contains("200"));
        }
    }

    mod parsing {
        //! Tier 3 — response parsing. Pure: feeds committed fixture bytes to `decode_json` and
        //! the response types, plus the pure post-processing helpers. No network, no server.

        use crate::sources::decode_json;
        use crate::sources::mygene::{
            MyGeneBatchGeneHit, MyGeneClient, MyGeneGetQueryResponse, MyGeneGetResponse,
            MyGeneSearchResponse, extract_uniprot_accession,
        };
        use reqwest::StatusCode;
        use reqwest::header::HeaderValue;

        macro_rules! fixture {
            ($name:expr) => {
                include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/testdata/sources/mygene/",
                    $name
                ))
            };
        }

        fn json_ct() -> HeaderValue {
            HeaderValue::from_static("application/json")
        }

        #[test]
        fn parses_search_response_from_real_fixture() {
            let resp: MyGeneSearchResponse = decode_json(
                crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
                StatusCode::OK,
                Some(&json_ct()),
                fixture!("search_egfr.json"),
                true,
            )
            .unwrap();
            assert!(!resp.hits.is_empty());
            assert!(resp.hits[0].symbol.is_some());
        }

        #[test]
        fn parses_get_response_fields_from_real_fixture() {
            let resp: MyGeneGetQueryResponse = decode_json(
                crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
                StatusCode::OK,
                Some(&json_ct()),
                fixture!("get_braf.json"),
                true,
            )
            .unwrap();
            let hit = resp.hits.into_iter().next().expect("a hit");
            assert_eq!(hit.symbol.as_deref(), Some("BRAF"));
            assert_eq!(
                hit.ensembl
                    .as_ref()
                    .and_then(|e| e.gene())
                    .map(String::as_str),
                Some("ENSG00000157764")
            );
            assert_eq!(
                hit.genomic_pos
                    .as_ref()
                    .and_then(|g| g.chr())
                    .map(String::as_str),
                Some("7")
            );
        }

        #[test]
        fn hgnc_wire_union_normalizes_and_rejects_malformed_values() {
            for (wire, expected) in [
                (serde_json::json!("008109"), vec!["HGNC:8109"]),
                (serde_json::json!(8109), vec!["HGNC:8109"]),
                (
                    serde_json::json!(["hgnc:8109", 8109, "HGNC:00042"]),
                    vec!["HGNC:8109", "HGNC:42"],
                ),
            ] {
                let mut hit: MyGeneGetResponse =
                    serde_json::from_value(serde_json::json!({})).unwrap();
                hit.hgnc = Some(wire);
                assert_eq!(hit.hgnc_ids().unwrap(), expected);
            }

            for wire in [
                serde_json::json!(0),
                serde_json::json!(-1),
                serde_json::json!(1.5),
                serde_json::json!([8109, false]),
                serde_json::json!([[8109]]),
            ] {
                let mut hit: MyGeneGetResponse =
                    serde_json::from_value(serde_json::json!({})).unwrap();
                hit.hgnc = Some(wire);
                assert!(hit.hgnc_ids().is_err(), "wire should be inconclusive");
            }
        }

        #[test]
        fn extract_uniprot_prefers_swiss_prot_from_real_fixture() {
            let resp: MyGeneGetQueryResponse = decode_json(
                crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
                StatusCode::OK,
                Some(&json_ct()),
                fixture!("get_braf.json"),
                true,
            )
            .unwrap();
            let hit = resp.hits.into_iter().next().expect("a hit");
            let uniprot = hit.uniprot.as_ref().expect("real BRAF carries uniprot");
            assert_eq!(
                extract_uniprot_accession(uniprot).as_deref(),
                Some("P15056")
            );
        }

        #[test]
        fn extract_uniprot_prefers_swiss_prot_over_trembl_synthetic() {
            let value = serde_json::json!({ "Swiss-Prot": ["P15056"], "TrEMBL": ["A0A0A0"] });
            assert_eq!(extract_uniprot_accession(&value).as_deref(), Some("P15056"));
        }

        #[test]
        fn extract_uniprot_returns_none_for_empty_object() {
            assert_eq!(extract_uniprot_accession(&serde_json::json!({})), None);
        }

        #[test]
        fn dedupe_symbols_maps_real_batch_in_input_order() {
            let rows: Vec<MyGeneBatchGeneHit> = decode_json(
                crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
                StatusCode::OK,
                Some(&json_ct()),
                fixture!("batch_symbols.json"),
                true,
            )
            .unwrap();
            // fixture maps 1956 -> EGFR, 7157 -> TP53, 673 -> BRAF
            let ids = vec!["7157".to_string(), "1956".to_string(), "673".to_string()];
            assert_eq!(
                MyGeneClient::dedupe_symbols_in_order(rows, &ids),
                vec!["TP53", "EGFR", "BRAF"]
            );
        }

        #[test]
        fn dedupe_symbols_dedupes_repeated_ids_keeping_first_position() {
            let rows: Vec<MyGeneBatchGeneHit> = serde_json::from_str(
        r#"[{"query":"1956","_id":"1956","symbol":"EGFR"},{"query":"7157","_id":"7157","symbol":"TP53"}]"#,
    )
    .unwrap();
            let ids = vec!["1956".to_string(), "7157".to_string(), "1956".to_string()];
            assert_eq!(
                MyGeneClient::dedupe_symbols_in_order(rows, &ids),
                vec!["EGFR", "TP53"]
            );
        }

        #[test]
        fn decode_json_maps_http_error_status_with_excerpt() {
            let err = decode_json::<MyGeneSearchResponse>(
                crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
                StatusCode::INTERNAL_SERVER_ERROR,
                None,
                b"upstream failure",
                true,
            )
            .unwrap_err();
            let msg = format!("{err:?}");
            assert_eq!(err.code(), "api");
            assert!(msg.contains("MyGene.info"), "got: {msg}");
            assert!(msg.contains("500"), "got: {msg}");
        }

        #[test]
        fn decode_json_rejects_non_json_content_type() {
            let html = HeaderValue::from_static("text/html");
            let err = decode_json::<MyGeneSearchResponse>(
                crate::error::SourceContext::retry(crate::error::SourceProvider::MYGENE),
                StatusCode::OK,
                Some(&html),
                b"<html><body>error</body></html>",
                true,
            )
            .unwrap_err();
            let msg = format!("{err:?}");
            assert!(msg.contains("MyGene.info"), "got: {msg}");
            assert!(msg.contains("HTML"), "got: {msg}");
        }
    }

    mod live;
}
