use serde::Serialize;
use serde_json::Value;

use crate::cache::{
    ProviderCaptureError, ProviderCaptureManifest, ProviderCaptureProvider, ProviderCaptureStore,
};
use crate::error::BioMcpError;
use crate::sources::clingen_cspec::CspecClient;

const FIELD_LIMIT: usize = 16 * 1024;

#[derive(Debug, Serialize)]
pub(crate) struct CspecManifestResponse {
    pub gene: String,
    pub resource_iris: Vec<String>,
    pub provider: &'static str,
}
#[derive(Debug, Serialize)]
pub(crate) struct CspecResponse {
    pub resource_iri: String,
    pub specification_id: String,
    pub display_version: String,
    pub criteria: Vec<CspecCriterion>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
    #[serde(flatten)]
    pub capture: CaptureProvenance,
}
#[derive(Debug, Serialize)]
pub(crate) struct CaptureProvenance {
    pub capture_id: String,
    pub source_sha256: String,
    pub byte_length: u64,
    pub media_type: String,
    pub captured_at: u64,
    pub expires_at: u64,
}
#[derive(Debug, Serialize)]
pub(crate) struct CspecCriterion {
    pub source_id: Option<String>,
    pub code: Option<String>,
    pub label: Option<String>,
    pub source_text: Option<String>,
    pub source_strength: Option<String>,
    pub configuration: Option<String>,
    pub thresholds: Option<String>,
    pub assay_restrictions: Option<String>,
    pub citations: Vec<String>,
    pub source_locator: String,
    pub capture_hash: String,
    pub truncated_fields: Vec<String>,
}

pub(crate) async fn retrieve(
    gene: &str,
    version_iri: Option<&str>,
    offset: usize,
    limit: usize,
) -> Result<serde_json::Value, BioMcpError> {
    validate_page(offset, limit)?;
    let client = CspecClient::new()?;
    let manifest = manifest(gene, &client).await?;
    if let Some(iri) = version_iri {
        let selected = select(&manifest.resource_iris, iri)?;
        let bytes = client.document(&selected).await?;
        let capture = capture(&bytes)?;
        return Ok(serde_json::to_value(page_from_bytes(
            &bytes,
            selected.as_str(),
            offset,
            limit,
            &capture,
        )?)
        .expect("CSpec response serializes"));
    }
    Ok(serde_json::to_value(manifest).expect("CSpec manifest serializes"))
}

pub(crate) fn page_capture(
    capture_id: &str,
    offset: usize,
    limit: usize,
) -> Result<CspecResponse, BioMcpError> {
    validate_page(offset, limit)?;
    let store = store()?;
    let bytes = store.read(capture_id).map_err(capture_error)?;
    let manifest = capture_manifest(capture_id, &bytes)?;
    let iri = document_iri_from_bytes(&bytes)?;
    page_from_bytes(&bytes, &iri, offset, limit, &manifest)
}

pub(crate) fn read_capture(capture_id: &str) -> Result<Vec<u8>, BioMcpError> {
    store()?.read(capture_id).map_err(capture_error)
}

async fn manifest(gene: &str, client: &CspecClient) -> Result<CspecManifestResponse, BioMcpError> {
    let value = client.manifest(gene).await?;
    let rows = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("manifest data must be an array"))?;
    let mut resource_iris = Vec::new();
    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| invalid("manifest row must be an object"))?;
        if object.len() != 1 {
            return Err(invalid("manifest row must contain exactly one @id"));
        }
        let iri = object
            .get("@id")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("manifest row @id is required"))?;
        let parsed = parse_iri(iri)?;
        resource_iris.push(parsed.to_string());
    }
    Ok(CspecManifestResponse {
        gene: gene.into(),
        resource_iris,
        provider: "ClinGen CSpec",
    })
}

fn select(manifest: &[String], value: &str) -> Result<reqwest::Url, BioMcpError> {
    let selected = parse_iri(value)?;
    let matches = manifest
        .iter()
        .filter(|candidate| *candidate == selected.as_str())
        .count();
    match matches {
        1 => Ok(selected),
        0 => Err(BioMcpError::NotFound {
            entity: "CSpec version IRI".into(),
            id: value.into(),
            suggestion: "rerun the manifest command and select one exact resource_iris value"
                .into(),
        }),
        _ => Err(invalid("duplicate normalized CSpec manifest IRI")),
    }
}

fn parse_iri(value: &str) -> Result<reqwest::Url, BioMcpError> {
    let url = reqwest::Url::parse(value)
        .map_err(|_| invalid("CSpec version must be a full resource IRI"))?;
    if url.scheme() != "https"
        || url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !matches!(
            url.host_str(),
            Some("cspec.genome.network" | "cspec.clinicalgenome.org")
        )
    {
        return Err(invalid("invalid CSpec resource IRI"));
    }
    let parts = url
        .path_segments()
        .map(|x| x.collect::<Vec<_>>())
        .unwrap_or_default();
    if parts.len() != 6
        || parts[0] != "cspec"
        || parts[1] != "SequenceVariantInterpretation"
        || parts[2] != "id"
        || parts[3].is_empty()
        || parts[4] != "version"
        || parts[5].is_empty()
    {
        return Err(invalid("invalid CSpec resource IRI path"));
    }
    Ok(url)
}

