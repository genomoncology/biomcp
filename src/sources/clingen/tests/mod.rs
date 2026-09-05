mod parsing;

use super::*;
use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{Response, StatusCode, Uri};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::Barrier;

const LOOKUP_PATH: &str = "/api/genes/look/TP53";
const VALIDITY_PATH: &str = "/kb/gene-validity/download";
const DOSAGE_PATH: &str = "/kb/gene-dosage/download";

#[derive(Clone)]
struct ResponseSpec {
    status: StatusCode,
    content_type: &'static str,
    body: Vec<u8>,
    delay: Duration,
}

impl ResponseSpec {
    fn ok_json(body: &[u8]) -> Self {
        Self {
            status: StatusCode::OK,
            content_type: "application/json",
            body: body.to_vec(),
            delay: Duration::ZERO,
        }
    }

    fn ok_csv(body: &[u8]) -> Self {
        Self {
            status: StatusCode::OK,
            content_type: "text/csv",
            body: body.to_vec(),
            delay: Duration::ZERO,
        }
    }

    fn delayed(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    fn failed(body: &[u8]) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            content_type: "text/plain",
            body: body.to_vec(),
            delay: Duration::ZERO,
        }
    }
}

#[derive(Clone)]
struct FixtureState {
    responses: Arc<HashMap<String, ResponseSpec>>,
    requests: Arc<Mutex<Vec<(String, Instant)>>>,
    barrier: Option<Arc<Barrier>>,
}

struct Fixture {
    base: String,
    requests: Arc<Mutex<Vec<(String, Instant)>>>,
    server: tokio::task::JoinHandle<()>,
}

impl Fixture {
    async fn start(responses: HashMap<String, ResponseSpec>, synchronize_starts: bool) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind ClinGen fixture");
        let address = listener.local_addr().expect("ClinGen fixture address");
        let requests = Arc::new(Mutex::new(Vec::new()));
        let state = FixtureState {
            responses: Arc::new(responses),
            requests: Arc::clone(&requests),
            barrier: synchronize_starts.then(|| Arc::new(Barrier::new(3))),
        };
        let app = Router::new().fallback(handle).with_state(state);
        let server = tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("serve ClinGen fixture");
        });
        Self {
            base: format!("http://{address}"),
            requests,
            server,
        }
    }

    fn client(&self) -> ClinGenClient {
        ClinGenClient::with_client_and_base(
            crate::sources::test_client().expect("test client"),
            self.base.clone(),
        )
    }

    fn paths(&self) -> Vec<String> {
        self.requests
            .lock()
            .expect("request log")
            .iter()
            .map(|(path, _)| path.clone())
            .collect()
    }

    fn starts(&self) -> Vec<Instant> {
        self.requests
            .lock()
            .expect("request log")
            .iter()
            .map(|(_, started)| *started)
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn handle(State(state): State<FixtureState>, uri: Uri) -> Response<Body> {
    let path = uri.path().to_string();
    state
        .requests
        .lock()
        .expect("request log")
        .push((path.clone(), Instant::now()));
    let Some(spec) = state.responses.get(&path).cloned() else {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("unexpected route"))
            .unwrap();
    };
    if let Some(barrier) = state.barrier {
        barrier.wait().await;
    }
    tokio::time::sleep(spec.delay).await;
    Response::builder()
        .status(spec.status)
        .header("content-type", spec.content_type)
        .body(Body::from(spec.body))
        .unwrap()
}

fn captured_responses() -> HashMap<String, ResponseSpec> {
    HashMap::from([
        (
            LOOKUP_PATH.to_string(),
            ResponseSpec::ok_json(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/clingen/lookup_tp53.json"
            ))),
        ),
        (
            VALIDITY_PATH.to_string(),
            ResponseSpec::ok_csv(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/clingen/validity_tp53.csv"
            ))),
        ),
        (
            DOSAGE_PATH.to_string(),
            ResponseSpec::ok_csv(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/testdata/sources/clingen/dosage_tp53.csv"
            ))),
        ),
    ])
}

