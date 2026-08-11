use std::borrow::Cow;
use std::cmp::Ordering;
use std::io::Read;

use flate2::read::GzDecoder;
use reqwest::StatusCode;
use reqwest::header::{ACCEPT, HeaderMap};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::error::BioMcpError;
use crate::sources::RequestPlan;

const UNIPROT_BASE: &str = "https://rest.uniprot.org";
const UNIPROT_API: &str = "uniprot";
const UNIPROT_BASE_ENV: &str = "BIOMCP_UNIPROT_BASE";
const UNIPROT_MAX_EXPANDED_BYTES: usize = 32 * 1024 * 1024;

pub struct UniProtClient {
    client: reqwest::Client,
    base: Cow<'static, str>,
}

#[derive(Debug, Clone)]
pub struct UniProtSearchPage {
    pub results: Vec<UniProtRecord>,
    pub total: Option<usize>,
    pub next_page_token: Option<String>,
}

impl UniProtClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::streaming_http_client()?,
            base: crate::sources::env_base(UNIPROT_BASE, UNIPROT_BASE_ENV),
        })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base.as_ref().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn request_from_plan(&self, plan: &RequestPlan) -> reqwest::RequestBuilder {
        let url = if plan.path.starts_with("http://") || plan.path.starts_with("https://") {
            plan.path.clone()
        } else {
            self.endpoint(&plan.path)
        };
        let mut request = self.client.get(url);
        for (key, value) in &plan.headers {
            request = request.header(key, value);
        }
        if !plan.query.is_empty() {
            request = request.query(&plan.query);
        }
        request
    }

    fn plan_url(&self, plan: &RequestPlan) -> String {
        if plan.path.starts_with("http://") || plan.path.starts_with("https://") {
            plan.path.clone()
        } else {
            self.endpoint(&plan.path)
        }
    }

    async fn get_json<T>(&self, request: reqwest::RequestBuilder) -> Result<T, BioMcpError>
    where
        T: DeserializeOwned,
    {
        let resp = crate::sources::retry_send(
            crate::error::SourceContext::retry(crate::error::SourceProvider::UNIPROT),
            3,
            || {
                request
                    .try_clone()
                    .expect("request should be cloneable")
                    .send()
            },
        )
        .await?;
        let status = resp.status();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::UNIPROT),
        )
        .await?;
        Self::decode_json_response(status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::UNIPROT,
            ))
        })
    }

    pub(crate) fn get_record_plan(accession: &str) -> Result<RequestPlan, BioMcpError> {
        let accession = accession.trim();
        if accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "UniProt accession is required".into(),
            ));
        }

        Ok(RequestPlan::get(format!("uniprotkb/{accession}.json"))
            .header(ACCEPT.as_str(), "application/json"))
    }

    pub(crate) fn search_plan(
        query: &str,
        limit: usize,
        offset: usize,
        next_page: Option<&str>,
    ) -> Result<RequestPlan, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "UniProt query is required".into(),
            ));
        }

        let size = limit.clamp(1, 25).to_string();
        let offset = offset.to_string();
        let token = normalize_next_page_token(next_page)?;
        if let Some(token) = token.as_deref() {
            if token.starts_with("http://") || token.starts_with("https://") {
                return Ok(
                    RequestPlan::get(token.to_string()).header(ACCEPT.as_str(), "application/json")
                );
            }

            return Ok(RequestPlan::get("uniprotkb/search")
                .header(ACCEPT.as_str(), "application/json")
                .query("query", query)
                .query("format", "json")
                .query("size", size)
                .query("cursor", token)
                .query(
                    "fields",
                    "accession,id,protein_name,gene_names,organism_name,length,cc_function,xref_pdb,xref_alphafolddb",
                ));
        }

        Ok(RequestPlan::get("uniprotkb/search")
            .header(ACCEPT.as_str(), "application/json")
            .query("query", query)
            .query("format", "json")
            .query("size", size)
            .query("offset", offset)
            .query(
                "fields",
                "accession,id,protein_name,gene_names,organism_name,length,cc_function,xref_pdb,xref_alphafolddb",
            ))
    }

    pub(crate) fn decode_json_response<T: DeserializeOwned>(
        status: StatusCode,
        bytes: &[u8],
    ) -> Result<T, BioMcpError> {
        let payload = decode_uniprot_payload(bytes)?;
        if !status.is_success() {
            let excerpt = crate::sources::body_excerpt(&payload);
            return Err(BioMcpError::Api {
                api: UNIPROT_API.to_string(),
                message: format!("HTTP {status}: {excerpt}"),
            });
        }
        serde_json::from_slice(&payload).map_err(|source| {
            let excerpt = crate::sources::body_excerpt(&payload);
            BioMcpError::Api {
                api: UNIPROT_API.to_string(),
                message: format!("Invalid JSON response: {excerpt} ({source})"),
            }
        })
    }

    pub(crate) fn decode_search_response(
        status: StatusCode,
        headers: &HeaderMap,
        bytes: &[u8],
    ) -> Result<UniProtSearchPage, BioMcpError> {
        let total = headers
            .get("x-total-results")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok());
        let next_page_token = parse_uniprot_next_link(headers.get("link"));
        let parsed: UniProtSearchResponse = Self::decode_json_response(status, bytes)?;
        Ok(UniProtSearchPage {
            results: parsed.results,
            total,
            next_page_token,
        })
    }

    pub async fn get_record(&self, accession: &str) -> Result<UniProtRecord, BioMcpError> {
        let plan = Self::get_record_plan(accession)?;
        let url = self.plan_url(&plan);
        crate::sources::rate_limit::wait_for_url_str(&url).await;
        self.get_json(self.request_from_plan(&plan)).await
    }

    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        offset: usize,
        next_page: Option<&str>,
    ) -> Result<UniProtSearchPage, BioMcpError> {
        let plan = Self::search_plan(query, limit, offset, next_page)?;
        let url = self.plan_url(&plan);
        crate::sources::rate_limit::wait_for_url_str(&url).await;
        let resp = crate::sources::retry_send(
            crate::error::SourceContext::retry(crate::error::SourceProvider::UNIPROT),
            3,
            || async { self.request_from_plan(&plan).send().await },
        )
        .await?;
        let status = resp.status();
        let headers = resp.headers().clone();
        let bytes = crate::sources::read_limited_source_body(
            resp,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::UNIPROT),
        )
        .await?;
        Self::decode_search_response(status, &headers, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::UNIPROT,
            ))
        })
    }
}

