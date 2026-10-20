use super::ArticleFixture;
use biomcp_mcp_contract_client::test_http::TestHttpServer;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

struct HangingZip {
    base: String,
    requested: Arc<AtomicBool>,
    primary_failure_nanos: Arc<AtomicU64>,
    server: TestHttpServer,
}

impl HangingZip {
    fn start(hang: bool) -> Self {
        Self::with_primary(hang, false)
    }

    fn with_primary(hang: bool, primary_failure: bool) -> Self {
        Self::with_collision(hang, primary_failure, false)
    }

    fn with_collision(hang: bool, primary_failure: bool, collision: bool) -> Self {
        let filename = if collision {
            "figshare-supplement.pdf"
        } else {
            "traces-s1.csv"
        };
        let primary_bytes: &[u8] = if collision {
            b"%PDF-1.4\nPrimary archive supplement\n%%EOF\n"
        } else {
            b"time,value\n0,1\n"
        };
        let metadata = format!(
            r#"{{"pmcid":"PMC123456","version":1,"license_code":"CC BY","xml_url":"s3://pmc-oa-opendata/PMC123456.1/article.nxml","media_urls":["s3://pmc-oa-opendata/PMC123456.1/{filename}"]}}"#
        );
        let xml = format!(
            r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><supplementary-material xlink:href="{filename}"><label>Data S1</label></supplementary-material></body></article>"#
        );
        let requested = Arc::new(AtomicBool::new(false));
        let primary_failure_nanos = Arc::new(AtomicU64::new(0));
        let failure_nanos = primary_failure_nanos.clone();
        let mut archive = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        archive
            .start_file("traces-s1.csv", zip::write::SimpleFileOptions::default())
            .unwrap();
        archive.write_all(b"time,value\n0,1\n").unwrap();
        let zip_bytes = archive.finish().unwrap().into_inner();
        let zip_requested = requested.clone();
        let server = TestHttpServer::respond(move |request| {
            let zip_requested = zip_requested.clone();
            let failure_nanos = failure_nanos.clone();
            let zip_bytes = zip_bytes.clone();
            let metadata = metadata.clone();
            let xml = xml.clone();
            async move {
                let request = String::from_utf8(request)
                    .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
                let host = request.lines().find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("host").then_some(value.trim())
                })
                .expect("request host");
                let figshare = format!(
                    r#"{{"id":22474820,"title":"Figshare asset fallback winner","doi":"10.1158/fixture.figshare","url_public_html":"https://aacr.figshare.com/articles/journal_contribution/Fixture/22474820","license":{{"name":"CC BY 4.0","url":"https://creativecommons.org/licenses/by/4.0/"}},"files":[{{"id":39926318,"name":"figshare-supplement.pdf","size":24,"download_url":"http://{host}/file.pdf"}}]}}"#
                );
                if primary_failure && request.starts_with("GET /?list-type=2") {
                    let started = std::time::Instant::now();
                    // sleep reason: inject the fixed 200-millisecond primary failure delay
                    #[allow(clippy::disallowed_methods)]
                    tokio::time::sleep(Duration::from_millis(200)).await;
                    failure_nanos.store(started.elapsed().as_nanos() as u64, Ordering::SeqCst);
                    return Ok(b"HTTP/1.1 200 OK\r\nContent-Type: application/xml\r\nContent-Length: 9\r\nConnection: close\r\n\r\n<records>".to_vec());
                }
                if request.contains("/supplementaryFiles") {
                    zip_requested.store(true, Ordering::SeqCst);
                    if hang { return std::future::pending().await; }
                }
                    let (status, content_type, body): (&str, &str, &[u8]) = if primary_failure
                        && request.contains("/supplementaryFiles")
                    {
                        ("200 OK", "application/zip", &zip_bytes)
                    } else if collision && request.starts_with("GET /graph/v1/paper/PMID:22663015")
                    {
                        ("200 OK", "application/json", br#"{"paperId":"paper-5","title":"Figshare asset fallback winner","openAccessPdf":{"url":"https://aacr.figshare.com/articles/journal_contribution/Fixture/22474820?file=39926318","status":"GREEN","license":"CC BY 4.0"}}"#)
                    } else if collision && request.starts_with("GET /v2/articles/22474820 ") {
                        ("200 OK", "application/json", figshare.as_bytes())
                    } else if collision && request.starts_with("POST /v2/articles/search ") {
                        ("200 OK", "application/json", b"[]")
                    } else if collision && request.starts_with("GET /file.pdf ") {
                        ("200 OK", "application/pdf", b"%PDF-1.4\nFigshare\n%%EOF\n")
                    } else if collision && request.contains("ids=") {
                        (
                            "200 OK",
                            "application/json",
                            br#"{"records":[{"pmid":22663015,"pmcid":"PMC123456"}]}"#,
                        )
                    } else if request.starts_with("GET /?list-type=2") {
                        ("200 OK", "application/xml", b"<ListBucketResult><CommonPrefixes><Prefix>PMC123456.1/</Prefix></CommonPrefixes></ListBucketResult>")
                    } else if request.starts_with("GET /PMC123456.1/PMC123456.1.json ") {
                        ("200 OK", "application/json", metadata.as_bytes())
                    } else if request.starts_with("GET /PMC123456.1/article.nxml ") {
                        ("200 OK", "application/xml", xml.as_bytes())
                    } else if request.starts_with(&format!("GET /PMC123456.1/{filename} ")) {
                        (
                            "200 OK",
                            if collision {
                                "application/pdf"
                            } else {
                                "text/csv"
                            },
                            primary_bytes,
                        )
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
                let mut response = header.into_bytes();
                response.extend_from_slice(body);
                Ok(response)
            }
        })
        .expect("start article archive fixture");
        Self {
            base: server.base.clone(),
            requested,
            primary_failure_nanos,
            server,
        }
    }

