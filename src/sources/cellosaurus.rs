//! Cellosaurus cell line knowledge resource client.
//!
//! Cellosaurus publishes no rate limit and needs no key. The origin is paced at
//! 334 ms in `crate::sources::rate_limit`, matching KEGG.

use std::borrow::Cow;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::error::BioMcpError;
use crate::sources::RequestBuilderSourceContextExt;

const CELLOSAURUS_BASE: &str = "https://api.cellosaurus.org";
const CELLOSAURUS_API: &str = "cellosaurus";
const CELLOSAURUS_BASE_ENV: &str = "BIOMCP_CELLOSAURUS_BASE";

/// Projection for the default card. The card never fetches `dr`.
pub(crate) const CARD_FIELDS: &[&str] = &["ac", "id", "sy", "ox", "di", "ca", "sx", "ag"];
/// Projection for a name search row.
pub(crate) const SEARCH_FIELDS: &[&str] = &["ac", "id", "sy", "ox", "di", "ca"];
/// Projection for a reverse cross-reference lookup.
pub(crate) const XREF_LOOKUP_FIELDS: &[&str] = &["ac", "id"];
/// The window Cellosaurus fills for a name search.
pub(crate) const SEARCH_ROWS: usize = 1000;
/// The window for a reverse cross-reference lookup.
pub(crate) const XREF_LOOKUP_ROWS: usize = 10;

pub struct CellosaurusClient {
    client: reqwest_middleware::ClientWithMiddleware,
    base: Cow<'static, str>,
}

/// One page of search rows, with the flag that says the window filled.
#[derive(Debug, Clone)]
pub(crate) struct CellosaurusSearchPage {
    pub records: Vec<CellosaurusRecord>,
    pub window_full: bool,
}

/// The Cellosaurus release the data came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CellosaurusRelease {
    pub version: String,
    pub updated: String,
}

impl CellosaurusClient {
    pub fn new() -> Result<Self, BioMcpError> {
        Ok(Self {
            client: crate::sources::shared_client()?,
            base: crate::sources::env_base(CELLOSAURUS_BASE, CELLOSAURUS_BASE_ENV),
        })
    }

    fn build_url(&self, segments: &[&str], params: &[(&str, &str)]) -> Result<String, BioMcpError> {
        let mut url = reqwest::Url::parse(self.base.as_ref()).map_err(|err| BioMcpError::Api {
            api: CELLOSAURUS_API.to_string(),
            message: format!("Invalid Cellosaurus base URL: {err}"),
        })?;
        {
            let mut path = url.path_segments_mut().map_err(|_| BioMcpError::Api {
                api: CELLOSAURUS_API.to_string(),
                message: "Invalid Cellosaurus base URL path".to_string(),
            })?;
            path.pop_if_empty();
            for segment in segments {
                path.push(segment);
            }
        }
        {
            let mut query = url.query_pairs_mut();
            for (key, value) in params {
                query.append_pair(key, value);
            }
        }
        Ok(url.to_string())
    }

