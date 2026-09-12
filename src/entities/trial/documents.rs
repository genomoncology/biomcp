use crate::error::BioMcpError;
use crate::sources::RequestBuilderSourceContextExt;
use crate::sources::clinicaltrials::ClinicalTrialsClient;
use crate::sources::provider_url_policy::{ProviderUrlConsumer, ProviderUrlPolicy};
use biodata::{
    ClinicalTrialArtifactDescriptor, ClinicalTrialArtifactId, ClinicalTrialProjection,
    ClinicalTrialSection, ClinicalTrialsGovApiV2Response, ClinicalTrialsGovArtifactRetrieval,
};
use std::collections::BTreeMap;

const CTGOV_CDN_BASE: &str = "https://cdn.clinicaltrials.gov";
const CTGOV_CDN_BASE_ENV: &str = "BIOMCP_CTGOV_CDN_BASE";
const DOCUMENT_MAX_BYTES: usize = 32 * 1024 * 1024;
const DOCUMENT_CONNECT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);
const DOCUMENT_REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);
const CTGOV_CDN_API: &str = "ClinicalTrials.gov document CDN";

#[derive(Clone)]
pub struct TrialDocumentsManifest {
    projection: ClinicalTrialProjection<biodata::Capture>,
    state: super::TrialSectionState,
    retrievals: BTreeMap<ClinicalTrialArtifactId, TrialDocumentRetrieval>,
}

#[derive(Clone)]
struct TrialDocumentRetrieval {
    filename: Option<String>,
    handle: Option<String>,
}

#[derive(serde::Serialize)]
struct TrialDocumentView<'a> {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    document_type: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    upload_date: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filename: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_protocol: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_sap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_icf: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    handle: Option<&'a str>,
}

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct TrialEligibilityProvenance {
    pub source_kind: String,
    pub source: String,
    pub source_authority: String,
    pub provider_record_identity: String,
    pub capture_digest: String,
    pub posted_documents_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents_handle: Option<String>,
}

impl std::fmt::Debug for TrialDocumentsManifest {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrialDocumentsManifest")
            .field("document_count", &self.documents().len())
            .finish_non_exhaustive()
    }
}

impl std::fmt::Debug for TrialEligibilityProvenance {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("TrialEligibilityProvenance")
            .field(
                "posted_documents_available",
                &self.posted_documents_available,
            )
            .finish_non_exhaustive()
    }
}

pub async fn trial_documents_manifest(nct_id: &str) -> Result<TrialDocumentsManifest, BioMcpError> {
    let nct_id = super::get::validated_nct_id(nct_id)?;
    let response = ClinicalTrialsClient::new()?
        .get_biodata_detail(&nct_id, &["documents".to_string()])
        .await?;
    manifest_from_response(response)
}

pub async fn trial_document_bytes(nct_id: &str, filename: &str) -> Result<Vec<u8>, BioMcpError> {
    let manifest = trial_documents_manifest(nct_id).await?;
    document_bytes_from_manifest(&manifest, filename, approved_cdn_base()?).await
}

async fn document_bytes_from_manifest(
    manifest: &TrialDocumentsManifest,
    filename: &str,
    base: reqwest::Url,
) -> Result<Vec<u8>, BioMcpError> {
    if !is_advertised(manifest, filename) {
        return Err(BioMcpError::NotFound {
            entity: "trial document".into(),
            id: "unadvertised filename".into(),
            suggestion: format!(
                "List documents: biomcp --json get trial {} documents",
                manifest.nct_id()
            ),
        });
    }
    validate_filename(filename)?;
    download_document_from_base(base, manifest.nct_id(), filename).await
}

pub(super) fn eligibility_provenance(
    response: &ClinicalTrialsGovApiV2Response,
) -> TrialEligibilityProvenance {
    let capture = response.capture();
    let available =
        matches!(response.artifacts(), ClinicalTrialSection::Present(rows) if !rows.is_empty());
    TrialEligibilityProvenance {
        source_kind: "registry".into(),
        source: "ClinicalTrials.gov registry".into(),
        source_authority: capture.source_authority().to_owned(),
        provider_record_identity: capture.provider_record_identity().to_owned(),
        capture_digest: capture.digest().to_owned(),
        posted_documents_available: available,
        documents_handle: available.then(|| documents_command(capture.provider_record_identity())),
    }
}

fn is_advertised(manifest: &TrialDocumentsManifest, filename: &str) -> bool {
    manifest
        .retrievals
        .values()
        .any(|retrieval| retrieval.filename.as_deref() == Some(filename))
}

