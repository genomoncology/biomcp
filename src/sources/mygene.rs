use std::borrow::Cow;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::BioMcpError;
use crate::sources::is_valid_gene_symbol;
use crate::utils::serde::StringOrVec;

const MYGENE_BASE: &str = "https://mygene.info/v3";
const MYGENE_API: &str = "mygene.info";
const MYGENE_BASE_ENV: &str = "BIOMCP_MYGENE_BASE";
const MYGENE_MAX_RESULT_WINDOW: usize = 10_000;

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

    #[cfg(test)]
    fn new_for_test(base: String) -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: Cow::Owned(base),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
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
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&bytes);
            return Err(BioMcpError::Api {
                api: MYGENE_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        crate::sources::ensure_json_content_type(MYGENE_API, content_type.as_ref(), &bytes)?;
        serde_json::from_slice(&bytes).map_err(|source| BioMcpError::ApiJson {
            api: MYGENE_API.to_string(),
            source,
        })
    }

    /// Search genes by query
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        chromosome: Option<&str>,
    ) -> Result<MyGeneSearchResponse, BioMcpError> {
        Self::validate_search_window(limit, offset)?;
        let url = self.endpoint("query");
        let size = limit.to_string();
        let from = offset.to_string();
        let mut req = self.client.get(&url).query(&[
            ("q", query),
            ("species", "human"),
            (
                "fields",
                "symbol,name,entrezgene,type_of_gene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,MIM,uniprot,pathway.kegg.id,pathway.reactome.id,go.BP.id,go.CC.id,go.MF.id",
            ),
            ("size", size.as_str()),
            ("from", from.as_str()),
        ]);

        if let Some(chr) = chromosome.map(str::trim).filter(|v| !v.is_empty()) {
            // MyGene supports `chr` query param filtering for `/query`.
            req = req.query(&[("chr", chr)]);
        }

        self.get_json(req).await
    }

    /// Get gene by symbol (single query for fields needed by the caller)
    pub async fn get(
        &self,
        symbol: &str,
        include_transcripts: bool,
    ) -> Result<MyGeneGetResponse, BioMcpError> {
        let query_url = self.endpoint("query");
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
        let query_resp: MyGeneGetQueryResponse = self
            .get_json(self.client.get(&query_url).query(&[
                ("q", q.as_str()),
                ("species", "human"),
                ("fields", fields),
                ("size", "1"),
            ]))
            .await?;

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

fn extract_uniprot_accession(value: &serde_json::Value) -> Option<String> {
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
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyGeneHit>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyGeneGetQueryResponse {
    #[allow(dead_code)]
    pub total: usize,
    pub hits: Vec<MyGeneGetResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MyGeneHit {
    pub symbol: Option<String>,
    pub name: Option<String>,
    pub entrezgene: Option<StringOrU64>,
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
    use super::*;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn search_includes_chr_query_param_for_chromosome_filter() {
        let server = MockServer::start().await;
        let client = MyGeneClient::new_for_test(format!("{}/v3", server.uri())).unwrap();

        let body = r#"{
          "total": 1,
          "hits": [
            {"symbol": "EGFR", "name": "epidermal growth factor receptor", "genomic_pos": {"chr": "7"}}
          ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", "symbol:EGFR"))
            .and(query_param("species", "human"))
            .and(query_param("size", "5"))
            .and(query_param("from", "0"))
            .and(query_param("chr", "7"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client.search("symbol:EGFR", 5, 0, Some("7")).await.unwrap();
        assert_eq!(resp.hits.len(), 1);
    }

    #[tokio::test]
    async fn get_uses_single_query_minimal_fields_by_default() {
        let server = MockServer::start().await;
        let client = MyGeneClient::new_for_test(format!("{}/v3", server.uri())).unwrap();

        let body = r#"{
          "total": 1,
          "hits": [
            {
              "_id": "673",
              "symbol": "BRAF",
              "name": "B-Raf proto-oncogene, serine/threonine kinase",
              "entrezgene": 673,
              "summary": "example summary.",
              "alias": ["B-RAF1"],
              "type_of_gene": "protein-coding",
              "ensembl": {"gene": "ENSG00000157764"},
              "genomic_pos": {"chr": "7"}
            }
          ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", "symbol:\"BRAF\""))
            .and(query_param("species", "human"))
            .and(query_param(
                "fields",
                "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg",
            ))
            .and(query_param("size", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client.get("BRAF", false).await.unwrap();
        assert_eq!(resp.symbol.as_deref(), Some("BRAF"));
        assert_eq!(
            resp.ensembl
                .as_ref()
                .and_then(|e| e.gene())
                .map(String::as_str),
            Some("ENSG00000157764")
        );
    }

    #[tokio::test]
    async fn get_includes_transcripts_fields_when_requested() {
        let server = MockServer::start().await;
        let client = MyGeneClient::new_for_test(format!("{}/v3", server.uri())).unwrap();

        let body = r#"{
          "total": 1,
          "hits": [
            {
              "symbol": "BRAF",
              "name": "B-Raf proto-oncogene, serine/threonine kinase",
              "entrezgene": 673,
              "summary": "example summary.",
              "alias": ["B-RAF1"],
              "type_of_gene": "protein-coding",
              "ensembl": {
                "gene": "ENSG00000157764",
                "transcript": ["ENST00000288602"],
                "protein": ["ENSP00000288602"]
              },
              "genomic_pos": {"chr": "7"}
            }
          ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", "symbol:\"BRAF\""))
            .and(query_param("species", "human"))
            .and(query_param(
                "fields",
                "symbol,name,summary,alias,type_of_gene,ensembl.gene,ensembl.transcript,ensembl.protein,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg",
            ))
            .and(query_param("size", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let resp = client.get("BRAF", true).await.unwrap();
        let ensembl = resp.ensembl.expect("ensembl expected");
        let first = ensembl.first().expect("ensembl entry expected");
        assert!(first.transcript.is_some());
        assert!(first.protein.is_some());
    }

    #[tokio::test]
    async fn get_rejects_invalid_symbol_characters() {
        let client = MyGeneClient::new_for_test("http://127.0.0.1/v3".into()).unwrap();
        let err = client.get("BRAF:V600E", false).await.unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("letters, numbers"));
    }

    #[tokio::test]
    async fn search_rejects_html_content_type_before_json_parse() {
        let server = MockServer::start().await;
        let client = MyGeneClient::new_for_test(format!("{}/v3", server.uri())).unwrap();

        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw("<html><body>error page</body></html>", "text/html"),
            )
            .expect(1)
            .mount(&server)
            .await;

        let err = client.search("EGFR", 1, 0, None).await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("mygene.info"));
        assert!(msg.contains("HTML"));
    }

    #[tokio::test]
    async fn search_rejects_offset_above_mygene_window() {
        let client = MyGeneClient::new_for_test("http://127.0.0.1/v3".into()).unwrap();
        let err = client
            .search("symbol:EGFR", 5, 10_000, None)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--offset"));
    }

    #[tokio::test]
    async fn search_rejects_offset_limit_window_overflow() {
        let client = MyGeneClient::new_for_test("http://127.0.0.1/v3".into()).unwrap();
        let err = client
            .search("symbol:EGFR", 2, 9_999, None)
            .await
            .unwrap_err();
        assert!(matches!(err, BioMcpError::InvalidArgument(_)));
        assert!(err.to_string().contains("--limit"));
    }

    #[tokio::test]
    async fn resolve_uniprot_accession_prefers_swiss_prot() {
        let server = MockServer::start().await;
        let client = MyGeneClient::new_for_test(format!("{}/v3", server.uri())).unwrap();

        let body = r#"{
          "total": 1,
          "hits": [
            {
              "symbol": "BRAF",
              "uniprot": {
                "Swiss-Prot": ["P15056"],
                "TrEMBL": ["A0A0A0"]
              }
            }
          ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", "symbol:\"BRAF\""))
            .and(query_param("species", "human"))
            .and(query_param(
                "fields",
                "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg",
            ))
            .and(query_param("size", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let accession = client.resolve_uniprot_accession("BRAF").await.unwrap();
        assert_eq!(accession, "P15056");
    }

    #[tokio::test]
    async fn resolve_uniprot_accession_returns_not_found_when_missing() {
        let server = MockServer::start().await;
        let client = MyGeneClient::new_for_test(format!("{}/v3", server.uri())).unwrap();

        let body = r#"{
          "total": 1,
          "hits": [
            {
              "symbol": "BRAF",
              "name": "B-Raf proto-oncogene"
            }
          ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v3/query"))
            .and(query_param("q", "symbol:\"BRAF\""))
            .and(query_param("species", "human"))
            .and(query_param(
                "fields",
                "symbol,name,summary,alias,type_of_gene,ensembl.gene,entrezgene,genomic_pos.chr,genomic_pos.start,genomic_pos.end,genomic_pos.strand,MIM,uniprot,pathway.kegg",
            ))
            .and(query_param("size", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/json"))
            .expect(1)
            .mount(&server)
            .await;

        let err = client.resolve_uniprot_accession("BRAF").await.unwrap_err();
        assert!(matches!(err, BioMcpError::NotFound { .. }));
    }
}
