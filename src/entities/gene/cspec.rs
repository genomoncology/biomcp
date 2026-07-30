use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::cache::{
    CspecCaptureBinding, ProviderCaptureError, ProviderCaptureManifest, ProviderCaptureStore,
};
use crate::error::BioMcpError;
use crate::sources::clingen_cspec::CspecClient;

const FIELD_LIMIT: usize = 16 * 1024;

#[derive(Debug)]
struct CspecDocumentIri {
    url: reqwest::Url,
    raw: String,
    specification_id: String,
}

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
    pub gene: String,
    pub disease: Option<String>,
    pub vcep: Option<String>,
    pub status: Option<String>,
    pub current: bool,
    pub criteria: Vec<CspecCriterion>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
    pub semantic_subset_version: &'static str,
    pub semantic_subset_sha256: String,
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
    pub capture_binding: CspecCaptureBinding,
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
        let bytes = client.document(&selected.url).await?;
        let binding = CspecCaptureBinding {
            binding_schema_version: 1,
            normalized_gene: gene.to_ascii_uppercase(),
            resource_iri: selected.raw.clone(),
            specification_id: selected.specification_id.clone(),
        };
        let capture = capture(&bytes, binding.clone())?;
        let stored_bytes = read_capture(&capture.capture_id)?;
        return Ok(serde_json::to_value(page_from_bytes(
            &stored_bytes,
            &binding,
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
    gene: &str,
    offset: usize,
    limit: usize,
) -> Result<CspecResponse, BioMcpError> {
    validate_page(offset, limit)?;
    let store = store()?;
    let bytes = store.read(capture_id).map_err(capture_error)?;
    let manifest = store.read_manifest(capture_id).map_err(capture_error)?;
    let binding = manifest
        .capture_binding
        .as_ref()
        .ok_or_else(|| capture_error(ProviderCaptureError::Corrupt))?;
    validate_binding(binding)?;
    if binding.normalized_gene != gene.to_ascii_uppercase() {
        return Err(BioMcpError::InvalidArgument(
            "CSpec capture does not match the requested gene".into(),
        ));
    }
    page_from_bytes(&bytes, binding, offset, limit, &manifest)
}

pub(crate) fn read_capture(capture_id: &str) -> Result<Vec<u8>, BioMcpError> {
    let store = store()?;
    let bytes = store.read(capture_id).map_err(capture_error)?;
    let manifest = store.read_manifest(capture_id).map_err(capture_error)?;
    validate_binding(
        manifest
            .capture_binding
            .as_ref()
            .ok_or(BioMcpError::CaptureCorrupt)?,
    )?;
    Ok(bytes)
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
        resource_iris.push(parsed.raw);
    }
    Ok(CspecManifestResponse {
        gene: gene.into(),
        resource_iris,
        provider: "ClinGen CSpec",
    })
}

fn select(manifest: &[String], value: &str) -> Result<CspecDocumentIri, BioMcpError> {
    let selected = parse_iri(value)?;
    if !manifest.iter().any(|candidate| candidate == value) {
        return Err(BioMcpError::NotFound {
            entity: "CSpec version IRI".into(),
            id: value.into(),
            suggestion: "rerun the manifest command and select one exact resource_iris value"
                .into(),
        });
    }
    let normalized_matches = manifest
        .iter()
        .filter_map(|candidate| parse_iri(candidate).ok())
        .filter(|candidate| candidate.url == selected.url)
        .count();
    if normalized_matches != 1 {
        return Err(invalid("duplicate normalized CSpec manifest IRI"));
    }
    Ok(selected)
}

fn parse_iri(value: &str) -> Result<CspecDocumentIri, BioMcpError> {
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
    let specification_id = parts[3].to_owned();
    Ok(CspecDocumentIri {
        url,
        raw: value.to_owned(),
        specification_id,
    })
}

fn capture(
    bytes: &[u8],
    binding: CspecCaptureBinding,
) -> Result<ProviderCaptureManifest, BioMcpError> {
    store()?
        .capture_cspec_bytes(binding, bytes)
        .map_err(capture_error)
}
fn store() -> Result<ProviderCaptureStore, BioMcpError> {
    Ok(ProviderCaptureStore::new(
        crate::cache::resolve_cache_config()?.cache_root,
    ))
}
fn capture_error(error: ProviderCaptureError) -> BioMcpError {
    match error {
        ProviderCaptureError::Unavailable => BioMcpError::CaptureUnavailable,
        ProviderCaptureError::Corrupt => BioMcpError::CaptureCorrupt,
        ProviderCaptureError::BindingConflict => BioMcpError::BindingConflict,
        ProviderCaptureError::UnsupportedProvider | ProviderCaptureError::Oversize => {
            BioMcpError::InvalidArgument("CSpec capture cannot be stored".into())
        }
    }
}
fn validate_binding(binding: &CspecCaptureBinding) -> Result<(), BioMcpError> {
    if binding.binding_schema_version != 1
        || binding.normalized_gene.is_empty()
        || binding.normalized_gene != binding.normalized_gene.to_ascii_uppercase()
        || binding.resource_iri.is_empty()
        || binding.specification_id.is_empty()
    {
        return Err(BioMcpError::CaptureCorrupt);
    }
    Ok(())
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

fn processing() -> BioMcpError {
    BioMcpError::InternalProcessing
}

fn page_from_bytes(
    bytes: &[u8],
    binding: &CspecCaptureBinding,
    offset: usize,
    limit: usize,
    capture: &ProviderCaptureManifest,
) -> Result<CspecResponse, BioMcpError> {
    let value: Value = serde_json::from_slice(bytes).map_err(|_| processing())?;
    let data = value
        .get("data")
        .and_then(Value::as_object)
        .ok_or_else(processing)?;
    let specification_id = data
        .get("entId")
        .and_then(Value::as_str)
        .ok_or_else(processing)?;
    let content = data.get("entContent").and_then(Value::as_object);
    let ld = data.get("ld").and_then(Value::as_object);
    if specification_id != binding.specification_id
        || data
            .get("@id")
            .is_some_and(|value| value.as_str() != Some(binding.resource_iri.as_str()))
        || content
            .and_then(|value| value.get("namespace"))
            .and_then(Value::as_str)
            != Some(specification_id)
    {
        return Err(processing());
    }
    if data.get("entType").and_then(Value::as_str) != Some("SequenceVariantInterpretation")
        || ld.is_none()
        || !data.get("ldFor").is_some_and(Value::is_object)
    {
        return Err(processing());
    }
    let display_version: String = content
        .and_then(|value| value.get("version"))
        .and_then(Value::as_str)
        .ok_or_else(processing)?
        .into();
    let status = content
        .and_then(|value| value.get("states"))
        .and_then(Value::as_array)
        .and_then(|states| {
            states
                .iter()
                .find(|state| state.get("current") == Some(&Value::Bool(true)))
        })
        .and_then(|state| state.get("name"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let vcep = data
        .get("ldFor")
        .and_then(|value| value.get("Organization"))
        .and_then(Value::as_array)
        .and_then(|organizations| organizations.first())
        .and_then(|organization| organization.get("entContent"))
        .and_then(|content| content.get("shortTitle"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let all = ld
        .and_then(|value| value.get("CriteriaCode"))
        .and_then(Value::as_array)
        .ok_or_else(processing)?;
    let parsed_criteria = all
        .iter()
        .enumerate()
        .map(|(index, row)| criterion(row, index, capture))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| processing())?;
    let semantic_criteria = parsed_criteria
        .iter()
        .map(|(_, semantic)| semantic.clone())
        .collect::<Vec<_>>();
    let semantic_subset_sha256 = semantic_subset_sha256(
        &binding.specification_id,
        &binding.resource_iri,
        &display_version,
        vcep.as_deref(),
        status.as_deref(),
        &semantic_criteria,
    )
    .map_err(|_| processing())?;
    let total = parsed_criteria.len();
    let criteria = parsed_criteria
        .into_iter()
        .map(|(criterion, _)| criterion)
        .skip(offset)
        .take(limit)
        .collect();
    Ok(CspecResponse {
        resource_iri: binding.resource_iri.clone(),
        specification_id: specification_id.into(),
        display_version,
        gene: binding.normalized_gene.clone(),
        disease: None,
        vcep,
        current: status.is_some(),
        status,
        offset,
        limit,
        total,
        criteria,
        semantic_subset_version: "cspec-semantic-v1",
        semantic_subset_sha256,
        capture: provenance(capture),
    })
}
fn provenance(m: &ProviderCaptureManifest) -> CaptureProvenance {
    CaptureProvenance {
        capture_id: m.capture_id.clone(),
        source_sha256: m.sha256.clone(),
        byte_length: m.byte_length,
        media_type: m.media_type.clone(),
        captured_at: m.captured_at,
        expires_at: m.expires_at,
        capture_binding: m
            .capture_binding
            .clone()
            .expect("CSpec pages require a stored capture binding"),
    }
}
fn criterion(
    row: &Value,
    index: usize,
    capture: &ProviderCaptureManifest,
) -> Result<(CspecCriterion, Value), BioMcpError> {
    if row.get("entType").and_then(Value::as_str) != Some("CriteriaCode")
        || !row.get("entContent").is_some_and(Value::is_object)
    {
        return Err(invalid("invalid CSpec criterion row"));
    }
    let content = row.get("entContent").expect("checked object");
    let mut truncated_fields = Vec::new();
    let text = |key: &str, truncated: &mut Vec<String>| {
        content
            .get(key)
            .and_then(Value::as_str)
            .map(|v| bounded(v, key, truncated))
    };
    let mut citations = Vec::new();
    let mut semantic_citations = Vec::new();
    if let Some(references) = content.get("references").filter(|value| !value.is_null()) {
        for reference in references
            .as_array()
            .ok_or_else(|| invalid("criterion references must be an array"))?
        {
            let object = reference
                .as_object()
                .ok_or_else(|| invalid("criterion reference must be an object"))?;
            if !object
                .keys()
                .all(|key| matches!(key.as_str(), "source" | "url" | "id"))
                || object.get("source").and_then(Value::as_str).is_none()
                || object.get("url").and_then(Value::as_str).is_none()
                || object.get("id").is_some_and(|value| !value.is_string())
            {
                return Err(invalid("invalid criterion reference"));
            }
            let url = object
                .get("url")
                .and_then(Value::as_str)
                .expect("checked URL");
            if !semantic_citations.iter().any(|citation| citation == url) {
                semantic_citations.push(url.to_owned());
                if citations.len() == 32 {
                    if !truncated_fields.iter().any(|field| field == "citations") {
                        truncated_fields.push("citations".into());
                    }
                } else {
                    citations.push(bounded(url, "citations", &mut truncated_fields));
                }
            }
        }
    }
    let criterion = CspecCriterion {
        source_id: row.get("entId").and_then(Value::as_str).map(str::to_owned),
        code: text("sepioID", &mut truncated_fields),
        label: text("label", &mut truncated_fields),
        source_text: text("instructionsToUse", &mut truncated_fields),
        source_strength: text("baseStrength", &mut truncated_fields),
        configuration: text("defaultStrength", &mut truncated_fields),
        thresholds: text("originalACMGSummary", &mut truncated_fields),
        assay_restrictions: text("additionalComments", &mut truncated_fields),
        citations,
        source_locator: format!("/data/ld/CriteriaCode/{index}"),
        capture_hash: capture.sha256.clone(),
        truncated_fields,
    };
    let semantic = json!({
        "source_id": row.get("entId").and_then(Value::as_str),
        "code": content.get("sepioID").and_then(Value::as_str),
        "label": content.get("label").and_then(Value::as_str),
        "source_text": content.get("instructionsToUse").and_then(Value::as_str),
        "source_strength": content.get("baseStrength").and_then(Value::as_str),
        "configuration": content.get("defaultStrength").and_then(Value::as_str),
        "thresholds": content.get("originalACMGSummary").and_then(Value::as_str),
        "assay_restrictions": content.get("additionalComments").and_then(Value::as_str),
        "citations": semantic_citations,
    });
    Ok((criterion, semantic))
}

fn semantic_subset_sha256(
    specification_id: &str,
    resource_iri: &str,
    display_version: &str,
    vcep: Option<&str>,
    status: Option<&str>,
    criteria: &[Value],
) -> Result<String, BioMcpError> {
    let value = json!({
        "specification_id": specification_id,
        "resource_iri": resource_iri,
        "display_version": display_version,
        "vcep": vcep,
        "status": status,
        "criteria": criteria,
    });
    let bytes = serde_json::to_vec(&value).map_err(BioMcpError::Json)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}
fn bounded(value: &str, field: &str, truncated: &mut Vec<String>) -> String {
    if value.len() <= FIELD_LIMIT {
        return value.into();
    }
    let end = value
        .char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= FIELD_LIMIT)
        .last()
        .expect("non-empty strings have a UTF-8 boundary at zero");
    truncated.push(field.into());
    value[..end].into()
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{FIELD_LIMIT, bounded, page_from_bytes, select};
    use crate::cache::{CspecCaptureBinding, ProviderCaptureManifest, ProviderCaptureProvider};
    use crate::error::BioMcpError;

    #[test]
    fn selection_requires_the_literal_manifest_iri() {
        let manifest = vec![
            "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
                .to_owned(),
        ];

        assert!(
            select(
                &manifest,
                "https://CSPEC.GENOME.NET/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1",
            )
            .is_err(),
            "normalized spelling must not select a manifest document"
        );
    }

    #[test]
    fn selected_document_must_match_the_manifest_specification_id() {
        let bytes = serde_json::to_vec(&json!({
            "status": { "code": 200 },
            "metadata": {},
            "data": {
                "entType": "SequenceVariantInterpretation",
                "entId": "GN999",
                "entContent": { "namespace": "GN999", "version": "1" },
                "ld": { "CriteriaCode": [] },
                "ldFor": {}
            }
        }))
        .expect("fixture serializes");
        let capture = ProviderCaptureManifest {
            capture_id: "capture:cspec:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            provider: ProviderCaptureProvider::Cspec,
            media_type: "application/json".into(),
            byte_length: bytes.len() as u64,
            sha256: "a".repeat(64),
            captured_at: 0,
            expires_at: 1,
            schema_version: 1,
            capture_binding: Some(CspecCaptureBinding {
                binding_schema_version: 1,
                normalized_gene: "ATM".into(),
                resource_iri: "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1".into(),
                specification_id: "GN020".into(),
            }),
        };

        assert!(
            page_from_bytes(
                &bytes,
                capture.capture_binding.as_ref().expect("binding"),
                0,
                25,
                &capture,
            )
            .is_err(),
            "a malformed provider document must not be projected under a different manifest specification"
        );
        assert!(matches!(
            page_from_bytes(
                &bytes,
                capture.capture_binding.as_ref().expect("binding"),
                0,
                25,
                &capture,
            ),
            Err(BioMcpError::InternalProcessing)
        ));
    }

    #[test]
    fn captured_atm_document_without_data_iri_pages_from_manifest_binding() {
        let bytes = include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_cspec/atm-gn020-1.5.1.json"
        ));
        let capture = ProviderCaptureManifest {
            capture_id: "capture:cspec:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            provider: ProviderCaptureProvider::Cspec,
            media_type: "application/json".into(),
            byte_length: bytes.len() as u64,
            sha256: "a".repeat(64),
            captured_at: 0,
            expires_at: 1,
            schema_version: 1,
            capture_binding: Some(CspecCaptureBinding {
                binding_schema_version: 1,
                normalized_gene: "ATM".into(),
                resource_iri: "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1".into(),
                specification_id: "GN020".into(),
            }),
        };

        let document: serde_json::Value =
            serde_json::from_slice(bytes).expect("captured provider fixture JSON");
        assert!(
            document["data"].get("@id").is_none(),
            "the real provider shape must omit data.@id"
        );

        let page = page_from_bytes(
            bytes,
            capture.capture_binding.as_ref().expect("binding"),
            0,
            25,
            &capture,
        )
        .expect("a captured provider document selected from the manifest must page");

        assert_eq!(
            page.resource_iri,
            capture
                .capture_binding
                .as_ref()
                .expect("binding")
                .resource_iri
        );
        assert_eq!(page.specification_id, "GN020");
        assert_eq!(page.display_version, "1.5");
        assert!(page.total > 0, "the captured document must yield criteria");
        assert!(
            page.criteria
                .iter()
                .any(|criterion| criterion.label.as_deref() == Some("BP6")),
            "the captured criterion landmark must be parsed"
        );
    }

    #[test]
    fn post_fetch_paging_failure_is_not_projected_as_a_clingen_provider_failure() {
        let mut document: serde_json::Value = serde_json::from_slice(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/testdata/sources/clingen_cspec/atm-gn020-1.5.1.json"
        )))
        .expect("fixture JSON");
        document["data"]["@id"] = json!(
            "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1"
        );
        document["data"]["ld"]["CriteriaCode"] = json!({});
        let bytes = serde_json::to_vec(&document).expect("fixture serializes");
        let capture = ProviderCaptureManifest {
            capture_id: "capture:cspec:sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            provider: ProviderCaptureProvider::Cspec,
            media_type: "application/json".into(),
            byte_length: bytes.len() as u64,
            sha256: "a".repeat(64),
            captured_at: 0,
            expires_at: 1,
            schema_version: 1,
            capture_binding: Some(CspecCaptureBinding {
                binding_schema_version: 1,
                normalized_gene: "ATM".into(),
                resource_iri: "https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1".into(),
                specification_id: "GN020".into(),
            }),
        };

        let error = page_from_bytes(
            &bytes,
            capture.capture_binding.as_ref().expect("binding"),
            0,
            25,
            &capture,
        )
        .expect_err("a malformed captured document must not page");
        let projection = error.public_projection();

        assert_eq!(projection.source, None);
        assert_eq!(projection.recovery, None);
    }

    #[test]
    fn bounded_preserves_utf8_when_the_limit_splits_a_character() {
        let input = format!("{}é", "a".repeat(FIELD_LIMIT - 1));
        let mut truncated = Vec::new();

        let result = bounded(&input, "source_text", &mut truncated);

        assert_eq!(result, "a".repeat(FIELD_LIMIT - 1));
        assert_eq!(truncated, ["source_text"]);
    }
}
