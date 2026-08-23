use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(unix)]
#[path = "pty_helpers.rs"]
mod pty_helpers;

struct ArticleFixture {
    root: PathBuf,
    env_file: PathBuf,
}

impl ArticleFixture {
    fn start() -> Self {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
        let output = Command::new("bash")
            .current_dir(repo)
            .args([
                "spec/fixtures/setup-article-fulltext-source-fixture.sh",
                repo.to_str().expect("repository path should be UTF-8"),
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
        let env_file = repo.join(".cache/spec-article-fulltext-source-env");
        assert!(env_file.is_file(), "fixture environment should exist");
        Self { root, env_file }
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
                repo.to_str().expect("repository path should be UTF-8"),
            ])
            .status();
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
#[test]
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
