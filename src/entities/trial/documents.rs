use crate::error::BioMcpError;
use crate::sources::clinicaltrials::{ClinicalTrialsClient, CtGovLargeDocument, CtGovStudy};

const CTGOV_CDN_BASE: &str = "https://cdn.clinicaltrials.gov";
const CTGOV_CDN_BASE_ENV: &str = "BIOMCP_CTGOV_CDN_BASE";
const DOCUMENT_MAX_BYTES: usize = 32 * 1024 * 1024;
const CTGOV_CDN_API: &str = "ClinicalTrials.gov document CDN";

#[derive(Debug, Clone, serde::Serialize)]
pub struct TrialDocumentsManifest {
    pub nct_id: String,
    pub source: String,
    pub documents: Vec<TrialDocument>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TrialDocument {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_protocol: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_sap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_icf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TrialEligibilityProvenance {
    pub source_kind: String,
    pub source: String,
    pub posted_documents_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents_handle: Option<String>,
}

pub async fn trial_documents_manifest(nct_id: &str) -> Result<TrialDocumentsManifest, BioMcpError> {
    let nct_id = super::get::validated_nct_id(nct_id)?;
    let study = ClinicalTrialsClient::new()?
        .get(&nct_id, &["documents".to_string()])
        .await?;
    Ok(manifest_from_study(&nct_id, &study))
}

pub async fn trial_document_bytes(nct_id: &str, filename: &str) -> Result<Vec<u8>, BioMcpError> {
    let manifest = trial_documents_manifest(nct_id).await?;
    if !is_advertised(&manifest, filename) {
        return Err(BioMcpError::NotFound {
            entity: "trial document".into(),
            id: filename.to_string(),
            suggestion: format!(
                "List documents: biomcp --json get trial {} documents",
                manifest.nct_id
            ),
        });
    }
    validate_filename(filename)?;
    download_document(&manifest.nct_id, filename).await
}

pub(super) fn eligibility_provenance(
    nct_id: &str,
    study: &CtGovStudy,
) -> TrialEligibilityProvenance {
    let available = large_documents(study).next().is_some();
    TrialEligibilityProvenance {
        source_kind: "registry".into(),
        source: "ClinicalTrials.gov registry".into(),
        posted_documents_available: available,
        documents_handle: available.then(|| documents_command(nct_id)),
    }
}

fn is_advertised(manifest: &TrialDocumentsManifest, filename: &str) -> bool {
    manifest
        .documents
        .iter()
        .any(|document| document.filename.as_deref() == Some(filename))
}

fn manifest_from_study(nct_id: &str, study: &CtGovStudy) -> TrialDocumentsManifest {
    TrialDocumentsManifest {
        nct_id: nct_id.to_string(),
        source: "ClinicalTrials.gov".into(),
        documents: large_documents(study)
            .map(|document| map_document(nct_id, document))
            .collect(),
    }
}

fn large_documents(study: &CtGovStudy) -> impl Iterator<Item = &CtGovLargeDocument> {
    study
        .document_section
        .as_ref()
        .and_then(|section| section.large_document_module.as_ref())
        .into_iter()
        .flat_map(|module| module.large_docs.iter())
}

fn map_document(nct_id: &str, document: &CtGovLargeDocument) -> TrialDocument {
    let filename = document.filename.clone();
    let handle = filename
        .as_deref()
        .filter(|value| validate_filename(value).is_ok())
        .map(|value| document_command(nct_id, value));
    TrialDocument {
        document_type: document.type_abbrev.clone(),
        label: document.label.clone(),
        date: document.date.clone(),
        upload_date: document.upload_date.clone(),
        filename,
        size_bytes: document.size,
        has_protocol: document.has_protocol,
        has_sap: document.has_sap,
        has_icf: document.has_icf,
        handle,
    }
}

fn documents_command(nct_id: &str) -> String {
    crate::next_command::NextCommand::biomcp()
        .args(["--json", "get", "trial", nct_id, "documents"])
        .render_shell()
}

fn document_command(nct_id: &str, filename: &str) -> String {
    crate::next_command::NextCommand::biomcp()
        .args(["get", "trial", nct_id, "document", filename])
        .render_shell()
}

fn validate_filename(filename: &str) -> Result<(), BioMcpError> {
    let invalid = filename.trim().is_empty()
        || filename == "."
        || filename == ".."
        || filename.contains(['/', '\\', '?', '#', '\0'])
        || reqwest::Url::parse(filename).is_ok();
    if invalid {
        return Err(BioMcpError::InvalidArgument(
            "Trial document filename must be one safe advertised path segment.".into(),
        ));
    }
    Ok(())
}

fn approved_cdn_base() -> Result<reqwest::Url, BioMcpError> {
    let configured = std::env::var(CTGOV_CDN_BASE_ENV).ok();
    let raw = configured.as_deref().unwrap_or(CTGOV_CDN_BASE);
    let url = reqwest::Url::parse(raw).map_err(|_| {
        BioMcpError::InvalidArgument(format!("{CTGOV_CDN_BASE_ENV} must be a valid origin"))
    })?;
    let loopback_host = url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    });
    if configured.is_some() && (!matches!(url.scheme(), "http" | "https") || !loopback_host) {
        return Err(BioMcpError::InvalidArgument(format!(
            "{CTGOV_CDN_BASE_ENV} is accepted only for a loopback HTTP(S) origin"
        )));
    }
    if url.cannot_be_a_base()
        || url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(BioMcpError::InvalidArgument(format!(
            "{CTGOV_CDN_BASE_ENV} must contain only an HTTP(S) origin"
        )));
    }
    Ok(url)
}

