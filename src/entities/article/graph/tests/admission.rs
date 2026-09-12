//! Ticket 1145 admission proof: the process-wide JATS worker permit under the
//! absolute twenty-two-second command deadline. The seam blocks on a Unix
//! socket read held by the test; a ready file proves the permit was acquired
//! before blocking. No tokio clock manipulation is involved — the commands
//! run against a shrunk real deadline (three seconds via the test-only
//! command-budget seam), so the whole proof settles in a few seconds while
//! exercising the same absolute-deadline admission and settlement path as
//! the frozen twenty-two-second production value.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::OnceLock;

use super::super::super::test_support::*;
use super::citation_evidence::citation_evidence;
use super::citation_evidence::install_jats_citation_seam;
use crate::sources::semantic_scholar::SemanticScholarClient;
use crate::transform::article::{JatsCitationExtraction, JatsCitationTargetIds};

const CITING_PID: &str = "11223344556677889900aabbccddeeff00112233";
const CITED_PID: &str = "4433221100ffeeddccbbaa998877665544332211";

static ADMISSION_READY_PATH: OnceLock<PathBuf> = OnceLock::new();
static ADMISSION_RELEASE: Mutex<Option<UnixStream>> = Mutex::new(None);
static ADMISSION_COUNT: Mutex<u32> = Mutex::new(0);