    pub(crate) fn record_url(
        &self,
        accession: &str,
        fields: &[&str],
    ) -> Result<String, BioMcpError> {
        let accession = accession.trim();
        if accession.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Cellosaurus accession is required".into(),
            ));
        }
        self.build_url(
            &["cell-line", accession],
            &[("format", "json"), ("fields", &fields.join(","))],
        )
    }

    pub(crate) fn search_url(
        &self,
        query: &str,
        fields: &[&str],
        rows: usize,
    ) -> Result<String, BioMcpError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(BioMcpError::InvalidArgument(
                "Cellosaurus search query is required".into(),
            ));
        }
        self.build_url(
            &["search", "cell-line"],
            &[
                ("q", query),
                ("format", "json"),
                ("fields", &fields.join(",")),
                ("rows", &rows.to_string()),
            ],
        )
    }

    pub(crate) fn release_info_url(&self) -> Result<String, BioMcpError> {
        self.build_url(&["release-info"], &[("format", "json")])
    }

    /// Fetch one URL and decode the Cellosaurus envelope.
    ///
    /// `not_found` maps an HTTP 404 to the caller's own error, because only the
    /// caller knows the identifier the user typed.
    async fn fetch_bytes(&self, url: &str) -> Result<(StatusCode, Vec<u8>), BioMcpError> {
        let request = self.client.get(url);
        let response = crate::sources::apply_cache_mode(request)
            .send_with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CELLOSAURUS,
            ))
            .await?;
        let status = response.status();
        let bytes = crate::sources::read_limited_source_body(
            response,
            crate::error::SourceContext::narrow(crate::error::SourceProvider::CELLOSAURUS),
        )
        .await?;
        Ok((status, bytes))
    }

    async fn fetch(&self, url: &str) -> Result<Option<CellosaurusBody>, BioMcpError> {
        let (status, bytes) = self.fetch_bytes(url).await?;
        decode_body(url, status, &bytes).map_err(|error| {
            error.with_source_context(crate::error::SourceContext::retry(
                crate::error::SourceProvider::CELLOSAURUS,
            ))
        })
    }

    /// One record by accession. `Ok(None)` means Cellosaurus answered 404.
    pub(crate) async fn get_record(
        &self,
        accession: &str,
        fields: &[&str],
    ) -> Result<Option<CellosaurusRecord>, BioMcpError> {
        let url = self.record_url(accession, fields)?;
        let Some(body) = self.fetch(&url).await? else {
            return Ok(None);
        };
        Ok(body.cell_line_list.into_iter().next())
    }

    /// The one search primitive. Every search path wraps this.
    pub(crate) async fn search(
        &self,
        query: &str,
        fields: &[&str],
        rows: usize,
    ) -> Result<CellosaurusSearchPage, BioMcpError> {
        let url = self.search_url(query, fields, rows)?;
        let records = self
            .fetch(&url)
            .await?
            .map(|body| body.cell_line_list)
            .unwrap_or_default();
        let window_full = records.len() >= rows;
        Ok(CellosaurusSearchPage {
            records,
            window_full,
        })
    }

    /// Search names and synonyms for one spelling.
    pub(crate) async fn search_names(
        &self,
        name: &str,
    ) -> Result<CellosaurusSearchPage, BioMcpError> {
        self.search(
            &format!("idsy:\"{}\"", escape_solr_value(name)),
            SEARCH_FIELDS,
            SEARCH_ROWS,
        )
        .await
    }

    /// Search by accession, for a query that already looks like a CVCL ID.
    pub(crate) async fn search_accession(
        &self,
        accession: &str,
    ) -> Result<CellosaurusSearchPage, BioMcpError> {
        self.search(
            &format!("ac:\"{}\"", escape_solr_value(accession)),
            SEARCH_FIELDS,
            SEARCH_ROWS,
        )
        .await
    }

    /// Reverse lookup from a source ID held in a cross-reference.
    pub(crate) async fn search_xref(&self, id: &str) -> Result<CellosaurusSearchPage, BioMcpError> {
        self.search(
            &format!("dr:\"{}\"", escape_solr_value(id)),
            XREF_LOOKUP_FIELDS,
            XREF_LOOKUP_ROWS,
        )
        .await
    }

    /// The Cellosaurus release behind every output.
    pub(crate) async fn release_info(&self) -> Result<CellosaurusRelease, BioMcpError> {
        let url = self.release_info_url()?;
        let body = self.fetch(&url).await?.ok_or_else(|| BioMcpError::Api {
            api: CELLOSAURUS_API.to_string(),
            message: format!("Cellosaurus release info is unavailable at {url}"),
        })?;
        body.header
            .and_then(|header| header.release)
            .and_then(|release| match (release.version, release.updated) {
                (Some(version), Some(updated)) => Some(CellosaurusRelease { version, updated }),
                _ => None,
            })
            .ok_or_else(|| BioMcpError::Api {
                api: CELLOSAURUS_API.to_string(),
                message: "Cellosaurus release info carried no version and date".to_string(),
            })
    }
}

/// Escape a value for a Solr phrase. Cellosaurus quotes the phrase, so the
/// quote and the backslash are the two characters that must be escaped.
pub(crate) fn escape_solr_value(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch == '"' || ch == '\\' {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// Decode one Cellosaurus response body.
///
/// A 404 is a missing record, not a failure. An HTTP 200 carrying an HTML
/// human-verification page where JSON was expected is a provider error that
/// names the URL and stops the command.
pub(crate) fn decode_body(
    url: &str,
    status: StatusCode,
    bytes: &[u8],
) -> Result<Option<CellosaurusBody>, BioMcpError> {
    if status == StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !status.is_success() {
        return Err(BioMcpError::Api {
            api: CELLOSAURUS_API.to_string(),
            message: format!(
                "HTTP {status} from {url}: {}",
                crate::sources::body_excerpt(bytes)
            ),
        });
    }
    if looks_like_html(bytes) {
        return Err(BioMcpError::Api {
            api: CELLOSAURUS_API.to_string(),
            message: format!(
                "Cellosaurus returned an HTML page where JSON was expected at {url}: {}",
                crate::sources::body_excerpt(bytes)
            ),
        });
    }
    let envelope: CellosaurusEnvelope =
        serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
            api: CELLOSAURUS_API.to_string(),
            source,
        })?;
    Ok(Some(envelope.cellosaurus))
}

