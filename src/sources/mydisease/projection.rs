//! Original-byte disease identity and source-only enrichment custody.
use super::MyDiseaseHpo;
use crate::error::{BioMcpError, SourceContext, SourceProvider};
use biodata::{MyDiseasePage, MyDiseaseProfile, MyDiseaseRow, parse_mydisease};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct MyDiseaseHit {
    pub id: String,
    pub mondo: Option<serde_json::Value>,
    pub disease_ontology: Option<serde_json::Value>,
    pub umls: Option<serde_json::Value>,
    pub disgenet: Option<serde_json::Value>,
    pub hpo: Option<MyDiseaseHpo>,
    pub row: MyDiseaseRow,
    // dead-code reason: retained original response custody until product assembly
    #[allow(dead_code)]
    pub page: Arc<MyDiseasePage>,
    // Presentation conversion is temporary until direct disease-document presentation.
    // Remove it before public release. Source-only enrichment has separate ownership.
    pub conversion: DiseaseConversionReport,
}
#[derive(Debug, Clone, Default)]
pub struct DiseaseConversionReport {
    pub losses: Vec<(&'static str, &'static str)>,
}
#[derive(Debug)]
pub struct MyDiseaseQueryResponse {
    pub total: usize,
    pub hits: Vec<MyDiseaseHit>,
}
#[derive(Deserialize)]
struct Enrichment {
    mondo: Option<serde_json::Value>,
    disease_ontology: Option<serde_json::Value>,
    umls: Option<serde_json::Value>,
    disgenet: Option<serde_json::Value>,
    hpo: Option<MyDiseaseHpo>,
}
pub(crate) fn failure(message: impl Into<String>) -> BioMcpError {
    BioMcpError::Api {
        api: "MyDisease.info".into(),
        message: message.into(),
    }
    .with_source_context(SourceContext::retry(SourceProvider::MYDISEASE))
}
pub(crate) fn validate_transport(
    status: reqwest::StatusCode,
    content_type: Option<&reqwest::header::HeaderValue>,
    bytes: &[u8],
) -> Result<(), BioMcpError> {
    let context = SourceContext::retry(SourceProvider::MYDISEASE);
    if !status.is_success() {
        return Err(failure(format!("HTTP {status}: MyDisease response failed")));
    }
    crate::sources::ensure_json_content_type(context, content_type, bytes)
}
fn convert(row: &MyDiseaseRow, page: Arc<MyDiseasePage>) -> Result<MyDiseaseHit, BioMcpError> {
    let mut enrichment: Enrichment = serde_json::from_str(row.source().raw())
        .map_err(|_| failure("MyDisease source-only enrichment shape failed"))?;
    // Enrichment cannot provide identity. Exact assertions remain in the BioData row.
    for section in [&mut enrichment.mondo, &mut enrichment.disease_ontology] {
        if let Some(object) = section.as_mut().and_then(serde_json::Value::as_object_mut) {
            object.remove("name");
        }
    }
    let (_, losses) = crate::transform::disease::identity_display(row);
    Ok(MyDiseaseHit {
        id: row.identity().provider_id().to_owned(),
        mondo: enrichment.mondo,
        disease_ontology: enrichment.disease_ontology,
        umls: enrichment.umls,
        disgenet: enrichment.disgenet,
        hpo: enrichment.hpo,
        row: row.clone(),
        page,
        conversion: DiseaseConversionReport { losses },
    })
}
fn parse(bytes: &[u8], profile: MyDiseaseProfile) -> Result<Arc<MyDiseasePage>, BioMcpError> {
    let page = parse_mydisease(bytes, profile)
        .map_err(|_| failure("MyDisease identity response failed"))?;
    page.trusted_rows()
        .map_err(|_| failure("MyDisease identity page rejected"))?;
    Ok(Arc::new(page))
}
pub(crate) fn decode_get(bytes: &[u8]) -> Result<MyDiseaseHit, BioMcpError> {
    let page = parse(bytes, MyDiseaseProfile::Get)?;
    let rows = page
        .trusted_rows()
        .map_err(|_| failure("MyDisease identity page rejected"))?;
    let row = rows
        .first()
        .ok_or_else(|| failure("MyDisease detail row missing"))?;
    convert(row, Arc::clone(&page))
}
pub(crate) fn decode_search(bytes: &[u8]) -> Result<MyDiseaseQueryResponse, BioMcpError> {
    let page = parse(bytes, MyDiseaseProfile::Search)?;
    let total = page
        .total()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| failure("MyDisease total is not representable"))?;
    let hits = page
        .trusted_rows()
        .map_err(|_| failure("MyDisease identity page rejected"))?
        .into_iter()
        .map(|row| convert(row, Arc::clone(&page)))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MyDiseaseQueryResponse { total, hits })
}
/// Optional product fallback permits only positively identified unavailable transport.
pub(crate) fn optional_failure(error: &BioMcpError) -> bool {
    match error {
        BioMcpError::WithSourceContext { source, .. } => optional_failure(source),
        BioMcpError::NotFound { .. } => true,
        BioMcpError::Http(error) => unavailable_transport(error),
        BioMcpError::HttpMiddleware(error) => unavailable_middleware(error),
        _ => false,
    }
}
// Synthetic test builders use the adopted parser too. No runtime legacy decoder exists.
#[cfg(test)]
impl<'de> Deserialize<'de> for MyDiseaseHit {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let bytes = serde_json::to_vec(&value).map_err(serde::de::Error::custom)?;
        decode_get(&bytes).map_err(serde::de::Error::custom)
    }
}
#[cfg(test)]
impl<'de> Deserialize<'de> for MyDiseaseQueryResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        let bytes = serde_json::to_vec(&value).map_err(serde::de::Error::custom)?;
        decode_search(&bytes).map_err(serde::de::Error::custom)
    }
}

fn unavailable_transport(error: &reqwest::Error) -> bool {
    error.is_connect()
        || (error.is_timeout()
            && error.status().is_none()
            && !error.is_body()
            && !error.is_decode())
}

fn unavailable_middleware(error: &reqwest_middleware::Error) -> bool {
    match error {
        reqwest_middleware::Error::Reqwest(error) => unavailable_transport(error),
        reqwest_middleware::Error::Middleware(error) => {
            if let Some(retry) = error.downcast_ref::<reqwest_retry::RetryError>() {
                match retry {
                    reqwest_retry::RetryError::WithRetries { err, .. }
                    | reqwest_retry::RetryError::Error(err) => unavailable_middleware(err),
                }
            } else {
                error
                    .chain()
                    .filter_map(|cause| cause.downcast_ref::<reqwest::Error>())
                    .any(unavailable_transport)
            }
        }
    }
}
