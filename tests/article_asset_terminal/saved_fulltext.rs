use super::{ArticleFixture, successful_output};
use std::io::Write as _;

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_line_reads_keep_the_returned_conversion_when_the_provider_later_has_none() {
    // Feature: article
    let fixture = ArticleFixture::start();
    let initial = successful_output(
        &fixture,
        &["--json", "get", "article", "22663012", "fulltext"],
    );
    let initial: serde_json::Value = serde_json::from_slice(&initial.stdout).unwrap();
    let path = std::path::Path::new(initial["full_text_path"].as_str().unwrap());
    let text = std::fs::read_to_string(path).unwrap();
    assert_eq!(initial["full_text_source"]["kind"], "html");
    assert!(!text.trim().is_empty());
    let mut env = std::fs::OpenOptions::new()
        .append(true)
        .open(&fixture.env_file)
        .unwrap();
    writeln!(
        env,
        "export BIOMCP_PMC_HTML_BASE=\"$BIOMCP_PMC_HTML_BASE/missing\""
    )
    .unwrap();

    let mut identities = vec!["22663012"];
    for key in ["pmcid", "doi"] {
        if let Some(id) = initial[key].as_str() {
            identities.push(id);
        }
    }
    for (id, range) in identities
        .into_iter()
        .flat_map(|id| ["1:2", "3:4"].map(|range| (id, range)))
    {
        assert_eq!(std::fs::read_to_string(path).unwrap(), text);
        let output = successful_output(
            &fixture,
            &["--json", "get", "article", id, "fulltext", "--lines", range],
        );
        let output: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let (start, end) = range.split_once(':').unwrap();
        let start = start.parse::<usize>().unwrap();
        let end = end.parse::<usize>().unwrap();
        let expected = text
            .split_inclusive('\n')
            .skip(start - 1)
            .take(end - start + 1)
            .collect::<String>();
        assert_eq!(output["text"], expected);
    }

    for bypass in ["--no-cache", "environment"] {
        let mut args = vec![
            "--json", "get", "article", "22663012", "fulltext", "--lines", "1:2",
        ];
        if bypass == "--no-cache" {
            args.insert(0, bypass);
        }
        let mut command = fixture.biomcp_command(&args);
        if bypass == "environment" {
            command.env("BIOMCP_CACHE_MODE", "off");
        }
        let output = command.output().unwrap();
        assert!(
            !output.status.success(),
            "explicit bypass must consult providers"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("\"text\""));
    }

    for invalid in [b"".as_slice(), b"\xff"] {
        std::fs::write(path, invalid).unwrap();
        let output = fixture
            .biomcp_command(&[
                "--json", "get", "article", "22663012", "fulltext", "--lines", "1:2",
            ])
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "unusable local content must not become a successful read"
        );
    }
    std::fs::remove_file(path).unwrap();
    let output = fixture
        .biomcp_command(&[
            "--json", "get", "article", "22663012", "fulltext", "--lines", "1:2",
        ])
        .output()
        .unwrap();
    assert!(
        !output.status.success(),
        "missing conversion must not become a successful read"
    );
}
