use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
#[path = "pty_helpers.rs"]
mod pty_helpers;

#[path = "article_asset_terminal/supplement_cache.rs"]
mod supplement_cache;

struct ArticleFixture {
    root: PathBuf,
    workspace: PathBuf,
    env_file: PathBuf,
}

impl ArticleFixture {
    fn start() -> Self {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace = repo.join(".cache").join(format!(
            "article-asset-terminal.{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should follow the Unix epoch")
                .as_nanos()
        ));
        fs::create_dir_all(&workspace).expect("fixture workspace should be created");
        let output = Command::new("bash")
            .current_dir(repo)
            .args([
                "spec/fixtures/setup-article-fulltext-source-fixture.sh",
                workspace
                    .to_str()
                    .expect("fixture workspace path should be UTF-8"),
            ])
            .output()
            .expect("article fixture setup should run");
        assert!(
            output.status.success(),
            "article fixture setup failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let root = PathBuf::from(
            String::from_utf8(output.stdout)
                .expect("fixture root should be UTF-8")
                .trim(),
        );
        let env_file = workspace.join(".cache/spec-article-fulltext-source-env");
        assert!(env_file.is_file(), "fixture environment should exist");
        Self {
            root,
            workspace,
            env_file,
        }
    }

    fn biomcp_command(&self, args: &[&str]) -> Command {
        let mut command = Command::new("bash");
        command.args([
            "-c",
            "source \"$1\"; shift; exec \"$@\"",
            "bash",
            self.env_file
                .to_str()
                .expect("fixture environment path should be UTF-8"),
            env!("CARGO_BIN_EXE_biomcp"),
        ]);
        command.args(args);
        command
    }
}