fn decode_uniprot_payload(bytes: &[u8]) -> Result<Vec<u8>, BioMcpError> {
    decode_uniprot_payload_with_limit(bytes, UNIPROT_MAX_EXPANDED_BYTES)
}

fn read_uniprot_expanded<R: Read>(reader: R, limit: usize) -> Result<Vec<u8>, BioMcpError> {
    let mut reader = reader.take(limit.saturating_add(1) as u64);
    let mut decoded = Vec::new();
    reader
        .read_to_end(&mut decoded)
        .map_err(|err| BioMcpError::Api {
            api: UNIPROT_API.to_string(),
            message: format!("Failed to decode gzip response: {err}"),
        })?;
    if decoded.len() > limit {
        return Err(BioMcpError::Api {
            api: UNIPROT_API.to_string(),
            message: format!("Expanded response exceeds {limit} bytes"),
        }
        .with_source_context(crate::error::SourceContext::narrow(
            crate::error::SourceProvider::UNIPROT,
        )));
    }
    Ok(decoded)
}

fn decode_uniprot_payload_with_limit(bytes: &[u8], limit: usize) -> Result<Vec<u8>, BioMcpError> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        read_uniprot_expanded(GzDecoder::new(bytes), limit)
    } else {
        read_uniprot_expanded(bytes, limit)
    }
}

fn parse_uniprot_next_link(value: Option<&reqwest::header::HeaderValue>) -> Option<String> {
    let raw = value?.to_str().ok()?;
    for part in raw.split(',') {
        let piece = part.trim();
        if !piece.contains("rel=\"next\"") {
            continue;
        }
        let start = piece.find('<')?;
        let end = piece[start + 1..].find('>')?;
        let url = piece[start + 1..start + 1 + end].trim();
        if !url.is_empty() {
            return Some(url.to_string());
        }
    }
    None
}

