use std::process::{Command, Output};

fn run_compatibility(json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(["--no-cache"]);
    if json {
        command.arg("--json");
    }
    command
        .args(["article", "batch", "not-an-article-id", "also-invalid"])
        .output()
        .expect("run article batch")
}

fn run_canonical(json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_biomcp"));
    command.args(["--no-cache"]);
    if json {
        command.arg("--json");
    }
    command
        .args([
            "batch",
            "article",
            "not-an-article-id,also-invalid",
            "--mode",
            "compact",
        ])
        .output()
        .expect("run canonical compact article batch")
}

const JSON_GOLDEN: &str = r#"{
  "items": [
    {
      "error": {
        "code": "invalid_argument",
        "message": "Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
      },
      "input": "not-an-article-id",
      "status": "error"
    },
    {
      "error": {
        "code": "invalid_argument",
        "message": "Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved."
      },
      "input": "also-invalid",
      "status": "error"
    }
  ],
  "summary": {
    "failed": 2,
    "succeeded": 0,
    "total": 2
  }
}
"#;

const MARKDOWN_GOLDEN: &str = r#"# Batch: article (2)

---

## not-an-article-id — error

Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved.

---

## also-invalid — error

Invalid argument: Unsupported identifier format. BioMCP resolves PMID (digits only, e.g., 22663011), PMCID (starts with PMC, e.g., PMC9984800), and DOI (starts with 10., e.g., 10.1056/NEJMoa1203421). publisher PIIs (e.g., S1535610826000103) are not indexed by PubMed or Europe PMC and cannot be resolved.

## Summary

Total: 2; succeeded: 0; failed: 2.

"#;

#[test]
fn failed_article_batch_report_stays_on_stdout_in_json_and_markdown() {
    for json in [false, true] {
        let output = run_compatibility(json);
        let canonical = run_canonical(json);
        assert_eq!(canonical.status, output.status);
        assert_eq!(canonical.stdout, output.stdout);
        assert_eq!(canonical.stderr, output.stderr);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            output.stderr.is_empty(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
        assert_eq!(stdout, if json { JSON_GOLDEN } else { MARKDOWN_GOLDEN });
        if json {
            let value: serde_json::Value = serde_json::from_str(&stdout).expect("batch JSON");
            assert_eq!(value["summary"]["failed"], 2);
            assert_eq!(value["items"].as_array().map(Vec::len), Some(2));
        } else {
            assert!(stdout.contains("# Batch: article (2)"));
            assert!(stdout.contains("Total: 2; succeeded: 0; failed: 2."));
        }
    }
}