fn assert_exact_routes(paths: &[String]) {
    assert_eq!(paths.len(), 3, "one shared lookup and two downloads");
    for expected in [LOOKUP_PATH, VALIDITY_PATH, DOSAGE_PATH] {
        assert_eq!(
            paths
                .iter()
                .filter(|path| path.as_str() == expected)
                .count(),
            1
        );
    }
}

#[tokio::test]
async fn operations_start_concurrently_and_share_one_lookup() {
    let fixture = Fixture::start(captured_responses(), true).await;
    let started = Instant::now();
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(500))
        .await
        .expect("ClinGen context");

    assert_eq!(context.validity_status.status, ClinGenFamilyState::Data);
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Data);
    assert_exact_routes(&fixture.paths());
    let starts = fixture.starts();
    let first = starts.iter().min().unwrap();
    let last = starts.iter().max().unwrap();
    assert!(last.duration_since(*first) < Duration::from_millis(100));
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[tokio::test]
async fn dosage_timeout_preserves_completed_validity() {
    let mut responses = captured_responses();
    responses.insert(
        DOSAGE_PATH.to_string(),
        responses[DOSAGE_PATH]
            .clone()
            .delayed(Duration::from_millis(200)),
    );
    let fixture = Fixture::start(responses, false).await;
    let started = Instant::now();
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(40))
        .await
        .expect("partial ClinGen context");

    assert_eq!(context.validity.len(), 1);
    assert_eq!(context.validity_status.status, ClinGenFamilyState::Data);
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::TimedOut);
    assert_eq!(
        context.dosage_status.message.as_deref(),
        Some(DOSAGE_TIMEOUT_MESSAGE)
    );
    assert!(started.elapsed() < Duration::from_millis(150));
    assert_exact_routes(&fixture.paths());
}

#[tokio::test]
async fn validity_failure_preserves_newest_dosage_row_and_public_message() {
    let mut responses = captured_responses();
    responses.insert(
        VALIDITY_PATH.to_string(),
        ResponseSpec::failed(b"secret upstream parser detail https://internal.invalid"),
    );
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(200))
        .await
        .expect("partial ClinGen context");

    assert!(context.validity.is_empty());
    assert_eq!(context.validity_status.status, ClinGenFamilyState::Failed);
    assert_eq!(
        context.validity_status.message.as_deref(),
        Some(VALIDITY_FAILED_MESSAGE)
    );
    assert_eq!(
        context.haploinsufficiency.as_deref(),
        Some("Sufficient Evidence for Haploinsufficiency")
    );
    assert_eq!(
        context.triplosensitivity.as_deref(),
        Some("No Evidence for Triplosensitivity")
    );
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Data);
}

#[tokio::test]
async fn failed_lookup_preserves_symbol_data_but_does_not_confirm_zero_match() {
    let mut responses = captured_responses();
    responses.insert(
        LOOKUP_PATH.to_string(),
        ResponseSpec::failed(b"lookup private failure"),
    );
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(200))
        .await
        .expect("symbol fallback context");

    assert_eq!(context.validity_status.status, ClinGenFamilyState::Data);
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Data);
    assert_exact_routes(&fixture.paths());

    let responses = HashMap::from([
        (
            "/api/genes/look/NRAS".to_string(),
            ResponseSpec::failed(b"lookup private failure"),
        ),
        (
            VALIDITY_PATH.to_string(),
            captured_responses()[VALIDITY_PATH].clone(),
        ),
        (
            DOSAGE_PATH.to_string(),
            captured_responses()[DOSAGE_PATH].clone(),
        ),
    ]);
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("NRAS", Duration::from_millis(200))
        .await
        .expect("inconclusive context");
    assert_eq!(context.validity_status.status, ClinGenFamilyState::Failed);
    assert_eq!(context.validity_status.op, ClinGenOperation::GeneLookup);
    assert_eq!(
        context.validity_status.message.as_deref(),
        Some(VALIDITY_LOOKUP_FAILED_MESSAGE)
    );
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Failed);
    assert_eq!(context.dosage_status.op, ClinGenOperation::GeneLookup);
    assert_eq!(
        context.dosage_status.message.as_deref(),
        Some(DOSAGE_LOOKUP_FAILED_MESSAGE)
    );
}