impl Drop for ArticleFixture {
    fn drop(&mut self) {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let _ = Command::new("bash")
            .current_dir(repo)
            .args([
                "spec/fixtures/cleanup-article-fulltext-source-fixture.sh",
                self.workspace
                    .to_str()
                    .expect("fixture workspace path should be UTF-8"),
            ])
            .status();
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

#[cfg(unix)]
#[test]
#[serial_test::serial(article_output_fixture)]
fn binary_article_assets_protect_terminals_and_preserve_files() {
    let fixture = ArticleFixture::start();
    let args = ["get", "article", "22663011", "asset", "traces-s1.csv"];
    let expected = b"time,value\n0,1\n";

    let terminal = pty_helpers::run_command_with_tty(fixture.biomcp_command(&args))
        .expect("binary asset command should complete in a terminal");
    assert!(
        !terminal.contains("time,value"),
        "binary asset bytes must not reach an interactive terminal:\n{terminal}"
    );
    assert!(
        terminal.contains("--output"),
        "terminal refusal should explain the file destination:\n{terminal}"
    );

    let piped = fixture
        .biomcp_command(&args)
        .output()
        .expect("piped binary asset command should run");
    assert!(
        piped.status.success(),
        "piped asset should succeed\nstdout:\n{:?}\nstderr:\n{}",
        piped.stdout,
        String::from_utf8_lossy(&piped.stderr)
    );
    assert_eq!(piped.stdout, expected);

    for destination in [
        fixture.root.join("supplement.docx"),
        PathBuf::from("./explicit.pdf"),
        PathBuf::from("bare.pdf"),
    ] {
        let output = fixture
            .biomcp_command(&[
                "get",
                "article",
                "22663011",
                "asset",
                "traces-s1.csv",
                "--output",
                destination
                    .to_str()
                    .expect("destination path should be UTF-8"),
            ])
            .current_dir(&fixture.root)
            .output()
            .expect("file-destination asset command should run");
        assert!(
            output.status.success(),
            "file destination {} should succeed\nstdout:\n{}\nstderr:\n{}",
            destination.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            fs::read(fixture.root.join(destination)).expect("destination should exist"),
            expected
        );
    }

    let missing = fixture
        .biomcp_command(&[
            "get",
            "article",
            "22663011",
            "asset",
            "traces-s1.csv",
            "--output",
            "missing/bare.pdf",
        ])
        .current_dir(&fixture.root)
        .output()
        .expect("missing-directory asset command should run");
    assert!(!missing.status.success());
    assert!(
        String::from_utf8_lossy(&missing.stderr)
            .contains("Output directory does not exist: missing"),
        "a real missing directory should still be refused: {:?}",
        missing
    );
    assert!(!fixture.root.join("missing").exists());
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn asset_output_directory_destinations_name_the_file_path_error() {
    let fixture = ArticleFixture::start();
    fs::create_dir(fixture.root.join("existing-directory")).expect("create output directory");
    let mut failures = Vec::new();
    for destination in ["./", "existing-directory"] {
        let output = fixture
            .biomcp_command(&[
                "get",
                "article",
                "22663011",
                "asset",
                "traces-s1.csv",
                "--output",
                destination,
            ])
            .current_dir(&fixture.root)
            .output()
            .expect("run directory output command");
        let message = String::from_utf8_lossy(&output.stderr);
        eprintln!(
            "--output {destination}: exit {:?}; {message}",
            output.status.code()
        );
        if output.status.success()
            || !message.contains("is a directory")
            || !message.contains(destination)
        {
            failures.push(format!("{destination}: {message}"));
        }
        assert!(fixture.root.is_dir());
        assert!(fixture.root.join("existing-directory").is_dir());
    }
    assert!(
        failures.is_empty(),
        "output must name the directory destination: {failures:?}"
    );
}

fn successful_output(fixture: &ArticleFixture, args: &[&str]) -> std::process::Output {
    let output = fixture
        .biomcp_command(args)
        .output()
        .expect("article command should run");
    assert!(
        output.status.success(),
        "article command should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_fulltext_out_uses_findable_name_and_frontmatter() {
    let fixture = ArticleFixture::start();
    let output_dir = fixture.root.join("owned-library");
    fs::create_dir(&output_dir).expect("owned output directory should be created");

    successful_output(
        &fixture,
        &[
            "get",
            "article",
            "22663011",
            "fulltext",
            "--out",
            output_dir
                .to_str()
                .expect("output directory should be UTF-8"),
        ],
    );

    let saved = output_dir.join("22663011-europe-full-text-winner.md");
    let document = fs::read_to_string(&saved).expect("findable fulltext file should exist");
    let (frontmatter, body) = document
        .strip_prefix("---\n")
        .and_then(|value| value.split_once("\n---\n"))
        .expect("fulltext should start with YAML frontmatter");
    let fields = frontmatter
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(key, value)| (key.trim(), value.trim().trim_matches('"')))
        .collect::<std::collections::HashMap<_, _>>();

    for (key, expected) in [
        ("pmid", "22663011"),
        ("pmcid", "PMC123456"),
        ("title", "Europe full text winner"),
        ("journal", "Journal One"),
        ("date", "2025-01-01"),
        ("source-rung", "Europe PMC XML"),
    ] {
        assert_eq!(fields.get(key), Some(&expected), "wrong {key} frontmatter");
    }
    assert!(
        fields.contains_key("doi"),
        "frontmatter should retain the DOI field"
    );
    let retrieved_at = fields
        .get("retrieved-at")
        .expect("frontmatter should record retrieval time");
    chrono::DateTime::parse_from_rfc3339(retrieved_at)
        .expect("retrieved-at should be an RFC 3339 timestamp");
    assert!(
        body.contains("Europe PMC body text"),
        "saved document should retain the resolved article body"
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_asset_out_preserves_its_name_and_bytes() {
    let fixture = ArticleFixture::start();
    let output_dir = fixture.root.join("owned-assets");
    fs::create_dir(&output_dir).expect("owned output directory should be created");

    successful_output(
        &fixture,
        &[
            "get",
            "article",
            "22663011",
            "asset",
            "traces-s1.csv",
            "--out",
            output_dir
                .to_str()
                .expect("output directory should be UTF-8"),
        ],
    );

    assert_eq!(
        fs::read(output_dir.join("traces-s1.csv"))
            .expect("asset should be saved under its own name"),
        b"time,value\n0,1\n"
    );
}

fn two_route_fixture_bytes() -> &'static [u8] {
    b"PK\x03\x04\nscrubbed two-route supplement XLSX fixture bytes\n"
}

fn word_case_fixture_bytes() -> &'static [u8] {
    b"PK\x03\x04\x14\x00\x06\x00scrubbed Word supplement DOCX fixture bytes\n"
}

fn pdf_case_fixture_bytes() -> &'static [u8] {
    b"%PDF-1.4\nscrubbed fixture PDF supplement bytes\n%%EOF\n"
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn empty_europe_pmc_supplement_finds_the_pmc_oa_bucket_copy() {
    // The Word case (PMC6168352): the only listed copy is
    // Europe PMC's zero-byte member and the package download fails, so the
    // PMC open-data bucket object the manifest names is fetched per file.
    let fixture = ArticleFixture::start();
    let filename = "NIHMS1501645-supplement-2.docx";

    let manifest = successful_output(
        &fixture,
        &["--json", "get", "article", "22663031", "assets"],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&manifest.stdout).expect("asset manifest should be JSON");
    let listed = doc["assets"]
        .as_array()
        .expect("assets should be a list")
        .iter()
        .filter(|asset| asset["filename"] == filename)
        .collect::<Vec<_>>();
    assert_eq!(
        listed.len(),
        1,
        "the bucket copy must be listed; manifest:\n{}",
        String::from_utf8_lossy(&manifest.stdout)
    );
    assert_eq!(listed[0]["provider"]["source"], "PMC OA");
    assert_eq!(
        listed[0]["size_bytes"],
        serde_json::json!(word_case_fixture_bytes().len())
    );

    let rows = doc["coverage"]
        .as_array()
        .expect("coverage should be a list")
        .iter()
        .filter(|row| row["filename"] == filename)
        .collect::<Vec<_>>();
    assert!(
        rows.iter()
            .any(|row| row["outcome"] == "empty_response"
                && row["provider"]["source"] == "Europe PMC"),
        "the empty Europe PMC member stays marked"
    );

    let destination = fixture.root.join("word-case.docx");
    successful_output(
        &fixture,
        &[
            "get",
            "article",
            "22663031",
            "asset",
            filename,
            "--output",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
        ],
    );
    assert_eq!(
        fs::read(&destination).expect("the bucket copy should download"),
        word_case_fixture_bytes()
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn wrong_type_supplement_reply_fails_the_download_and_writes_no_file() {
    // The wrong-type case (PMC7787686): a vendor HTML page
    // under a .docx name is refused with its own outcome instead of being
    // delivered as the file.
    let fixture = ArticleFixture::start();
    let filename = "pgen.1009231.s004.docx";

    let manifest = successful_output(
        &fixture,
        &["--json", "get", "article", "22663032", "assets"],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&manifest.stdout).expect("asset manifest should be JSON");
    assert!(
        doc["assets"]
            .as_array()
            .expect("assets should be a list")
            .iter()
            .all(|asset| asset["filename"] != filename),
        "wrong-type bytes must never be listed as an asset"
    );
    let rows = doc["coverage"]
        .as_array()
        .expect("coverage should be a list")
        .iter()
        .filter(|row| row["filename"] == filename)
        .collect::<Vec<_>>();
    let europe_row = rows
        .iter()
        .find(|row| row["provider"]["source"] == "Europe PMC")
        .expect("the Europe PMC route stays named");
    assert_eq!(europe_row["outcome"], "unusable_asset");
    assert!(
        rows.iter()
            .any(|row| row["outcome"] == "unusable_asset" && row["provider"]["source"] == "PMC OA")
    );
    let europe_key = europe_row["asset_key"]
        .as_str()
        .expect("the wrong-type route keeps a stable key")
        .to_string();

    let destination = fixture.root.join("wrong-type.docx");
    let output = fixture
        .biomcp_command(&[
            "get",
            "article",
            "22663032",
            "asset",
            &europe_key,
            "--output",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
        ])
        .output()
        .expect("wrong-type asset command should run");
    assert!(
        !output.status.success(),
        "a wrong-type asset reply must fail the download\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("article_asset_not_retrievable"),
        "refusal should name the failed retrieval:\n{stderr}"
    );
    assert!(
        stderr.contains("unusable_asset"),
        "refusal should name the unusable bytes:\n{stderr}"
    );
    assert!(
        !destination.exists(),
        "a failed download must write no output file"
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn two_route_pdf_supplement_prefers_the_pmc_oa_copy() {
    // The PDF case (PMC4858276): Europe PMC's zero-byte PDF
    // member beside a figure with bytes, and the PMC OA package holding the
    // real file.
    let fixture = ArticleFixture::start();
    let filename = "pone.0153788.s001.pdf";

    let manifest = successful_output(
        &fixture,
        &["--json", "get", "article", "22663033", "assets"],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&manifest.stdout).expect("asset manifest should be JSON");
    let listed = doc["assets"]
        .as_array()
        .expect("assets should be a list")
        .iter()
        .filter(|asset| asset["filename"] == filename)
        .collect::<Vec<_>>();
    assert_eq!(listed.len(), 1, "only the PMC OA copy may list the PDF");
    assert_eq!(listed[0]["provider"]["source"], "PMC OA");
    assert_eq!(
        listed[0]["size_bytes"],
        serde_json::json!(pdf_case_fixture_bytes().len())
    );
    assert!(
        doc["coverage"]
            .as_array()
            .expect("coverage should be a list")
            .iter()
            .any(|row| row["filename"] == filename
                && row["outcome"] == "empty_response"
                && row["provider"]["source"] == "Europe PMC")
    );

    let destination = fixture.root.join("pdf-case.pdf");
    successful_output(
        &fixture,
        &[
            "get",
            "article",
            "22663033",
            "asset",
            filename,
            "--output",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
        ],
    );
    let bytes = fs::read(&destination).expect("the full PDF copy should download");
    assert_eq!(bytes, pdf_case_fixture_bytes());
    assert!(bytes.starts_with(b"%PDF-"));
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn empty_supplement_reply_fails_the_download_and_writes_no_file() {
    let fixture = ArticleFixture::start();
    let destination = fixture.root.join("empty-supplement.xlsx");
    let output = fixture
        .biomcp_command(&[
            "get",
            "article",
            "22663025",
            "asset",
            "NIHMS1501644-supplement-1.xlsx",
            "--output",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
        ])
        .output()
        .expect("empty-route asset command should run");
    assert!(
        !output.status.success(),
        "a zero-byte asset reply must fail the download\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("article_asset_not_retrievable"),
        "refusal should name the failed retrieval:\n{stderr}"
    );
    assert!(
        stderr.contains("empty_response"),
        "refusal should name the empty response:\n{stderr}"
    );
    assert!(
        !destination.exists(),
        "a failed download must write no output file"
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn two_route_supplement_prefers_bytes_and_marks_the_empty_alternative() {
    let fixture = ArticleFixture::start();
    let filename = "NIHMS1501643-supplement-2.xlsx";

    let manifest = successful_output(
        &fixture,
        &["--json", "get", "article", "22663024", "assets"],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&manifest.stdout).expect("asset manifest should be JSON");
    let assets = doc["assets"].as_array().expect("assets should be a list");
    let listed = assets
        .iter()
        .filter(|asset| asset["filename"] == filename)
        .collect::<Vec<_>>();
    assert_eq!(
        listed.len(),
        1,
        "only the route that returned bytes may list the supplement"
    );
    assert_eq!(listed[0]["provider"]["source"], "PMC OA");
    assert_eq!(
        listed[0]["size_bytes"],
        serde_json::json!(two_route_fixture_bytes().len())
    );

    let empty_rows = doc["coverage"]
        .as_array()
        .expect("coverage should be a list")
        .iter()
        .filter(|row| row["filename"] == filename)
        .collect::<Vec<_>>();
    assert_eq!(empty_rows.len(), 1, "the empty alternative stays marked");
    assert_eq!(empty_rows[0]["outcome"], "empty_response");
    assert_eq!(empty_rows[0]["provider"]["source"], "Europe PMC");
    let empty_key = empty_rows[0]["asset_key"]
        .as_str()
        .expect("the marked alternative keeps a stable key")
        .to_string();
    assert!(
        empty_key.starts_with("europe-pmc-")
            && empty_key.ends_with("--NIHMS1501643-supplement-2.xlsx"),
        "unexpected empty-route key: {empty_key}"
    );

    let destination = fixture.root.join("full-supplement.xlsx");
    successful_output(
        &fixture,
        &[
            "get",
            "article",
            "22663024",
            "asset",
            filename,
            "--output",
            destination
                .to_str()
                .expect("destination path should be UTF-8"),
        ],
    );
    assert_eq!(
        fs::read(&destination).expect("the full copy should download"),
        two_route_fixture_bytes()
    );

    let refused = fixture.root.join("refused-supplement.xlsx");
    let output = fixture
        .biomcp_command(&[
            "get",
            "article",
            "22663024",
            "asset",
            &empty_key,
            "--output",
            refused.to_str().expect("destination path should be UTF-8"),
        ])
        .output()
        .expect("empty-route asset command should run");
    assert!(
        !output.status.success(),
        "the empty route must refuse instead of writing a zero-byte file\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("empty_response"),
        "refusal should name the empty response:\n{stderr}"
    );
    assert!(
        !refused.exists(),
        "the refused route must write no output file"
    );
}

fn article_fulltext_pdf_attempt(doc: &serde_json::Value) -> &serde_json::Value {
    doc["full_text_coverage"]["attempts"]
        .as_array()
        .expect("full_text_coverage attempts should be a list")
        .iter()
        .find(|attempt| attempt["source_kind"] == "pdf")
        .unwrap_or_else(|| panic!("no pdf attempt recorded: {doc}"))
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn publisher_open_pdf_link_downloads_and_returns_its_text() {
    let fixture = ArticleFixture::start();

    let output = successful_output(
        &fixture,
        &["--json", "get", "article", "22663026", "fulltext", "--pdf"],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("article JSON should parse");
    let manifest = &doc["full_text_manifest"];
    assert_eq!(manifest["source_kind"], "pdf");
    assert_eq!(manifest["provider"]["label"], "Semantic Scholar PDF");
    assert_eq!(manifest["provenance"]["pdf_fallback_used"], true);
    assert!(
        manifest["reuse"]["license"]
            .as_str()
            .unwrap()
            .contains("CC BY")
    );

    let attempt = article_fulltext_pdf_attempt(&doc);
    assert_eq!(attempt["outcome"], "data");
    assert_eq!(attempt["coverage"], "full_text");
    assert_eq!(attempt["reason"], "body_detected");

    let saved = doc["full_text_path"]
        .as_str()
        .expect("pdf fulltext should name its saved path");
    let text = fs::read_to_string(saved).expect("captured PDF text should be saved");
    let lower = text.to_lowercase();
    assert!(
        lower.contains("controller"),
        "saved text lost the paper title"
    );
    assert!(
        lower.contains("investment"),
        "saved text lost the paper title"
    );
    assert!(lower.contains("qin"), "saved text lost the author byline");
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn open_pdf_responses_distinguish_unavailable_bytes_from_unusable_text() {
    let fixture = ArticleFixture::start();
    for (pmid, reason, coverage) in [
        ("22663041", "open_pdf_download_failed", "unavailable"),
        ("22663042", "open_pdf_download_failed", "unavailable"),
        ("22663043", "open_pdf_download_failed", "unavailable"),
        ("22663044", "open_pdf_text_unusable", "unusable"),
    ] {
        let output = successful_output(
            &fixture,
            &["--json", "get", "article", pmid, "fulltext", "--pdf"],
        );
        let doc: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("article JSON should parse");
        let attempt = article_fulltext_pdf_attempt(&doc);
        assert_eq!(
            attempt["reason"], reason,
            "wrong PDF response reason for {pmid}: {doc}"
        );
        assert_eq!(
            attempt["coverage"], coverage,
            "wrong PDF coverage for {pmid}"
        );
        assert_eq!(attempt["outcome"], "unavailable");
        assert_eq!(doc["full_text_path"], serde_json::Value::Null);
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn open_pdf_failures_name_the_part_that_failed() {
    let fixture = ArticleFixture::start();

    // Each recorded shape names its failed part instead of a bare
    // source_unavailable/no_content. 22663030 has no full-text winner, so
    // the PDF step runs and its link could not even be asked for.
    for (pmid, expected_reason, outcome) in [
        ("22663019", "open_pdf_link_absent", "empty"),
        ("22663030", "open_pdf_link_unavailable", "unavailable"),
        ("22663027", "open_pdf_link_rejected", "unavailable"),
        ("22663028", "open_pdf_download_failed", "unavailable"),
        ("22663014", "open_pdf_download_absent", "empty"),
        ("22663029", "open_pdf_text_unusable", "unavailable"),
    ] {
        let output = fixture
            .biomcp_command(&["--json", "get", "article", pmid, "fulltext", "--pdf"])
            .output()
            .expect("article fulltext command should run");
        assert!(
            output.status.success(),
            "{pmid} fulltext --pdf should still exit 0\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let recorded = String::from_utf8_lossy(&output.stdout).to_lowercase();
        assert!(
            !recorded.contains("secret"),
            "{pmid} recorded output echoed the refused URL: {recorded}"
        );
        let doc: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("article JSON should parse");
        let attempt = article_fulltext_pdf_attempt(&doc);
        assert_eq!(
            attempt["reason"], expected_reason,
            "wrong pdf-step reason for {pmid}: {doc}"
        );
        assert_eq!(
            attempt["outcome"], outcome,
            "wrong pdf-step outcome for {pmid}"
        );
        assert_eq!(doc["full_text_path"], serde_json::Value::Null);
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_without_a_pmc_record_reports_no_fulltext_without_an_invalid_xml_request() {
    let fixture = ArticleFixture::start();
    let output = successful_output(
        &fixture,
        &["--json", "get", "article", "22663030", "fulltext"],
    );
    let doc: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        doc["section_outcomes"]["fulltext"]["outcome"], "empty",
        "{doc}"
    );
    assert_eq!(doc["full_text_coverage"]["coverage"], "none");
    assert!(
        doc["full_text_coverage"]["attempts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|attempt| attempt["outcome"] == "empty" && attempt["reason"] == "no_content")
    );
    assert!(!String::from_utf8_lossy(&output.stdout).contains("sources failed"));
    let requests = fs::read_to_string(fixture.root.join("request-log.txt")).unwrap();
    assert!(!requests.contains("fulltext:xml:"), "{requests}");
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_fulltext_xml_absence_does_not_retry_but_an_outage_does() {
    let fixture = ArticleFixture::start();
    for (pmid, expected_outcome, request, retries) in [
        ("22663014", "empty", "fulltext:xml:europepmc-pmc", false),
        (
            "22663019",
            "unavailable",
            "fulltext:xml:europepmc-pmc-outage",
            true,
        ),
    ] {
        let output = successful_output(&fixture, &["--json", "get", "article", pmid, "fulltext"]);
        let doc: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            doc["section_outcomes"]["fulltext"]["outcome"], expected_outcome,
            "{doc}"
        );
        let attempts = doc["full_text_coverage"]["attempts"].as_array().unwrap();
        let attempt = attempts
            .iter()
            .find(|attempt| attempt["provider"]["label"] == "Europe PMC XML")
            .unwrap();
        assert_eq!(attempt["outcome"], expected_outcome);
        assert_eq!(
            attempt["reason"],
            if retries {
                "source_unavailable"
            } else {
                "no_content"
            }
        );
        let requests = fs::read_to_string(fixture.root.join("request-log.txt")).unwrap();
        let count = requests.lines().filter(|line| *line == request).count();
        if retries {
            assert!(
                count > 1,
                "outage must retain transient retries: {requests}"
            );
        } else {
            assert_eq!(count, 1, "absence must not retry: {requests}");
        }
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_fulltext_without_out_keeps_the_managed_cache_path() {
    let fixture = ArticleFixture::start();

    let output = successful_output(&fixture, &["get", "article", "22663011", "fulltext"]);
    let stdout = String::from_utf8(output.stdout).expect("article output should be UTF-8");
    let saved_path = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Saved to: "))
        .map(PathBuf::from)
        .expect("default summary should report its cache path");

    assert_eq!(
        saved_path.parent(),
        Some(fixture.root.join("cache/downloads").as_path())
    );
    assert_eq!(
        saved_path.extension().and_then(|value| value.to_str()),
        Some("txt")
    );
    assert!(
        saved_path.is_file(),
        "default fulltext should remain in the managed cache"
    );
    assert!(
        !fixture
            .root
            .join("22663011-europe-full-text-winner.md")
            .exists(),
        "default retrieval should not create a user-library copy"
    );
    let saved_bytes = fs::read(&saved_path).unwrap();
    let export_dir = fixture.root.join("saved-export");
    fs::create_dir(&export_dir).unwrap();
    successful_output(
        &fixture,
        &[
            "get",
            "article",
            "22663011",
            "fulltext",
            "--out",
            export_dir.to_str().unwrap(),
        ],
    );
    let export = export_dir.join("22663011-europe-full-text-winner.md");
    let exported_bytes = fs::read(&export).unwrap();
    for command in [
        vec!["cache", "clean", "--max-size", "1B"],
        vec!["cache", "clear", "--yes"],
    ] {
        successful_output(&fixture, &command);
        assert_eq!(fs::read(&saved_path).unwrap(), saved_bytes);
        assert_eq!(fs::read(&export).unwrap(), exported_bytes);
        let view = successful_output(
            &fixture,
            &["get", "article", "22663011", "fulltext", "--lines", "1:2"],
        );
        assert!(!view.stdout.is_empty());
        assert_eq!(fs::read(&saved_path).unwrap(), saved_bytes);
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn supplement_search_finds_both_acceptance_examples_at_their_locators() {
    let fixture = ArticleFixture::start();

    let search = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "30209399",
            "F1761I",
        ],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&search.stdout).expect("supplement search should be JSON");
    let hits = doc["hits"].as_array().expect("hits should be a list");
    assert_eq!(doc["hits_truncated"], false);
    assert_eq!(doc["incomplete"], false);
    assert_eq!(hits.len(), 1, "one acceptance hit is expected: {doc}");
    assert_eq!(hits[0]["worksheet"], "Sheet1");
    assert_eq!(hits[0]["row"], serde_json::json!(2780));
    assert_eq!(hits[0]["locator"], "Sheet1 row 2780");
    assert_eq!(hits[0]["format"], "xlsx");
    assert_eq!(hits[0]["filename"], "supplementary-table-s1.xlsx");
    assert!(
        hits[0]["context"]
            .as_str()
            .expect("context should be text")
            .contains("BRCA1"),
        "the hit context should carry the matched row text"
    );
    let sha256 = hits[0]["sha256"].as_str().expect("checksum should be text");
    assert_eq!(
        sha256.len(),
        64,
        "the source file's checksum rides each hit"
    );
    let files = doc["files"].as_array().expect("files should be a list");
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["outcome"], "searched");
    assert_eq!(files[0]["sha256"], hits[0]["sha256"]);

    let cache = fixture.root.join("cache/biomcp.db");
    let key = supplement_cache::key(sha256);
    let mut cached = supplement_cache::read(&cache, &key);
    cached["format"] = serde_json::json!("csv");
    cached["extraction"]["blocks"] = serde_json::json!([]);
    supplement_cache::write(&cache, &key, &cached);
    let again = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "30209399",
            "F1761I",
        ],
    );
    let again: serde_json::Value = serde_json::from_slice(&again.stdout).unwrap();
    assert_eq!(
        again["hits"][0]["row"], 2780,
        "incompatible cached format must be reparsed"
    );

    let mut cached = supplement_cache::read(&cache, &key);
    let mut block = cached["extraction"]["blocks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["row"] == 2780)
        .unwrap()
        .clone();
    block["text"] = serde_json::json!(format!("F1761I{}", "é".repeat(400)));
    cached["extraction"]["blocks"] = serde_json::json!([block]);
    supplement_cache::write(&cache, &key, &cached);
    let unicode = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "30209399",
            "F1761I",
        ],
    );
    let unicode: serde_json::Value = serde_json::from_slice(&unicode.stdout).unwrap();
    assert!(unicode["hits"][0]["context"].as_str().unwrap().len() <= 400);

    for needle in ["P245S", "Pro245Ser", "p.Pro245Ser"] {
        let search = successful_output(
            &fixture,
            &["--json", "article", "supplement-search", "28944238", needle],
        );
        let doc: serde_json::Value =
            serde_json::from_slice(&search.stdout).expect("supplement search should be JSON");
        let hits = doc["hits"].as_array().expect("hits should be a list");
        assert_eq!(doc["query"], needle);
        assert_eq!(
            hits.len(),
            1,
            "one acceptance hit is expected for {needle}: {doc}"
        );
        assert_eq!(hits[0]["worksheet"], "ST2");
        assert_eq!(hits[0]["row"], serde_json::json!(85));
        assert_eq!(hits[0]["locator"], "ST2 row 85");
        let context = hits[0]["context"].as_str().expect("context should be text");
        assert!(context.contains("AXIN2") && context.contains("p.Pro245Ser"));
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn supplement_search_replays_named_unreachable_files_and_keeps_readable_hits() {
    let fixture = ArticleFixture::start();
    for (pmid, expected, outcome) in [
        (
            "38368708",
            vec!["NIHMS2014154-supplement-Supplementary.docx"],
            "source_unavailable",
        ),
        (
            "27397505",
            vec![
                "mmc3.xlsx",
                "mmc4.xlsx",
                "mmc5.xlsx",
                "mmc7.xlsx",
                "mmc9.pdf",
            ],
            "pmc_proof_of_work",
        ),
    ] {
        let output = successful_output(
            &fixture,
            &["--json", "article", "supplement-search", pmid, "I448T"],
        );
        let doc: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(doc["incomplete"], true, "{doc}");
        let files = doc["files"].as_array().unwrap();
        for name in &expected {
            let row = files
                .iter()
                .find(|row| row["filename"] == *name)
                .expect("named unread file");
            assert_eq!(row["outcome"], outcome, "{doc}");
            assert!(row.get("sha256").is_none());
        }
        let output = successful_output(&fixture, &["article", "supplement-search", pmid, "I448T"]);
        let markdown = String::from_utf8(output.stdout).unwrap();
        assert!(markdown.contains("Search incomplete"));
        for name in &expected {
            assert!(markdown.contains(name));
        }
        assert!(markdown.contains(outcome));
        if pmid == "27397505" {
            assert_eq!(
                files.len(),
                6,
                "five blocked files and one authored readable control: {doc}"
            );
            assert_eq!(doc["hits"][0]["filename"], "readable-control.csv");
            assert!(markdown.contains("authored readable control"));
        }
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn supplement_search_names_unreachable_files_and_marks_both_formats_incomplete() {
    let fixture = ArticleFixture::start();
    let output = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "22663023",
            "I448T",
        ],
    );
    let doc: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        doc["incomplete"], true,
        "unreachable files prevent an absence claim: {doc}"
    );
    let files = doc["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(
        files[0]["filename"],
        "NIHMS265402-supplement-Supplementary_Tables.xls"
    );
    assert_eq!(files[0]["outcome"], "pmc_proof_of_work");
    assert!(
        files[0].get("sha256").is_none(),
        "unread bytes have no checksum"
    );
    let filtered = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "22663023",
            "I448T",
            "--asset",
            "NIHMS265402-supplement-Supplementary_Tables.xls",
        ],
    );
    let filtered: serde_json::Value = serde_json::from_slice(&filtered.stdout).unwrap();
    assert_eq!(filtered["files"], doc["files"]);
    assert_eq!(filtered["incomplete"], true);
    let output = successful_output(
        &fixture,
        &["article", "supplement-search", "22663023", "I448T"],
    );
    let markdown = String::from_utf8(output.stdout).unwrap();
    assert!(markdown.contains("Search incomplete"));
    assert!(markdown.contains("NIHMS265402-supplement-Supplementary_Tables.xls"));
    assert!(markdown.contains("pmc_proof_of_work"));
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn supplement_search_reports_an_unreadable_file_with_format_and_reason() {
    let fixture = ArticleFixture::start();

    let search = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "22663024",
            "F1761I",
        ],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&search.stdout).expect("supplement search should be JSON");
    assert_eq!(doc["incomplete"], true);
    let hits = doc["hits"].as_array().expect("hits should be a list");
    assert_eq!(hits.len(), 0, "the scrubbed xlsx bytes hold no text: {doc}");
    let files = doc["files"].as_array().expect("files should be a list");
    assert_eq!(files.len(), 1, "the one supplement is reported: {doc}");
    assert_eq!(files[0]["outcome"], "unextractable");
    assert_eq!(files[0]["format"], "xlsx");
    assert_eq!(files[0]["filename"], "NIHMS1501643-supplement-2.xlsx");
    assert!(
        files[0]["reason"]
            .as_str()
            .expect("reason should be text")
            .contains("workbook could not be opened"),
        "the refusal should name why the file could not be read: {doc}"
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn supplement_search_reports_an_absent_search_and_supports_markdown() {
    let fixture = ArticleFixture::start();

    let search = successful_output(
        &fixture,
        &["--json", "article", "supplement-search", "30209399", "G12D"],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&search.stdout).expect("supplement search should be JSON");
    assert_eq!(doc["hits"].as_array().expect("hits list").len(), 0);
    assert_eq!(doc["files"][0]["outcome"], "searched");

    let literal = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "30209399",
            " F1761I",
        ],
    );
    let literal: serde_json::Value = serde_json::from_slice(&literal.stdout).unwrap();
    assert_eq!(literal["query"], " F1761I");
    assert!(
        literal["hits"].as_array().unwrap().is_empty(),
        "spaces are literal search text"
    );

    let rendered = successful_output(
        &fixture,
        &["article", "supplement-search", "30209399", "F1761I"],
    );
    let markdown = String::from_utf8(rendered.stdout).unwrap();
    assert!(markdown.contains("Search complete"));
    assert!(markdown.contains("Sheet1 row 2780"));

    let filtered = successful_output(
        &fixture,
        &[
            "--json",
            "article",
            "supplement-search",
            "30209399",
            "F1761I",
            "--asset",
            "supplementary-table-s1.xlsx",
        ],
    );
    let doc: serde_json::Value =
        serde_json::from_slice(&filtered.stdout).expect("filtered search should be JSON");
    assert_eq!(
        doc["hits"].as_array().expect("hits list").len(),
        1,
        "the asset filter still searches the one file: {doc}"
    );
}