    fn finish(&mut self) -> std::io::Result<()> {
        self.server.finish()
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn complete_primary_assets_do_not_wait_for_an_optional_zip() {
    let fixture = ArticleFixture::start();
    let mut zip = HangingZip::start(true);
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
    zip.finish().unwrap();
}

fn configure_archive(fixture: &ArticleFixture, zip: &HangingZip) {
    use std::fs::OpenOptions;
    let mut env = OpenOptions::new()
        .append(true)
        .open(&fixture.env_file)
        .unwrap();
    for key in [
        "BIOMCP_TEST_UNPACED_ORIGIN",
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
    let mut zip = HangingZip::start(false);
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
    zip.finish().unwrap();
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn failed_primary_still_returns_the_zip_assets() {
    let fixture = ArticleFixture::start();
    let mut zip = HangingZip::with_primary(false, true);
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
    zip.finish().unwrap();
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn primary_collision_handle_does_not_consult_the_zip() {
    use std::fs::OpenOptions;
    let fixture = ArticleFixture::start();
    let mut zip = HangingZip::with_collision(true, false, true);
    // Earlier idle and partial requests must not prevent CLI fixture requests.
    let _idle_connection = TcpStream::connect(zip.base.trim_start_matches("http://")).unwrap();
    let mut partial_connection =
        TcpStream::connect(zip.base.trim_start_matches("http://")).unwrap();
    partial_connection.write_all(b"GET / HTTP/1.1\r\n").unwrap();
    configure_archive(&fixture, &zip);
    let mut env = OpenOptions::new()
        .append(true)
        .open(&fixture.env_file)
        .unwrap();
    for key in [
        "BIOMCP_NCBI_IDCONV_BASE",
        "BIOMCP_S2_BASE",
        "BIOMCP_FIGSHARE_BASE",
    ] {
        writeln!(env, "export {key}='{}'", zip.base).unwrap();
    }
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663015", "assets"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let manifest: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let primary = manifest["assets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|asset| {
            asset["filename"] == "figshare-supplement.pdf"
                && asset["provider"]["source"] == "PMC OA"
        })
        .expect("the primary collision asset remains listed");
    let key = primary["asset_key"].as_str().unwrap();
    assert_ne!(
        key, "figshare-supplement.pdf",
        "the fixture must establish a real public collision handle: {manifest}"
    );
    assert!(!zip.requested.load(Ordering::SeqCst));
    let output = fixture
        .biomcp_command(&["get", "article", "22663015", "asset", key])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        output.stdout,
        b"%PDF-1.4\nPrimary archive supplement\n%%EOF\n"
    );
    assert!(
        !zip.requested.load(Ordering::SeqCst),
        "listed primary collision handles must not request the hanging ZIP"
    );
    // Shutdown also handles an incomplete request whose client remains connected.
    zip.finish().unwrap();
}

#[cfg(unix)]
#[test]
fn disconnected_fixture_client_does_not_panic_or_block_other_requests() {
    use std::os::fd::AsRawFd;
    let mut server = TestHttpServer::respond(|_| async {
        Ok(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok".to_vec())
    })
    .unwrap();
    let address = server.base.trim_start_matches("http://");
    let client = TcpStream::connect(address).unwrap();
    let linger = libc::linger {
        l_onoff: 1,
        l_linger: 0,
    };
    // SAFETY: the descriptor is live and the option points at a valid linger value.
    assert_eq!(
        unsafe {
            libc::setsockopt(
                client.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_LINGER,
                std::ptr::from_ref(&linger).cast(),
                std::mem::size_of_val(&linger) as libc::socklen_t,
            )
        },
        0
    );
    drop(client);
    let _idle = TcpStream::connect(address).unwrap();
    let mut partial = TcpStream::connect(address).unwrap();
    partial.write_all(b"GET /").unwrap();
    let mut survivor = TcpStream::connect(address).unwrap();
    survivor
        .set_read_timeout(Some(Duration::from_secs(60)))
        .unwrap();
    survivor
        .write_all(b"GET /survives HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    survivor.read_to_string(&mut response).unwrap();
    assert!(response.ends_with("\r\n\r\nok"), "{response}");
    server.finish().unwrap();
}
