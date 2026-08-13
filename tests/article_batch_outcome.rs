use std::process::Command;

fn run(json: bool) -> std::process::Output {
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

#[test]
fn failed_article_batch_report_stays_on_stdout_in_json_and_markdown() {
    for json in [false, true] {
        let output = run(json);
        assert_eq!(output.status.code(), Some(1));
        assert!(
            output.stderr.is_empty(),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 report");
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
