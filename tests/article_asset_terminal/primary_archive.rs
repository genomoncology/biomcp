use super::ArticleFixture;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

struct HangingZip {
    base: String,
    requested: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    worker: Option<thread::JoinHandle<()>>,
}

impl HangingZip {
    fn start(hang: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requested = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let zip_requested = requested.clone();
        let stopping = stop.clone();
        let worker = thread::spawn(move || {
            let mut held = Vec::new();
            while !stopping.load(Ordering::SeqCst) {
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let mut bytes = [0; 8192];
                    let count = stream.read(&mut bytes).unwrap();
                    let request = String::from_utf8_lossy(&bytes[..count]);
                    if request.contains("/supplementaryFiles") {
                        zip_requested.store(true, Ordering::SeqCst);
                        if hang {
                            held.push(stream);
                            continue;
                        }
                    }
                    let (status, content_type, body): (&str, &str, &[u8]) = if request
                        .starts_with("GET /?list-type=2")
                    {
                        ("200 OK", "application/xml", b"<ListBucketResult><CommonPrefixes><Prefix>PMC123456.1/</Prefix></CommonPrefixes></ListBucketResult>")
                    } else if request.starts_with("GET /PMC123456.1/PMC123456.1.json ") {
                        ("200 OK", "application/json", br#"{"pmcid":"PMC123456","version":1,"license_code":"CC BY","xml_url":"s3://pmc-oa-opendata/PMC123456.1/article.nxml","media_urls":["s3://pmc-oa-opendata/PMC123456.1/traces-s1.csv"]}"#)
                    } else if request.starts_with("GET /PMC123456.1/article.nxml ") {
                        ("200 OK", "application/xml", br#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><supplementary-material xlink:href="traces-s1.csv"><label>Data S1</label></supplementary-material></body></article>"#)
                    } else if request.starts_with("GET /PMC123456.1/traces-s1.csv ") {
                        ("200 OK", "text/csv", b"time,value\n0,1\n")
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
        });
        Self {
            base,
            requested,
            stop,
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
        "a complete primary archive must not request the hanging secondary ZIP"
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