fn capture(bytes: &[u8]) -> Result<ProviderCaptureManifest, BioMcpError> {
    store()?
        .capture_bytes(ProviderCaptureProvider::Cspec, "application/json", bytes)
        .map_err(capture_error)
}
fn store() -> Result<ProviderCaptureStore, BioMcpError> {
    Ok(ProviderCaptureStore::new(
        crate::cache::resolve_cache_config()?.cache_root,
    ))
}
fn capture_manifest(
    capture_id: &str,
    bytes: &[u8],
) -> Result<ProviderCaptureManifest, BioMcpError> {
    let store = store()?;
    let manifest = store
        .capture_bytes(ProviderCaptureProvider::Cspec, "application/json", bytes)
        .map_err(capture_error)?;
    if manifest.capture_id != capture_id {
        return Err(capture_error(ProviderCaptureError::Corrupt));
    }
    Ok(manifest)
}
fn capture_error(error: ProviderCaptureError) -> BioMcpError {
    BioMcpError::InvalidArgument(format!("CSpec capture is unavailable: {error:?}"))
}
fn validate_page(_offset: usize, limit: usize) -> Result<(), BioMcpError> {
    if !(1..=50).contains(&limit) {
        Err(BioMcpError::InvalidArgument(
            "CSpec --limit must be between 1 and 50".into(),
        ))
    } else {
        Ok(())
    }
}
fn invalid(message: &str) -> BioMcpError {
    BioMcpError::Api {
        api: "ClinGen CSpec".into(),
        message: message.into(),
    }
}

fn page_from_bytes(
    bytes: &[u8],
    iri: &str,
    offset: usize,
    limit: usize,
    capture: &ProviderCaptureManifest,
) -> Result<CspecResponse, BioMcpError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: "ClinGen CSpec".into(),
        source,
    })?;
    let data = value
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(|| invalid("document data must be an object"))?;
    let specification_id = data
        .get("entId")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("document entId is required"))?;
    let content = data.get("entContent").and_then(Value::as_object);
    let ld = data.get("ld").and_then(Value::as_object);
    if data.get("entType").and_then(Value::as_str) != Some("SequenceVariantInterpretation")
        || content
            .and_then(|value| value.get("namespace"))
            .and_then(Value::as_str)
            != Some(specification_id)
        || ld.is_none()
        || !data.get("ldFor").is_some_and(Value::is_object)
    {
        return Err(invalid("invalid CSpec document identity"));
    }
    let display_version = content
        .and_then(|value| value.get("version"))
        .and_then(Value::as_str)
        .ok_or_else(|| invalid("document display version is required"))?
        .into();
    let all = ld
        .and_then(|value| value.get("CriteriaCode"))
        .and_then(Value::as_array)
        .ok_or_else(|| invalid("document CriteriaCode must be an array"))?;
    let criteria = all
        .iter()
        .enumerate()
        .filter_map(|(index, row)| criterion(row, index, capture))
        .skip(offset)
        .take(limit)
        .collect::<Vec<_>>();
    Ok(CspecResponse {
        resource_iri: iri.into(),
        specification_id: specification_id.into(),
        display_version,
        offset,
        limit,
        total: all.len(),
        criteria,
        capture: provenance(capture),
    })
}
fn document_iri_from_bytes(bytes: &[u8]) -> Result<String, BioMcpError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|source| BioMcpError::ApiJson {
        api: "ClinGen CSpec".into(),
        source,
    })?;
    value
        .pointer("/data/@id")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| invalid("captured document has no resource IRI"))
}
fn provenance(m: &ProviderCaptureManifest) -> CaptureProvenance {
    CaptureProvenance {
        capture_id: m.capture_id.clone(),
        source_sha256: m.sha256.clone(),
        byte_length: m.byte_length,
        media_type: m.media_type.clone(),
        captured_at: m.captured_at,
        expires_at: m.expires_at,
    }
}
fn criterion(
    row: &Value,
    index: usize,
    capture: &ProviderCaptureManifest,
) -> Option<CspecCriterion> {
    if row.get("entType").and_then(Value::as_str) != Some("CriteriaCode")
        || !row.get("entContent").is_some_and(Value::is_object)
    {
        return None;
    }
    let content = row.get("entContent")?;
    let mut truncated_fields = Vec::new();
    let text = |key: &str, truncated: &mut Vec<String>| {
        content
            .get(key)
            .and_then(Value::as_str)
            .map(|v| bounded(v, key, truncated))
    };
    Some(CspecCriterion {
        source_id: row.get("entId").and_then(Value::as_str).map(str::to_owned),
        code: text("sepioID", &mut truncated_fields),
        label: text("label", &mut truncated_fields),
        source_text: text("instructionsToUse", &mut truncated_fields),
        source_strength: text("baseStrength", &mut truncated_fields),
        configuration: text("defaultStrength", &mut truncated_fields),
        thresholds: text("originalACMGSummary", &mut truncated_fields),
        assay_restrictions: text("additionalComments", &mut truncated_fields),
        citations: content
            .get("references")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect(),
        source_locator: format!("/data/ld/CriteriaCode/{index}"),
        capture_hash: capture.sha256.clone(),
        truncated_fields,
    })
}
fn bounded(value: &str, field: &str, truncated: &mut Vec<String>) -> String {
    if value.len() > FIELD_LIMIT {
        truncated.push(field.into());
        value[..FIELD_LIMIT].to_string()
    } else {
        value.into()
    }
}