fn normalize_next_page_token(next_page: Option<&str>) -> Result<Option<String>, BioMcpError> {
    let Some(token) = next_page
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
    else {
        return Ok(None);
    };

    if token.len() > 2048 {
        return Err(BioMcpError::InvalidArgument(
            "--next-page token is too long".into(),
        ));
    }
    if token.chars().all(|ch| ch.is_ascii_digit()) {
        return Err(BioMcpError::InvalidArgument(
            "--next-page token is invalid. Use pagination.next_page_token from the previous result."
                .into(),
        ));
    }
    if token.chars().any(|ch| ch.is_whitespace()) {
        return Err(BioMcpError::InvalidArgument(
            "--next-page token must not contain whitespace".into(),
        ));
    }
    if token.starts_with("http://") || token.starts_with("https://") {
        let parsed = reqwest::Url::parse(&token).map_err(|_| {
            BioMcpError::InvalidArgument(
                "--next-page token URL is invalid. Use pagination.next_page_token from the previous result."
                    .into(),
            )
        })?;
        if parsed.host_str() != Some("rest.uniprot.org") {
            return Err(BioMcpError::InvalidArgument(
                "--next-page token must be a rest.uniprot.org URL. Use pagination.next_page_token from the previous result.".into(),
            ));
        }
    }

    Ok(Some(token))
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtSearchResponse {
    #[serde(default)]
    pub results: Vec<UniProtRecord>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtRecord {
    #[serde(default)]
    pub primary_accession: String,
    #[serde(rename = "uniProtkbId")]
    pub uni_prot_kb_id: Option<String>,
    pub protein_description: Option<UniProtProteinDescription>,
    #[serde(default)]
    pub genes: Vec<UniProtGene>,
    pub organism: Option<UniProtOrganism>,
    pub sequence: Option<UniProtSequence>,
    #[serde(default)]
    pub comments: Vec<UniProtComment>,
    #[serde(rename = "uniProtKBCrossReferences", default)]
    pub uni_prot_kb_cross_references: Vec<UniProtCrossReference>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtProteinDescription {
    pub recommended_name: Option<UniProtNameContainer>,
    pub submission_names: Option<Vec<UniProtNameContainer>>,
    #[serde(default)]
    pub alternative_names: Vec<UniProtNameContainer>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtNameContainer {
    pub full_name: Option<UniProtTextValue>,
    #[serde(default)]
    pub short_names: Vec<UniProtTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtTextValue {
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtGene {
    pub gene_name: Option<UniProtTextValue>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtOrganism {
    pub scientific_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtSequence {
    pub length: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtComment {
    pub comment_type: Option<String>,
    #[serde(default)]
    pub texts: Vec<UniProtTextValue>,
    #[serde(default)]
    pub isoforms: Vec<UniProtIsoform>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtIsoform {
    pub name: UniProtTextValue,
    #[serde(default)]
    pub synonyms: Vec<UniProtTextValue>,
    pub isoform_sequence_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UniProtProteinIsoformSummary {
    pub name: String,
    pub is_displayed: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniProtCrossReference {
    pub database: Option<String>,
    pub id: Option<String>,
    #[serde(default)]
    pub properties: Vec<UniProtCrossReferenceProperty>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UniProtCrossReferenceProperty {
    pub key: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UniProtPdbStructure {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residue_covered: Option<bool>,
    pub source: String,
}

impl UniProtRecord {
    pub fn display_name(&self) -> String {
        if let Some(desc) = self.protein_description.as_ref() {
            if let Some(value) = desc
                .recommended_name
                .as_ref()
                .and_then(|v| v.full_name.as_ref())
                .map(|v| v.value.trim())
                .filter(|v| !v.is_empty())
            {
                return value.to_string();
            }

            if let Some(value) = desc
                .submission_names
                .as_ref()
                .and_then(|v| v.first())
                .and_then(|v| v.full_name.as_ref())
                .map(|v| v.value.trim())
                .filter(|v| !v.is_empty())
            {
                return value.to_string();
            }
        }

        self.primary_accession.clone()
    }

    pub fn primary_gene_symbol(&self) -> Option<String> {
        self.genes
            .first()
            .and_then(|g| g.gene_name.as_ref())
            .map(|g| g.value.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    pub fn function_summary(&self) -> Option<String> {
        self.comments
            .iter()
            .find(|c| {
                c.comment_type
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.eq_ignore_ascii_case("function"))
            })
            .and_then(|c| c.texts.first())
            .map(|v| v.value.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    pub fn protein_isoforms(&self) -> Vec<UniProtProteinIsoformSummary> {
        let Some(comment) = self.comments.iter().find(|c| {
            c.comment_type
                .as_deref()
                .map(str::trim)
                .is_some_and(|v| v.eq_ignore_ascii_case("alternative products"))
        }) else {
            return Vec::new();
        };

        comment
            .isoforms
            .iter()
            .filter_map(|isoform| {
                let name = isoform
                    .synonyms
                    .iter()
                    .find_map(|synonym| {
                        let value = synonym.value.trim();
                        (!value.is_empty()).then(|| value.to_string())
                    })
                    .or_else(|| {
                        let value = isoform.name.value.trim();
                        (!value.is_empty()).then(|| value.to_string())
                    })?;
                let is_displayed = isoform
                    .isoform_sequence_status
                    .as_deref()
                    .map(str::trim)
                    .is_some_and(|v| v.eq_ignore_ascii_case("displayed"));
                Some(UniProtProteinIsoformSummary { name, is_displayed })
            })
            .collect()
    }

    pub fn alternative_protein_names(&self) -> Vec<String> {
        let Some(desc) = self.protein_description.as_ref() else {
            return Vec::new();
        };

        let display_name = self.display_name();
        let display_name = display_name.trim();
        let mut names = Vec::new();

        for alt in &desc.alternative_names {
            for short_name in &alt.short_names {
                let value = short_name.value.trim();
                if value.is_empty()
                    || value.eq_ignore_ascii_case(display_name)
                    || names
                        .iter()
                        .any(|name: &String| name.eq_ignore_ascii_case(value))
                {
                    continue;
                }
                names.push(value.to_string());
            }

            let Some(full_name) = alt.full_name.as_ref() else {
                continue;
            };
            let value = full_name.value.trim();
            if value.is_empty()
                || value.eq_ignore_ascii_case(display_name)
                || names
                    .iter()
                    .any(|name: &String| name.eq_ignore_ascii_case(value))
            {
                continue;
            }
            names.push(value.to_string());
        }

        names
    }

    pub fn structure_ids(&self) -> Vec<String> {
        let mut out = Vec::new();
        for x in &self.uni_prot_kb_cross_references {
            let Some(db) = x.database.as_deref().map(str::trim) else {
                continue;
            };
            let Some(id) = x.id.as_deref().map(str::trim) else {
                continue;
            };
            if id.is_empty() {
                continue;
            }
            if !matches!(db, "PDB" | "AlphaFoldDB") {
                continue;
            }
            if out.iter().any(|v: &String| v == id) {
                continue;
            }
            out.push(id.to_string());
        }
        out
    }

    pub fn structure_count(&self) -> usize {
        self.structure_ids().len()
    }

    pub fn alphafold_ids(&self) -> Vec<String> {
        self.uni_prot_kb_cross_references
            .iter()
            .filter_map(|x| {
                let db = x.database.as_deref()?.trim();
                let id = x.id.as_deref()?.trim();
                (db == "AlphaFoldDB" && !id.is_empty()).then(|| id.to_string())
            })
            .collect()
    }

    pub fn typed_pdb_structures(&self, residue: Option<u32>) -> Vec<UniProtPdbStructure> {
        self.uni_prot_kb_cross_references
            .iter()
            .filter_map(|x| {
                let db = x.database.as_deref()?.trim();
                let id = x.id.as_deref()?.trim();
                if db != "PDB" || id.is_empty() {
                    return None;
                }
                let chains = cross_ref_property(x, "Chains");
                Some(UniProtPdbStructure {
                    id: id.to_string(),
                    method: cross_ref_property(x, "Method"),
                    resolution: cross_ref_property(x, "Resolution")
                        .filter(|v| !v.trim().is_empty() && v.trim() != "-"),
                    residue_covered: residue.and_then(|position| {
                        chains
                            .as_deref()
                            .map(|value| chain_coverage_contains(value, position))
                    }),
                    chains,
                    source: "UniProt cross-reference".to_string(),
                })
            })
            .collect()
    }

    pub fn structure_summaries(&self, limit: usize) -> Vec<String> {
        #[derive(Debug)]
        struct PdbRow {
            id: String,
            method: Option<String>,
            resolution_text: Option<String>,
            resolution_value: Option<f64>,
        }

        let limit = limit.max(1);
        let mut seen: Vec<String> = Vec::new();
        let mut pdb_rows: Vec<PdbRow> = Vec::new();
        let mut other_rows: Vec<String> = Vec::new();

        for x in &self.uni_prot_kb_cross_references {
            let Some(db) = x.database.as_deref().map(str::trim) else {
                continue;
            };
            let Some(id) = x.id.as_deref().map(str::trim) else {
                continue;
            };
            if id.is_empty() {
                continue;
            }
            if !matches!(db, "PDB" | "AlphaFoldDB") {
                continue;
            }
            if seen.iter().any(|v| v == id) {
                continue;
            }
            seen.push(id.to_string());

            if db == "PDB" {
                let method = cross_ref_property(x, "Method");
                let resolution_text = cross_ref_property(x, "Resolution")
                    .map(|v| v.trim().to_string())
                    .filter(|v| !v.is_empty() && v != "-");
                let resolution_value = resolution_text
                    .as_deref()
                    .and_then(parse_resolution_angstrom);

                pdb_rows.push(PdbRow {
                    id: id.to_string(),
                    method,
                    resolution_text,
                    resolution_value,
                });
            } else {
                other_rows.push(format!("{id} (AlphaFold model)"));
            }
        }

        pdb_rows.sort_by(|a, b| match (a.resolution_value, b.resolution_value) {
            (Some(lhs), Some(rhs)) => lhs.partial_cmp(&rhs).unwrap_or(Ordering::Equal),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => a.id.cmp(&b.id),
        });

        let mut out: Vec<String> = Vec::new();
        for row in pdb_rows {
            let line = match (row.method.as_deref(), row.resolution_text.as_deref()) {
                (Some(method), Some(resolution)) => format!("{} ({method}, {resolution})", row.id),
                (Some(method), None) => format!("{} ({method})", row.id),
                (None, Some(resolution)) => format!("{} ({resolution})", row.id),
                (None, None) => row.id,
            };
            out.push(line);
            if out.len() >= limit {
                return out;
            }
        }

        for row in other_rows {
            out.push(row);
            if out.len() >= limit {
                break;
            }
        }

        out
    }
}

fn cross_ref_property(row: &UniProtCrossReference, key: &str) -> Option<String> {
    row.properties.iter().find_map(|p| {
        let matches = p
            .key
            .as_deref()
            .map(str::trim)
            .is_some_and(|k| k.eq_ignore_ascii_case(key));
        if !matches {
            return None;
        }
        p.value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .map(str::to_string)
    })
}

fn chain_coverage_contains(value: &str, position: u32) -> bool {
    value.split(',').any(|piece| {
        let Some((_, ranges)) = piece.trim().split_once('=') else {
            return false;
        };
        ranges.split('/').any(|range| {
            let Some((start, end)) = range.trim().split_once('-') else {
                return false;
            };
            let (Ok(start), Ok(end)) = (start.trim().parse::<u32>(), end.trim().parse::<u32>())
            else {
                return false;
            };
            start <= position && position <= end
        })
    })
}

fn parse_resolution_angstrom(value: &str) -> Option<f64> {
    let token = value
        .trim()
        .trim_end_matches('A')
        .trim_end_matches('a')
        .trim();
    let token = token.split_whitespace().next()?;
    token.parse::<f64>().ok()
}

#[cfg(test)]
mod tests;
