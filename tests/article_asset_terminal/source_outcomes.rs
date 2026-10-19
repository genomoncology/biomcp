use super::ArticleFixture;

#[test]
#[serial_test::serial(article_output_fixture)]
fn asset_provider_failure_names_the_source_and_retry_reason() {
    let fixture = ArticleFixture::start();
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663013", "assets"])
        .output()
        .expect("asset lookup should run");
    assert!(!output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let answer = format!("{stdout}\n{stderr}");
    assert!(answer.contains("Europe PMC"), "{answer}");
    assert!(
        answer.contains("PMC Open Access returned no open-access asset package"),
        "{answer}"
    );
    assert!(answer.contains("source_unavailable"), "{answer}");
    assert!(answer.contains("Retry"), "{answer}");
    assert!(!answer.contains("BioMCP source"), "{answer}");
    assert!(!answer.contains("Review source configuration"), "{answer}");
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn html_fulltext_success_keeps_expected_package_misses_quiet() {
    let fixture = ArticleFixture::start();
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663012", "fulltext"])
        .output()
        .expect("fulltext lookup should run");
    assert!(output.status.success(), "{output:?}");
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("PMC HTML"));
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn successful_pdf_fulltext_still_warns_about_a_real_asset_provider_outage() {
    let fixture = ArticleFixture::start();
    let destination = fixture.root.join("fulltext");
    std::fs::create_dir(&destination).unwrap();
    let output = fixture
        .biomcp_command(&[
            "--json",
            "get",
            "article",
            "22663013",
            "fulltext",
            "--pdf",
            "--out",
            destination.to_str().unwrap(),
        ])
        .output()
        .expect("PDF fulltext lookup should run");
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Europe PMC supplementary request failed"),
        "{stderr}"
    );
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn healthy_asset_package_misses_name_the_consulted_sources() {
    let fixture = ArticleFixture::start();
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663014", "assets"])
        .output()
        .expect("asset lookup should run");
    assert!(!output.status.success());
    let answer = String::from_utf8_lossy(&output.stdout);
    assert!(answer.contains("PMC Open Access"), "{answer}");
    assert!(answer.contains("Europe PMC"), "{answer}");
    assert!(answer.contains("no open-access asset package"), "{answer}");
    assert!(!answer.contains("BioMCP source"), "{answer}");
}
