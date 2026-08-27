use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
#[path = "pty_helpers.rs"]
mod pty_helpers;

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

    let destination = fixture.root.join("supplement.docx");
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
        .output()
        .expect("file-destination asset command should run");
    assert!(
        output.status.success(),
        "file destination should succeed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read(destination).expect("destination should exist"),
        expected
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
}