fn same_origin(left: &reqwest::Url, right: &reqwest::Url) -> bool {
    left.scheme() == right.scheme()
        && left.host_str() == right.host_str()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn download_url(base: &reqwest::Url, nct_id: &str, filename: &str) -> reqwest::Url {
    let mut url = base.clone();
    url.path_segments_mut()
        .expect("approved HTTP(S) origin supports path segments")
        .extend(["large-docs", &nct_id[9..], nct_id, filename]);
    url
}

async fn download_document(nct_id: &str, filename: &str) -> Result<Vec<u8>, BioMcpError> {
    download_document_from_base(approved_cdn_base()?, nct_id, filename).await
}

async fn download_document_from_base(
    base: reqwest::Url,
    nct_id: &str,
    filename: &str,
) -> Result<Vec<u8>, BioMcpError> {
    let approved_origin = base.clone();
    let client = reqwest::Client::builder()
        .gzip(false)
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            if !same_origin(&approved_origin, attempt.url()) {
                attempt.error("trial document redirect left the approved origin")
            } else if attempt.previous().len() >= 10 {
                attempt.error("too many trial document redirects")
            } else {
                attempt.follow()
            }
        }))
        .build()?;
    let response = client
        .get(download_url(&base, nct_id, filename))
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(BioMcpError::Api {
            api: CTGOV_CDN_API.into(),
            message: format!("HTTP {} retrieving trial document", response.status()),
        });
    }
    read_document_body(response).await
}

