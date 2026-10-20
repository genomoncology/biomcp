use super::ArticleFixture;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

struct HangingZip {
    base: String,
    requested: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    primary_failure_nanos: Arc<AtomicU64>,
    worker: Option<thread::JoinHandle<()>>,
}

impl HangingZip {
    fn start(hang: bool) -> Self {
        Self::with_primary(hang, false)
    }

    fn with_primary(hang: bool, primary_failure: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requested = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let primary_failure_nanos = Arc::new(AtomicU64::new(0));
        let failure_nanos = primary_failure_nanos.clone();
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        archive
            .start_file("traces-s1.csv", zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"time,value\n0,1\n").unwrap();
        let zip_bytes = archive.finish().unwrap().into_inner();
        let zip_requested = requested.clone();
        let stopping = stop.clone();
        let worker = thread::spawn(move || {
            let mut held = Vec::new();
            let mut failures = Vec::new();
            while !stopping.load(Ordering::SeqCst) {
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut bytes = [0; 8192];
                    let count = stream.read(&mut bytes).unwrap();
                    let request = String::from_utf8_lossy(&bytes[..count]);
                    if primary_failure && request.starts_with("GET /?list-type=2") {
                        let failure_nanos = failure_nanos.clone();
                        failures.push(thread::spawn(move || {
                            let started = std::time::Instant::now();
                            thread::sleep(Duration::from_millis(200));
                            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\nContent-Length: 9\r\nConnection: close\r\n\r\n<records>");
                            failure_nanos.store(started.elapsed().as_nanos() as u64, Ordering::SeqCst);
                        }));
                        continue;
                    }
                    if request.contains("/supplementaryFiles") {
                        zip_requested.store(true, Ordering::SeqCst);
                        if hang {
                            held.push(stream);
                            continue;
                        }
                    }
                    let (status, content_type, body): (&str, &str, &[u8]) = if primary_failure
                        && request.contains("/supplementaryFiles")
                    {
                        ("200 OK", "application/zip", &zip_bytes)
                    } else if request.starts_with("GET /?list-type=2") {
                        ("200 OK", "application/xml", b"<ListBucketResult><CommonPrefixes><Prefix>PMC123456.1/</Prefix></CommonPrefixes></ListBucketResult>")
                    } else if request.starts_with("GET /PMC123456.1/PMC123456.1.json ") {
                        ("200 OK", "application/json", br#"{"pmcid":"PMC123456","version":1,"license_code":"CC BY","xml_url":"s3://pmc-oa-opendata/PMC123456.1/article.nxml","media_urls":["s3://pmc-oa-opendata/PMC123456.1/traces-s1.csv"]}"#)
                    } else if request.starts_with("GET /PMC123456.1/article.nxml ") {
                        ("200 OK", "application/xml", br#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><supplementary-material xlink:href="traces-s1.csv"><label>Data S1</label></supplementary-material></body></article>"#)
                    } else if request.starts_with("GET /PMC123456.1/traces-s1.csv ") {
                        ("200 OK", "text/csv", b"time,value\n0,1\n")
                    } else if !primary_failure && request.starts_with("GET /PMC123456/fullTextXML ")
                    {
                        (
                            "503 Service Unavailable",
                            "text/plain",
                            b"optional XML outage",
                        )
                    } else {
                        ("404 Not Found", "text/plain", b"not found")
                    };
                    let header = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = stream.write_all(header.as_bytes());
                    let _ = stream.write_all(body);
                } else {
                    thread::sleep(Duration::from_millis(2));
                }
            }
            for failure in failures {
                failure.join().unwrap();
            }
        });
        Self {
            base,
            requested,
            stop,
            primary_failure_nanos,
            worker: Some(worker),
        }
    }
}

impl Drop for HangingZip {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        self.worker.take().unwrap().join().unwrap();
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn complete_primary_assets_do_not_wait_for_an_optional_zip() {
    let fixture = ArticleFixture::start();
    let zip = HangingZip::start(true);
    configure_archive(&fixture, &zip);
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663011", "assets"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert!(
        !zip.requested.load(Ordering::SeqCst),
        "a complete primary archive must not request the hanging secondary ZIP: {output:?}"
    );
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut names: Vec<_> = manifest["assets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|asset| asset["filename"].as_str().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["traces-s1.csv"]);
    assert!(
        manifest["source_attempts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|attempt| {
                attempt["provider"]["label"] == "Europe PMC XML"
                    && attempt["outcome"] == "source_unavailable"
            }),
        "optional XML outage must remain visible: {manifest}"
    );
    assert!(
        !manifest["source_attempts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|attempt| { attempt["source_document"] == "europe_pmc_zip" }),
        "an unconsulted ZIP must not be labeled absent or unavailable: {manifest}"
    );
    let output = fixture
        .biomcp_command(&["get", "article", "22663011", "asset", "traces-s1.csv"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, b"time,value\n0,1\n");
    assert!(!zip.requested.load(Ordering::SeqCst));
}

fn configure_archive(fixture: &ArticleFixture, zip: &HangingZip) {
    use std::fs::OpenOptions;
    let mut env = OpenOptions::new()
        .append(true)
        .open(&fixture.env_file)
        .unwrap();
    for key in [
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_PMC_OA_BASE",
        "BIOMCP_PMC_HTML_BASE",
        "BIOMCP_PUBMED_BASE",
    ] {
        writeln!(env, "export {key}='{}'", zip.base).unwrap();
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn missing_primary_asset_still_consults_the_secondary_zip() {
    let fixture = ArticleFixture::start();
    let zip = HangingZip::start(false);
    configure_archive(&fixture, &zip);
    let output = fixture
        .biomcp_command(&["get", "article", "22663011", "asset", "missing.csv"])
        .output()
        .unwrap();
    assert!(!output.status.success(), "{output:?}");
    assert!(
        zip.requested.load(Ordering::SeqCst),
        "a requested asset missing from the primary must still try the ZIP fallback"
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn failed_primary_still_returns_the_zip_assets() {
    let fixture = ArticleFixture::start();
    let zip = HangingZip::with_primary(false, true);
    configure_archive(&fixture, &zip);
    let started = std::time::Instant::now();
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663011", "assets"])
        .output()
        .unwrap();
    eprintln!(
        "FALLBACK_CLI_SECONDS={:.9} PRIMARY_FAILURE_SECONDS={:.9}",
        started.elapsed().as_secs_f64(),
        zip.primary_failure_nanos.load(Ordering::SeqCst) as f64 / 1_000_000_000.0
    );
    assert!(output.status.success(), "{output:?}");
    assert!(zip.requested.load(Ordering::SeqCst));
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        manifest["assets"].as_array().unwrap().iter().any(|asset| {
            asset["filename"] == "traces-s1.csv" && asset["provider"]["source"] == "Europe PMC"
        }),
        "the failed primary must retain the ZIP fallback: {manifest}"
    );
}