#[tokio::test]
async fn timed_out_lookup_preserves_symbol_data_but_marks_zero_match_timed_out() {
    let mut responses = captured_responses();
    responses.insert(
        LOOKUP_PATH.to_string(),
        responses[LOOKUP_PATH]
            .clone()
            .delayed(Duration::from_millis(200)),
    );
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(40))
        .await
        .expect("symbol fallback context");
    assert_eq!(context.validity_status.status, ClinGenFamilyState::Data);
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Data);

    let responses = HashMap::from([
        (
            "/api/genes/look/NRAS".to_string(),
            ResponseSpec::ok_json(b"[]").delayed(Duration::from_millis(200)),
        ),
        (
            VALIDITY_PATH.to_string(),
            captured_responses()[VALIDITY_PATH].clone(),
        ),
        (
            DOSAGE_PATH.to_string(),
            captured_responses()[DOSAGE_PATH].clone(),
        ),
    ]);
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("NRAS", Duration::from_millis(40))
        .await
        .expect("inconclusive context");
    assert_eq!(context.validity_status.status, ClinGenFamilyState::TimedOut);
    assert_eq!(context.validity_status.op, ClinGenOperation::GeneLookup);
    assert_eq!(
        context.validity_status.message.as_deref(),
        Some(VALIDITY_LOOKUP_TIMEOUT_MESSAGE)
    );
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::TimedOut);
    assert_eq!(context.dosage_status.op, ClinGenOperation::GeneLookup);
    assert_eq!(
        context.dosage_status.message.as_deref(),
        Some(DOSAGE_LOOKUP_TIMEOUT_MESSAGE)
    );
}

#[tokio::test]
async fn valid_zero_match_is_empty_and_oversized_download_is_failed() {
    let mut responses = captured_responses();
    responses.insert(
        DOSAGE_PATH.to_string(),
        ResponseSpec::ok_csv(&vec![b'x'; crate::sources::DEFAULT_MAX_BODY_BYTES + 1]),
    );
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(500))
        .await
        .expect("bounded ClinGen context");
    assert_eq!(context.validity_status.status, ClinGenFamilyState::Data);
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Failed);
    assert_eq!(
        context.dosage_status.message.as_deref(),
        Some(DOSAGE_FAILED_MESSAGE)
    );

    let empty_validity =
        b"GENE SYMBOL,GENE ID (HGNC),DISEASE LABEL,CLASSIFICATION,CLASSIFICATION DATE,MOI\n";
    let empty_dosage = b"GENE SYMBOL,HGNC ID,HAPLOINSUFFICIENCY,TRIPLOSENSITIVITY,DATE\n";
    let responses = HashMap::from([
        (
            LOOKUP_PATH.to_string(),
            captured_responses()[LOOKUP_PATH].clone(),
        ),
        (
            VALIDITY_PATH.to_string(),
            ResponseSpec::ok_csv(empty_validity),
        ),
        (DOSAGE_PATH.to_string(), ResponseSpec::ok_csv(empty_dosage)),
    ]);
    let fixture = Fixture::start(responses, false).await;
    let context = fixture
        .client()
        .gene_context("TP53", Duration::from_millis(200))
        .await
        .expect("healthy empty ClinGen context");
    assert_eq!(context.validity_status.status, ClinGenFamilyState::Empty);
    assert_eq!(context.dosage_status.status, ClinGenFamilyState::Empty);
}