fn looks_like_html(bytes: &[u8]) -> bool {
    let sniff = String::from_utf8_lossy(&bytes[..bytes.len().min(128)])
        .trim_start()
        .to_ascii_lowercase();
    sniff.starts_with("<!doctype html") || sniff.starts_with("<html")
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusEnvelope {
    #[serde(rename = "Cellosaurus")]
    pub cellosaurus: CellosaurusBody,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct CellosaurusBody {
    #[serde(default, rename = "cell-line-list")]
    pub cell_line_list: Vec<CellosaurusRecord>,
    #[serde(default)]
    pub header: Option<CellosaurusHeader>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusHeader {
    #[serde(default)]
    pub release: Option<CellosaurusReleaseBody>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusReleaseBody {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub updated: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct CellosaurusRecord {
    #[serde(default, rename = "accession-list")]
    pub accession_list: Vec<CellosaurusTypedValue>,
    #[serde(default, rename = "name-list")]
    pub name_list: Vec<CellosaurusTypedValue>,
    #[serde(default, rename = "species-list")]
    pub species_list: Vec<CellosaurusXref>,
    #[serde(default, rename = "disease-list")]
    pub disease_list: Vec<CellosaurusXref>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub sex: Option<String>,
    #[serde(default)]
    pub age: Option<String>,
    #[serde(default, rename = "xref-list")]
    pub xref_list: Vec<CellosaurusXref>,
    #[serde(default, rename = "sequence-variation-list")]
    pub sequence_variation_list: Vec<CellosaurusVariation>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusTypedValue {
    #[serde(default, rename = "type")]
    pub value_type: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusXref {
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub accession: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusVariation {
    #[serde(default, rename = "variation-type")]
    pub variation_type: Option<String>,
    #[serde(default, rename = "mutation-type")]
    pub mutation_type: Option<String>,
    #[serde(default, rename = "mutation-description")]
    pub mutation_description: Option<String>,
    #[serde(default, rename = "zygosity-type")]
    pub zygosity_type: Option<String>,
    #[serde(default, rename = "source-list")]
    pub source_list: Vec<CellosaurusVariationSource>,
    #[serde(default, rename = "xref-list")]
    pub xref_list: Vec<CellosaurusXref>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusVariationSource {
    #[serde(default)]
    pub reference: Option<CellosaurusVariationReference>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct CellosaurusVariationReference {
    #[serde(default, rename = "resource-internal-ref")]
    pub resource_internal_ref: Option<String>,
}

impl CellosaurusRecord {
    pub(crate) fn primary_accession(&self) -> Option<&str> {
        self.accession_list
            .iter()
            .find(|entry| entry.value_type.as_deref() == Some("primary"))
            .and_then(|entry| entry.value.as_deref())
    }

    pub(crate) fn secondary_accessions(&self) -> Vec<String> {
        self.accession_list
            .iter()
            .filter(|entry| entry.value_type.as_deref() != Some("primary"))
            .filter_map(|entry| entry.value.clone())
            .collect()
    }

    pub(crate) fn identifier(&self) -> Option<&str> {
        self.name_list
            .iter()
            .find(|entry| entry.value_type.as_deref() == Some("identifier"))
            .and_then(|entry| entry.value.as_deref())
    }

    pub(crate) fn synonyms(&self) -> Vec<String> {
        self.name_list
            .iter()
            .filter(|entry| entry.value_type.as_deref() == Some("synonym"))
            .filter_map(|entry| entry.value.clone())
            .collect()
    }
}

impl CellosaurusVariationSource {
    /// A PubMed ID held in `resource-internal-ref`, published as `PubMed=NNN`.
    pub(crate) fn pubmed_id(&self) -> Option<String> {
        self.reference
            .as_ref()?
            .resource_internal_ref
            .as_deref()?
            .strip_prefix("PubMed=")
            .map(str::to_string)
    }
}

/// The cross-reference databases the cell line card joins on. Every other
/// Cellosaurus cross-reference stays out of the card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellLineXrefDatabase {
    DepMap,
    CosmicClp,
    Chembl,
    CellModelPassport,
    Gdsc,
    PharmacoDb,
    LincsLdp,
}

impl CellLineXrefDatabase {
    /// Map a Cellosaurus `database` name to a join key, or `None` when the
    /// cross-reference is not one the card carries.
    pub(crate) fn from_name(name: &str) -> Option<Self> {
        match name {
            "DepMap" => Some(Self::DepMap),
            "Cosmic-CLP" => Some(Self::CosmicClp),
            "ChEMBL-Cells" => Some(Self::Chembl),
            "Cell_Model_Passport" => Some(Self::CellModelPassport),
            "GDSC" => Some(Self::Gdsc),
            "PharmacoDB" => Some(Self::PharmacoDb),
            "LINCS_LDP" => Some(Self::LincsLdp),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
