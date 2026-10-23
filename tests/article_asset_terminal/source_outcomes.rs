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
        stderr.contains("Europe PMC supplementary request failed")
            || stderr.contains("article asset summary timed out"),
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

#[test]
#[serial_test::serial(article_output_fixture)]
fn identity_and_discovery_failures_name_the_upstream_provider() {
    let fixture = ArticleFixture::start();
    for (pmid, provider) in [
        ("99002271", "NCBI ID Converter"),
        ("99002272", "Semantic Scholar"),
    ] {
        let output = fixture
            .biomcp_command(&["--json", "get", "article", pmid, "assets"])
            .output()
            .expect("asset lookup should run");
        assert!(!output.status.success(), "{output:?}");
        let answer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        let message = answer["error"]["message"].as_str().unwrap();
        assert!(message.contains(provider), "{message}");
        assert!(!message.contains("Figshare"), "{message}");
        assert!(
            !message.contains("no open-access asset package"),
            "{message}"
        );
        assert!(!message.contains("authored"), "{message}");
    }
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn malformed_asset_xml_names_only_the_provider_that_failed() {
    let fixture = ArticleFixture::start();
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "99002273", "assets"])
        .output()
        .expect("asset lookup should run");
    assert!(!output.status.success(), "{output:?}");
    let answer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let message = answer["error"]["message"].as_str().unwrap();
    assert!(
        message.starts_with("Article asset sources unavailable: Europe PMC could not confirm"),
        "{message}"
    );
    assert!(!message.contains("NCBI EFetch"), "{message}");
    assert!(!message.contains("Figshare"), "{message}");
}

#[test]
#[serial_test::serial(article_output_fixture)]
fn empty_supplement_container_scopes_the_listing_and_names_the_article_route() {
    let fixture = ArticleFixture::start();
    let output = fixture
        .biomcp_command(&["--json", "get", "article", "22663014", "assets"])
        .output()
        .expect("asset lookup should run");
    assert!(!output.status.success(), "{output:?}");
    let answer: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let message = answer["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("Europe PMC listed no supplements (source_listed_none)"),
        "{message}"
    );
    assert!(
        message.contains("does not establish that none exist"),
        "{message}"
    );
    assert!(
        !message.contains("Europe PMC could not confirm"),
        "{message}"
    );
    assert!(
        message.contains("https://pmc.ncbi.nlm.nih.gov/articles/PMC123459/"),
        "{message}"
    );

    let fulltext = fixture
        .biomcp_command(&["--json", "get", "article", "22663012", "fulltext"])
        .output()
        .expect("fulltext lookup should run");
    assert!(fulltext.status.success(), "{fulltext:?}");
    let article: serde_json::Value = serde_json::from_slice(&fulltext.stdout).unwrap();
    assert!(article["not_included"].is_null(), "{article}");

    // A populated package still exposes its assets and normal data observation.
    let populated = fixture
        .biomcp_command(&["--json", "get", "article", "22663018", "assets"])
        .output()
        .expect("populated asset lookup should run");
    assert!(populated.status.success(), "{populated:?}");
    let manifest: serde_json::Value = serde_json::from_slice(&populated.stdout).unwrap();
    assert!(!manifest["assets"].as_array().unwrap().is_empty());
    assert!(
        manifest["source_attempts"]
            .as_array()
            .unwrap()
            .iter()
            .any(|attempt| {
                attempt["provider"]["source"] == "Europe PMC"
                    && attempt["source_document"] == "europe_pmc_zip"
                    && attempt["outcome"] == "data"
            }),
        "{manifest}"
    );
}
