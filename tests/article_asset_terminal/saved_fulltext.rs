use super::{ArticleFixture, successful_output};
use std::io::Write as _;

#[test]
#[serial_test::serial(article_output_fixture)]
fn article_fulltext_keeps_a_successful_download_when_the_optional_locator_cannot_be_saved() {
    // Feature: article
    let fixture = ArticleFixture::start();
    // A directory cannot be opened as the cache database. HTTP caching and the
    // conversion body cache already treat this local failure as best effort.
    std::fs::create_dir_all(fixture.root.join("cache/biomcp.db")).unwrap();
    let output = successful_output(
        &fixture,
        &["--json", "get", "article", "22663012", "fulltext"],
    );
    let article: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let text = std::fs::read_to_string(article["full_text_path"].as_str().unwrap()).unwrap();
    assert!(!text.trim().is_empty());
}

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

    let mut identities = vec!["22663012".to_owned(), "00022663012".to_owned()];
    for key in ["pmcid", "doi"] {
        if let Some(id) = initial[key].as_str() {
            identities.push(id.to_owned());
            if key == "pmcid" {
                identities.push(format!("PMCID: {id}"));
            }
        }
    }
    for (id, range) in identities
        .into_iter()
        .flat_map(|id| ["1:2", "3:4"].map(|range| (id.clone(), range)))
    {
        assert_eq!(std::fs::read_to_string(path).unwrap(), text);
        let output = successful_output(
            &fixture,
            &[
                "--json", "get", "article", &id, "fulltext", "--lines", range,
            ],
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

    let outline = successful_output(
        &fixture,
        &[
            "--json",
            "get",
            "article",
            "22663012",
            "fulltext",
            "--outline",
        ],
    );
    let outline: serde_json::Value = serde_json::from_slice(&outline.stdout).unwrap();
    assert!(outline["headings"].as_array().is_some());

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
    #[cfg(unix)]
    {
        let outside = fixture.root.join("outside.txt");
        std::fs::write(&outside, "Private external content\n").unwrap();
        std::os::unix::fs::symlink(&outside, path).unwrap();
        let output = fixture
            .biomcp_command(&[
                "--json", "get", "article", "22663012", "fulltext", "--lines", "1:2",
            ])
            .output()
            .unwrap();
        assert!(
            !output.status.success(),
            "managed views must refuse symlinks"
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("Private external content"));
    }
}