fn manifest_from_response(
    response: ClinicalTrialsGovApiV2Response,
) -> Result<TrialDocumentsManifest, BioMcpError> {
    let nct_id = response.capture().provider_record_identity().to_owned();
    let result = response
        .into_projection_with_artifact_retrievals()
        .map_err(|_| BioMcpError::InternalProcessing)?;
    let state = super::section_state(result.artifact_state());
    let artifact_retrievals: &[ClinicalTrialsGovArtifactRetrieval] = result.artifact_retrievals();
    let retrievals = artifact_retrievals
        .iter()
        .filter(|retrieval| {
            result
                .projection()
                .trial()
                .artifacts()
                .unwrap_or_default()
                .iter()
                .any(|document| retrieval.id() == document.id())
        })
        .map(|retrieval| {
            let filename = retrieval.advertised_file_name().map(str::to_owned);
            let handle = filename
                .as_deref()
                .filter(|value| validate_filename(value).is_ok())
                .map(|value| document_command(&nct_id, value));
            (retrieval.id(), TrialDocumentRetrieval { filename, handle })
        })
        .collect();
    Ok(TrialDocumentsManifest {
        projection: result.into_projection(),
        state,
        retrievals,
    })
}

fn document_view<'a>(
    document: &'a ClinicalTrialArtifactDescriptor,
    retrieval: Option<&'a TrialDocumentRetrieval>,
) -> TrialDocumentView<'a> {
    TrialDocumentView {
        document_type: document.document_type().map(biodata::ExtensibleCode::code),
        label: document.label(),
        date: document.document_date(),
        upload_date: document.upload_date(),
        filename: retrieval.and_then(|value| value.filename.as_deref()),
        size_bytes: document.size_bytes(),
        has_protocol: document.has_protocol(),
        has_sap: document.has_statistical_analysis_plan(),
        has_icf: document.has_informed_consent_form(),
        handle: retrieval.and_then(|value| value.handle.as_deref()),
    }
}

impl TrialDocumentsManifest {
    pub(crate) fn nct_id(&self) -> &str {
        self.projection.capture().provider_record_identity()
    }

    fn documents(&self) -> Vec<TrialDocumentView<'_>> {
        self.projection
            .trial()
            .artifacts()
            .unwrap_or_default()
            .iter()
            .map(|document| document_view(document, self.retrievals.get(&document.id())))
            .collect()
    }

    pub(crate) fn document_handles(&self) -> impl Iterator<Item = &str> {
        self.retrievals
            .values()
            .filter_map(|value| value.handle.as_deref())
    }
}

impl serde::Serialize for TrialDocumentsManifest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(serde::Serialize)]
        struct View<'a> {
            nct_id: &'a str,
            source: &'static str,
            section_state: super::TrialSectionState,
            documents: Vec<TrialDocumentView<'a>>,
        }
        View {
            nct_id: self.nct_id(),
            source: "ClinicalTrials.gov",
            section_state: self.state,
            documents: self.documents(),
        }
        .serialize(serializer)
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

fn download_url(base: &reqwest::Url, nct_id: &str, filename: &str) -> reqwest::Url {
    let mut url = base.clone();
    url.path_segments_mut()
        .expect("approved HTTP(S) origin supports path segments")
        .extend(["large-docs", &nct_id[9..], nct_id, filename]);
    url
}

async fn download_document_from_base(
    base: reqwest::Url,
    nct_id: &str,
    filename: &str,
) -> Result<Vec<u8>, BioMcpError> {
    let policy =
        ProviderUrlPolicy::for_consumer(ProviderUrlConsumer::ClinicalTrialsDocument, Some(&base))?;
    download_document_with_policy(base, nct_id, filename, policy).await
}

async fn download_document_with_policy(
    base: reqwest::Url,
    nct_id: &str,
    filename: &str,
    policy: ProviderUrlPolicy,
) -> Result<Vec<u8>, BioMcpError> {
    let url = download_url(&base, nct_id, filename);
    policy.validate_url(&url)?;
    let client = reqwest::Client::builder()
        .connect_timeout(DOCUMENT_CONNECT_TIMEOUT)
        .timeout(DOCUMENT_REQUEST_TIMEOUT)
        .gzip(false)
        .no_proxy()
        .dns_resolver(policy.dns_resolver())
        .redirect(policy.redirect_policy())
        .build()?;
    let response = client
        .get(url)
        .send_with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::CLINICAL_TRIALS,
        ))
        .await?;
    if !response.status().is_success() {
        return Err(BioMcpError::Api {
            api: CTGOV_CDN_API.into(),
            message: format!("HTTP {} retrieving trial document", response.status()),
        }
        .with_source_context(crate::error::SourceContext::retry(
            crate::error::SourceProvider::CLINICAL_TRIALS,
        )));
    }
    read_document_body(response).await
}