/// The test-only parse seam: records admission, then blocks until the test
/// releases the socket. It performs no request, no file mutation beyond the
/// ready log, and touches no shared result state.
fn admission_parse(
    _xml: &str,
    _target: &JatsCitationTargetIds,
) -> Result<JatsCitationExtraction, ()> {
    {
        let mut count = ADMISSION_COUNT.lock().unwrap();
        *count += 1;
    }
    if let Some(path) = ADMISSION_READY_PATH.get() {
        use std::io::Write as _;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("admission ready file");
        writeln!(file, "admitted").expect("admission ready append");
    }
    let mut blocker = ADMISSION_RELEASE.lock().unwrap();
    if let Some(socket) = blocker.as_mut() {
        let mut buffer = [0u8; 1];
        // Blocks the pure parsing worker until the test writes the release
        // byte. Dropping or closing the socket also unblocks with an error,
        // which still settles the worker.
        let _ = socket.read(&mut buffer);
    }
    Ok(JatsCitationExtraction::Linked {
        ref_id: "bib7".into(),
        passages: Vec::new(),
    })
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn citation_evidence_deadline_bounds_late_jats_workers_under_one_permit() {
    let mut env = TestEnv::new();
    let cache = crate::test_support::TempDirGuard::new("citation-admission");
    env.set("BIOMCP_CACHE_DIR", cache.path());
    env.set("BIOMCP_TEST_CITATION_COMMAND_DEADLINE_MS", "3000");

    let jats_body = "<article><front><article-meta><article-title>T</article-title>\
</article-meta></front><body><sec><title>Results</title><p>Anchor \
<xref ref-type=\"bibr\" rid=\"bib7\">7</xref> text.</p></sec></body>\
<back><ref-list><ref id=\"bib7\"><element-citation>\
<pub-id pub-id-type=\"doi\">10.1/target</pub-id>\
</element-citation></ref></ref-list></back></article>";
    let requests = std::sync::Arc::new(Mutex::new(Vec::<String>::new()));
    let logged = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        let target = request
            .split_whitespace()
            .nth(1)
            .unwrap_or_default()
            .to_string();
        logged.lock().unwrap().push(target.clone());
        let reply = if request.starts_with("POST") {
            let (paper_id, ext) = if request.contains(CITING_PID) {
                (
                    CITING_PID,
                    "\"externalIds\":{\"PubMed\":\"39991290\",\"PubMedCentral\":\"PMC12923956\"}",
                )
            } else {
                (CITED_PID, "\"externalIds\":{\"DOI\":\"10.1/target\"}")
            };
            format!("[{{\"paperId\":\"{paper_id}\",\"title\":\"T\",{ext}}}]")
        } else if target.contains("/fullTextXML") {
            jats_body.to_string()
        } else if target.contains("/references") {
            format!(
                "{{\"offset\":0,\"next\":null,\"data\":[{{\"contexts\":[],\
\"intents\":[\"background\"],\"isInfluential\":false,\
\"citedPaper\":{{\"paperId\":\"{CITED_PID}\",\"title\":\"T\",\
\"externalIds\":{{\"DOI\":\"10.1/target\"}}}}}}]}}"
            )
        } else {
            "{\"records\":[]}".to_string()
        };
        TestHttpReply::Bytes(test_http_response(
            "200 OK",
            "application/xml",
            reply.as_bytes(),
        ))
    })
    .await;
    env.set("BIOMCP_EUROPEPMC_BASE", &fixture.base);
    env.set("BIOMCP_NCBI_IDCONV_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);

    let (mut release_socket, blocking_socket) = UnixStream::pair().expect("admission socket pair");
    *ADMISSION_RELEASE.lock().unwrap() = Some(blocking_socket);
    let ready_path = std::path::Path::new(cache.path()).join("admissions.log");
    ADMISSION_READY_PATH
        .set(ready_path.clone())
        .expect("admission path once");
    install_jats_citation_seam(Some(admission_parse));

    let first_client =
        SemanticScholarClient::new_with_cache_observers(&fixture.base, |_, _| {}, |_, _| {})
            .unwrap();
    let second_client =
        SemanticScholarClient::new_with_cache_observers(&fixture.base, |_, _| {}, |_, _| {})
            .unwrap();
    // Both commands force the JATS path. The first admits the only worker
    // permit; the second can admit none while it is held. Each reaches its
    // own absolute deadline.
    let first = tokio::spawn(crate::sources::semantic_scholar::with_test_client(
        first_client,
        citation_evidence(CITING_PID, CITED_PID, true),
    ));
    let second = tokio::spawn(crate::sources::semantic_scholar::with_test_client(
        second_client,
        citation_evidence(CITING_PID, CITED_PID, true),
    ));

    // Wait for exactly the first admission with generous real polling.
    let mut admitted_lines = 0;
    for _ in 0..600 {
        admitted_lines = std::fs::read_to_string(&ready_path)
            .map(|text| text.lines().filter(|line| !line.trim().is_empty()).count())
            .unwrap_or(0);
        if admitted_lines >= 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    assert_eq!(admitted_lines, 1, "the permit admits exactly one worker");

    let first_outcome = first.await.expect("first task joins");
    let second_outcome = second.await.expect("second task joins");
    let first_error = first_outcome.expect_err("first deadline");
    let second_error = second_outcome.expect_err("second deadline");
    for error in [&first_error, &second_error] {
        match error {
            crate::error::BioMcpError::Api { api, message } => {
                assert_eq!(api, "article-citation-evidence");
                assert_eq!(message, "invocation deadline exceeded");
            }
            other => panic!("bounded deadline error, got: {other:?}"),
        }
    }
    assert_eq!(
        std::fs::read_to_string(&ready_path)
            .expect("admission log")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count(),
        1,
        "the second command admitted no worker while the permit was held"
    );

    // Release the late pure worker: it settles, releases the permit exactly
    // once, and mutates nothing else.
    release_socket.write_all(&[1u8]).expect("release byte");
    drop(release_socket);
    for _ in 0..100 {
        if *ADMISSION_COUNT.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    install_jats_citation_seam(None);

    // The permit is free again: a fresh command admits a worker and completes
    // with the real parser over the same fixture.
    let settle_client =
        SemanticScholarClient::new_with_cache_observers(&fixture.base, |_, _| {}, |_, _| {})
            .unwrap();
    let settled = crate::sources::semantic_scholar::with_test_client(
        settle_client,
        citation_evidence(CITING_PID, CITED_PID, true),
    )
    .await
    .expect("permit released");
    assert_eq!(
        settled.status,
        super::super::citation_evidence::CitationEvidenceStatus::ContextFromFulltext
    );
    assert_eq!(settled.passages.len(), 1);
    assert_eq!(
        *ADMISSION_COUNT.lock().unwrap(),
        1,
        "the late worker never re-ran"
    );
}