async fn read_document_body(mut response: reqwest::Response) -> Result<Vec<u8>, BioMcpError> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        let remaining_with_overflow = DOCUMENT_MAX_BYTES
            .saturating_sub(body.len())
            .saturating_add(1);
        body.extend_from_slice(&chunk[..chunk.len().min(remaining_with_overflow)]);
        if body.len() > DOCUMENT_MAX_BYTES {
            return Err(BioMcpError::Api {
                api: CTGOV_CDN_API.into(),
                message: format!("Response body exceeded {DOCUMENT_MAX_BYTES} bytes"),
            });
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    use axum::{
        Router,
        body::Body,
        http::{Response, StatusCode, header},
        routing::get,
    };

    use super::*;

    async fn serve(router: Router) -> (reqwest::Url, tokio::task::JoinHandle<()>) {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let task = tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });
        (
            reqwest::Url::parse(&format!("http://{address}")).unwrap(),
            task,
        )
    }

    fn study_with_documents(documents: serde_json::Value) -> CtGovStudy {
        serde_json::from_value(serde_json::json!({
            "documentSection": {"largeDocumentModule": {"largeDocs": documents}}
        }))
        .unwrap()
    }

    #[test]
    fn maps_manifest_metadata_handles_and_empty_state() {
        let study = study_with_documents(serde_json::json!([{
            "typeAbbrev": "Prot_SAP",
            "label": "Protocol and SAP",
            "date": "2019-07-18",
            "uploadDate": "2024-12-12T10:49",
            "filename": "Protocol final.pdf",
            "size": 50,
            "hasProtocol": true,
            "hasSap": true,
            "hasIcf": false
        }, {
            "filename": "Oversized.pdf",
            "size": 33554433
        }]));
        let manifest = manifest_from_study("NCT03361748", &study);
        assert_eq!(manifest.documents.len(), 2);
        assert_eq!(manifest.documents[0].size_bytes, Some(50));
        assert_eq!(
            manifest.documents[0].handle.as_deref(),
            Some("biomcp get trial NCT03361748 document \"Protocol final.pdf\"")
        );
        assert!(is_advertised(&manifest, "Protocol final.pdf"));
        assert!(!is_advertised(&manifest, "protocol final.pdf"));
        assert!(!is_advertised(&manifest, "Unknown.pdf"));
        assert_eq!(manifest.documents[1].size_bytes, Some(33_554_433));
        assert!(
            manifest_from_study("NCT41300001", &study_with_documents(serde_json::json!([])))
                .documents
                .is_empty()
        );
    }

    #[test]
    fn eligibility_provenance_tracks_document_availability() {
        let available = eligibility_provenance(
            "NCT03361748",
            &study_with_documents(serde_json::json!([{"filename": "Protocol.pdf"}])),
        );
        assert!(available.posted_documents_available);
        assert_eq!(
            available.documents_handle.as_deref(),
            Some("biomcp --json get trial NCT03361748 documents")
        );

        let unavailable =
            eligibility_provenance("NCT41300001", &study_with_documents(serde_json::json!([])));
        assert!(!unavailable.posted_documents_available);
        assert!(unavailable.documents_handle.is_none());
    }

    #[test]
    fn rejects_unsafe_document_filenames() {
        for filename in [
            "",
            ".",
            "..",
            "../x.pdf",
            "x\\y.pdf",
            "x.pdf?q=1",
            "https://example.test/x.pdf",
        ] {
            assert!(
                validate_filename(filename).is_err(),
                "accepted {filename:?}"
            );
        }
        assert!(validate_filename("   ").is_err());
        assert!(validate_filename("Protocol 1%.pdf").is_ok());
    }

    #[test]
    fn omits_handles_for_unsafe_advertised_filenames() {
        let manifest = manifest_from_study(
            "NCT03361748",
            &study_with_documents(serde_json::json!([{"filename": "../Protocol.pdf"}])),
        );
        assert_eq!(
            manifest.documents[0].filename.as_deref(),
            Some("../Protocol.pdf")
        );
        assert!(manifest.documents[0].handle.is_none());
    }

    #[test]
    fn constructs_fixed_percent_encoded_path() {
        let base = reqwest::Url::parse("https://cdn.clinicaltrials.gov").unwrap();
        assert_eq!(
            download_url(&base, "NCT03361748", "Protocol 1%.pdf").as_str(),
            "https://cdn.clinicaltrials.gov/large-docs/48/NCT03361748/Protocol%201%25.pdf"
        );
    }

    #[tokio::test]
    async fn permits_exact_body_limit_and_rejects_one_extra_byte() {
        let router = Router::new()
            .route(
                "/large-docs/48/NCT03361748/exact.pdf",
                get(|| async { vec![b'x'; DOCUMENT_MAX_BYTES] }),
            )
            .route(
                "/large-docs/48/NCT03361748/too-large.pdf",
                get(|| async { vec![b'x'; DOCUMENT_MAX_BYTES + 1] }),
            );
        let (base, task) = serve(router).await;

        let exact = download_document_from_base(base.clone(), "NCT03361748", "exact.pdf")
            .await
            .unwrap();
        assert_eq!(exact.len(), DOCUMENT_MAX_BYTES);
        let error = download_document_from_base(base, "NCT03361748", "too-large.pdf")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("exceeded 33554432 bytes"));
        task.abort();
    }

    #[tokio::test]
    async fn rejects_off_origin_redirect_before_contacting_target() {
        let target_requests = Arc::new(AtomicUsize::new(0));
        let counter = target_requests.clone();
        let target_router = Router::new().fallback(get(move || {
            let counter = counter.clone();
            async move {
                counter.fetch_add(1, Ordering::SeqCst);
                "unexpected"
            }
        }));
        let (target_base, target_task) = serve(target_router).await;
        let location = target_base.join("stolen.pdf").unwrap().to_string();
        let redirect_router = Router::new().fallback(get(move || {
            let location = location.clone();
            async move {
                Response::builder()
                    .status(StatusCode::FOUND)
                    .header(header::LOCATION, location)
                    .body(Body::empty())
                    .unwrap()
            }
        }));
        let (approved_base, redirect_task) = serve(redirect_router).await;

        let error = download_document_from_base(approved_base, "NCT03361748", "Protocol.pdf")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("redirect"));
        assert_eq!(target_requests.load(Ordering::SeqCst), 0);
        redirect_task.abort();
        target_task.abort();
    }
}