async fn read_document_body(mut response: reqwest::Response) -> Result<Vec<u8>, BioMcpError> {
    let provider = crate::error::SourceProvider::CLINICAL_TRIALS;
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(BioMcpError::from)
        .map_err(|error| error.with_source_context(crate::error::SourceContext::retry(provider)))?
    {
        let remaining_with_overflow = DOCUMENT_MAX_BYTES
            .saturating_sub(body.len())
            .saturating_add(1);
        body.extend_from_slice(&chunk[..chunk.len().min(remaining_with_overflow)]);
        if body.len() > DOCUMENT_MAX_BYTES {
            return Err(BioMcpError::Api {
                api: CTGOV_CDN_API.into(),
                message: format!("Response body exceeded {DOCUMENT_MAX_BYTES} bytes"),
            }
            .with_source_context(crate::error::SourceContext::narrow(provider)));
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

    async fn download_fixture(
        base: reqwest::Url,
        nct_id: &str,
        filename: &str,
    ) -> Result<Vec<u8>, BioMcpError> {
        let policy =
            ProviderUrlPolicy::test_fixture(ProviderUrlConsumer::ClinicalTrialsDocument, &base)?;
        download_document_with_policy(base, nct_id, filename, policy).await
    }

    fn response_with_documents(documents: serde_json::Value) -> ClinicalTrialsGovApiV2Response {
        let body = serde_json::to_vec(&serde_json::json!({
            "protocolSection": {
                "identificationModule": {
                    "nctId": "NCT03361748",
                    "briefTitle": "Synthetic validation study"
                },
                "statusModule": {"overallStatus": "RECRUITING"},
                "sponsorCollaboratorsModule": {
                    "leadSponsor": {"name": "Example sponsor"}
                },
                "conditionsModule": {"conditions": ["Example condition"]},
                "designModule": {"studyType": "INTERVENTIONAL"}
            },
            "documentSection": {"largeDocumentModule": {"largeDocs": documents}}
        }))
        .unwrap();
        ClinicalTrialsClient::decode_biodata_detail_response(
            "NCT03361748",
            &["documents".to_string()],
            reqwest::StatusCode::OK,
            &body,
        )
        .unwrap()
    }

    #[test]
    fn maps_manifest_metadata_handles_and_empty_state() {
        let response = response_with_documents(serde_json::json!([{
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
        let manifest = manifest_from_response(response).unwrap();
        let documents = manifest.documents();
        assert_eq!(documents.len(), 2);
        assert_eq!(documents[0].size_bytes, Some(50));
        assert_eq!(
            documents[0].handle,
            Some("biomcp get trial NCT03361748 document \"Protocol final.pdf\"")
        );
        assert!(is_advertised(&manifest, "Protocol final.pdf"));
        assert!(!is_advertised(&manifest, "protocol final.pdf"));
        assert!(!is_advertised(&manifest, "Unknown.pdf"));
        assert_eq!(documents[1].size_bytes, Some(33_554_433));
        let diagnostic = format!("{manifest:?}");
        for sentinel in ["Protocol and SAP", "2019-07-18", "Protocol final.pdf"] {
            assert!(!diagnostic.contains(sentinel));
        }
        assert!(
            manifest_from_response(response_with_documents(serde_json::json!([])))
                .unwrap()
                .documents()
                .is_empty()
        );
    }

    #[test]
    fn document_output_preserves_every_request_state_and_explicit_empty() {
        let unrequested = manifest_from_response(
            ClinicalTrialsClient::decode_biodata_detail_response(
                "NCT03361748",
                &[],
                reqwest::StatusCode::OK,
                &serde_json::to_vec(&serde_json::json!({
                    "protocolSection": {
                        "identificationModule": {"nctId": "NCT03361748", "briefTitle": "T"},
                        "statusModule": {"overallStatus": "RECRUITING"},
                        "sponsorCollaboratorsModule": {"leadSponsor": {"name": "S"}},
                        "conditionsModule": {"conditions": ["C"]},
                        "designModule": {"studyType": "INTERVENTIONAL"}
                    }
                }))
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let absent =
            manifest_from_response(response_with_documents(serde_json::Value::Null)).unwrap();
        let empty = manifest_from_response(response_with_documents(serde_json::json!([]))).unwrap();
        let populated = manifest_from_response(response_with_documents(serde_json::json!([{
            "filename": "P.pdf"
        }])))
        .unwrap();
        let mut unavailable = unrequested.clone();
        unavailable.state = crate::entities::trial::TrialSectionState::Unavailable;
        for (manifest, state, count) in [
            (&unrequested, "not_requested", 0),
            (&unavailable, "unavailable", 0),
            (&absent, "absent", 0),
            (&empty, "present", 0),
            (&populated, "present", 1),
        ] {
            let value = serde_json::to_value(manifest).unwrap();
            assert_eq!(value["section_state"], state);
            assert_eq!(value["documents"].as_array().unwrap().len(), count);
        }
    }

    #[test]
    fn eligibility_provenance_tracks_document_availability() {
        let response = response_with_documents(serde_json::json!([{"filename": "Protocol.pdf"}]));
        let available = eligibility_provenance(&response);
        assert!(available.posted_documents_available);
        assert_eq!(available.source_authority, "clinicaltrials.gov");
        assert_eq!(available.provider_record_identity, "NCT03361748");
        assert_eq!(available.capture_digest, response.capture().digest());
        let diagnostic = format!("{available:?}");
        for sentinel in ["clinicaltrials.gov", "NCT03361748", "sha256:"] {
            assert!(!diagnostic.contains(sentinel));
        }
        assert_eq!(
            available.documents_handle.as_deref(),
            Some("biomcp --json get trial NCT03361748 documents")
        );

        let response = response_with_documents(serde_json::json!([]));
        let unavailable = eligibility_provenance(&response);
        assert!(!unavailable.posted_documents_available);
        assert!(unavailable.documents_handle.is_none());

        let response = response_with_documents(serde_json::Value::Null);
        let absent = eligibility_provenance(&response);
        assert!(!absent.posted_documents_available);
        assert!(absent.documents_handle.is_none());
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

    #[tokio::test]
    async fn unadvertised_filename_is_absent_from_error_diagnostics() {
        const SENTINEL: &str = "UNADVERTISED-FILENAME-SENTINEL-0116.pdf";
        let response = response_with_documents(serde_json::json!([]));
        let manifest = manifest_from_response(response).unwrap();
        let error = document_bytes_from_manifest(
            &manifest,
            SENTINEL,
            reqwest::Url::parse("https://cdn.clinicaltrials.gov").unwrap(),
        )
        .await
        .unwrap_err();
        assert!(!error.to_string().contains(SENTINEL));
        assert!(!format!("{error:?}").contains(SENTINEL));
    }

    #[test]
    fn omits_handles_for_unsafe_advertised_filenames() {
        let response =
            response_with_documents(serde_json::json!([{"filename": "../Protocol.pdf"}]));
        let manifest = manifest_from_response(response).unwrap();
        let documents = manifest.documents();
        assert_eq!(documents[0].filename, Some("../Protocol.pdf"));
        assert!(documents[0].handle.is_none());
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
    async fn reported_oversize_remains_listable_and_actual_small_body_is_retrievable() {
        let response = response_with_documents(serde_json::json!([{
            "filename": "reported-oversize.pdf",
            "size": 33554433
        }]));
        let manifest = manifest_from_response(response).unwrap();
        assert_eq!(manifest.documents()[0].size_bytes, Some(33_554_433));

        let router = Router::new().route(
            "/large-docs/48/NCT03361748/reported-oversize.pdf",
            get(|| async { b"small actual body".to_vec() }),
        );
        let (base, task) = serve(router).await;
        assert!(is_advertised(&manifest, "reported-oversize.pdf"));
        let body = download_fixture(base, manifest.nct_id(), "reported-oversize.pdf")
            .await
            .unwrap();
        assert_eq!(body, b"small actual body");
        task.abort();
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

        let exact = download_fixture(base.clone(), "NCT03361748", "exact.pdf")
            .await
            .unwrap();
        assert_eq!(exact.len(), DOCUMENT_MAX_BYTES);
        let error = download_fixture(base, "NCT03361748", "too-large.pdf")
            .await
            .unwrap_err();
        assert!(format!("{error:?}").contains("exceeded 33554432 bytes"));
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

        let error = download_fixture(approved_base, "NCT03361748", "Protocol.pdf")
            .await
            .unwrap_err();
        assert!(format!("{error:?}").contains("redirect"));
        assert_eq!(target_requests.load(Ordering::SeqCst), 0);
        redirect_task.abort();
        target_task.abort();
    }
}
