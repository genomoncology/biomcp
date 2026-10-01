//! Nested and downstream refusal witnesses run the shipped command path.
use super::*;
use std::io::Write;

async fn command(
    harness: &ContractHarness,
    args: &[&str],
    env: &[(&str, String)],
    error: bool,
) -> Value {
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(15),
        tokio::process::Command::new(&harness.biomcp_bin)
            .args(args)
            .env_remove("UMLS_API_KEY")
            .envs(env.iter().cloned())
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        output.status.success(),
        !error,
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("SOURCE-ONLY-CANARY"));
    let value: Value = serde_json::from_str(&text).unwrap();
    if error {
        assert!(value["error"]["code"].is_string());
    }
    value
}
fn environment(base: &str, cache: &std::path::Path) -> Vec<(&'static str, String)> {
    vec![
        ("BIOMCP_MYDISEASE_BASE", base.into()),
        ("BIOMCP_OLS4_BASE", base.into()),
        ("BIOMCP_CIVIC_BASE", base.into()),
        ("BIOMCP_UMLS_BASE", base.into()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", base.into()),
        ("BIOMCP_CACHE_DIR", cache.display().to_string()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
    ]
}

pub(super) async fn nested_refusal_table(harness: &ContractHarness) {
    // The selected root is valid. Parent rejection must stop the card before civic.
    for (parent_status, parent_body, query_body, error, expected_requests, expected_parent) in [
        (
            "200 OK",
            r#"{"_id":"MONDO:2","mondo":{"name":17},"canary":"SOURCE-ONLY-CANARY"}"#,
            "",
            true,
            2,
            "",
        ),
        (
            "200 OK",
            r#"{"_id":"MONDO:2"}"#,
            r#"{"total":1,"hits":[{"_id":"MONDO:2","mondo":{"name":false}}]}"#,
            true,
            3,
            "",
        ),
        (
            "404 Not Found",
            "{}",
            r#"{"total":1,"hits":[{"_id":" "}]}"#,
            true,
            3,
            "",
        ),
        (
            "200 OK",
            r#"{"_id":"MONDO:2","mondo":{"name":"Parent tumor"}}"#,
            "",
            false,
            3,
            "Parent tumor (MONDO:2)",
        ),
        (
            "404 Not Found",
            "{}",
            r#"{"total":0,"hits":[]}"#,
            false,
            4,
            "MONDO:2",
        ),
    ] {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            captured
                .lock()
                .unwrap()
                .push(request.lines().next().unwrap().to_owned());
            let (status, body) = if request.starts_with("GET /disease/MONDO:1?") {
                (
                    "200 OK",
                    r#"{"_id":"MONDO:1","mondo":{"name":"Root tumor","parents":["MONDO:2"]}}"#,
                )
            } else if request.starts_with("GET /disease/MONDO:2?") {
                (parent_status, parent_body)
            } else if request.starts_with("GET /query?") {
                ("200 OK", query_body)
            } else {
                ("200 OK", r#"{"data":{}}"#)
            };
            TestHttpReply::Bytes(test_http_response(
                status,
                "application/json",
                body.as_bytes(),
            ))
        })
        .await;
        let cache = tempfile::tempdir().unwrap();
        let env = environment(&fixture.base, cache.path());
        let value = command(
            harness,
            &["--json", "--no-cache", "get", "disease", "MONDO:1", "civic"],
            &env,
            error,
        )
        .await;
        if !error {
            assert_eq!(value["parents"][0], expected_parent);
        }
        let observed = requests.lock().unwrap().clone();
        assert_eq!(observed.len(), expected_requests, "{observed:?}");
        if error {
            assert!(
                observed.iter().all(
                    |line| line.starts_with("GET /disease/") || line.starts_with("GET /query?")
                )
            );
        }
        // Raw and typed MCP inherit the same terminal parent disposition.
        if error {
            let client = harness.spawn_stdio_client(&env).await.unwrap();
            for typed in [false, true] {
                let before = requests.lock().unwrap().len();
                let result = if typed {
                    client.peer().call_tool(CallToolRequestParams::new("get")
                        .with_arguments(json!({"entity":"disease","id":"MONDO:1","sections":["civic"],"json":true}).as_object().unwrap().clone())).await.unwrap()
                } else {
                    call_biomcp_json(&client, "biomcp --no-cache get disease MONDO:1 civic")
                        .await
                        .unwrap()
                };
                assert_eq!(result.is_error, Some(true));
                assert!(!first_text(&result.content).contains("SOURCE-ONLY-CANARY"));
                assert_eq!(requests.lock().unwrap().len() - before, expected_requests);
            }
            client.cancel().await.unwrap();
        }
    }
    fallback_refusal(harness).await;
}

async fn fallback_refusal(harness: &ContractHarness) {
    // Original local OLS and MyDisease bytes exercise both nested catches.
    for (crosswalk, reject_detail, error) in [
        (false, false, true),
        (true, false, true),
        (false, true, true),
        (false, false, false),
    ] {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            let mut log = captured.lock().unwrap();
            log.push(request.lines().next().unwrap().to_owned());
            let gets = log.iter().filter(|line| line.starts_with("GET /disease/")).count();
            let query = request.starts_with("GET /query?");
            let xref_query = query && (request.contains("xrefs") || request.contains("umls.mesh"));
            let body = if xref_query && crosswalk {
                r#"{"total":2,"hits":[{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor"}},{"_id":" ","canary":"SOURCE-ONLY-CANARY"}]}"#
            } else if query { r#"{"total":0,"hits":[]}"# }
            else if request.starts_with("GET /api/search?") {
                if crosswalk {
                    r#"{"response":{"docs":[{"iri":"https://example.invalid/MESH_1","obo_id":"MESH:D000001","ontology_prefix":"mesh","label":"Synthetic disease tumor","type":"class"}]}}"#
                } else {
                    r#"{"response":{"docs":[{"iri":"https://example.invalid/MONDO_1","obo_id":"MONDO:1","ontology_prefix":"mondo","label":"Synthetic tumor","type":"class"}]}}"#
                }
            } else if request.starts_with("GET /disease/") {
                if error && (!reject_detail || gets == 2) {
                    r#"{"_id":"MONDO:1","mondo":{"name":false},"canary":"SOURCE-ONLY-CANARY"}"#
                } else { r#"{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor"}}"# }
            } else { r#"{"data":{}}"# };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body.as_bytes()))
        }).await;
        let cache = tempfile::tempdir().unwrap();
        let env = environment(&fixture.base, cache.path());
        let value = command(
            harness,
            &[
                "--json",
                "--no-cache",
                "get",
                "disease",
                "Synthetic tumor",
                "civic",
            ],
            &env,
            error,
        )
        .await;
        let observed = requests.lock().unwrap().clone();
        if error {
            assert_eq!(
                value["error"]["code"], "api",
                "crosswalk={crosswalk} detail={reject_detail}: {observed:?} {value}"
            );
            assert_eq!(
                observed
                    .iter()
                    .filter(|line| line.starts_with("GET /disease/"))
                    .count(),
                if crosswalk {
                    0
                } else if reject_detail {
                    2
                } else {
                    1
                }
            );
            assert_eq!(
                observed
                    .iter()
                    .filter(|line| line.starts_with("GET /api/search?"))
                    .count(),
                1,
                "no discover rescue after rejection: {observed:?}"
            );
            assert_eq!(
                observed
                    .iter()
                    .filter(|line| line.starts_with("GET /query?"))
                    .count(),
                if crosswalk { 2 } else { 1 }
            );
            assert!(!observed.iter().any(|line| line.starts_with("POST /")));
        } else {
            assert_eq!(value["name"], "Synthetic tumor");
        }
    }
}

pub(super) async fn trial_and_diagnostic_table(harness: &ContractHarness) {
    trial_grounding(harness).await;
    diagnostic_resolution(harness).await;
}
async fn trial_grounding(harness: &ContractHarness) {
    for mode in [
        "concept",
        "keyword",
        "not found",
        "transport",
        "rejected",
        "nonmatching rejected",
    ] {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.lines().next().unwrap().to_owned());
            let body = if request.starts_with("GET /query?") {
                match mode {
                    "concept" => r#"{"total":1,"hits":[{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor","xrefs":{"ncit":["C100"]}}}]}"#,
                    "keyword" => r#"{"total":1,"hits":[{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor"}}]}"#,
                    "rejected" => r#"{"total":1,"hits":[{"_id":" ","canary":"SOURCE-ONLY-CANARY"}]}"#,
                    "nonmatching rejected" => r#"{"total":2,"hits":[{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor"}},{"_id":"MONDO:2","mondo":{"name":17}}]}"#,
                    _ => r#"{"total":0,"hits":[]}"#,
                }
            } else if request.starts_with("GET /api/search?") { r#"{"response":{"docs":[]}}"# }
            else { r#"{"total":0,"trials":[]}"# };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body.as_bytes()))
        }).await;
        let cache = tempfile::tempdir().unwrap();
        let mut env = environment(&fixture.base, cache.path());
        if mode == "transport" {
            env[0].1 = "http://127.0.0.1:0".into();
        }
        // Authored fixture credential replaces any inherited credential; no external API.
        env.extend([
            ("NCI_API_KEY", "synthetic-fixture-key".into()),
            ("BIOMCP_NCI_CTS_BASE", fixture.base.clone()),
        ]);
        let error = mode.contains("rejected");
        command(
            harness,
            &[
                "--json",
                "--no-cache",
                "search",
                "trial",
                "--condition",
                "Synthetic tumor",
                "--source",
                "nci",
            ],
            &env,
            error,
        )
        .await;
        let observed = requests.lock().unwrap().clone();
        let downstream = observed
            .iter()
            .filter(|line| line.starts_with("GET /trials?"))
            .collect::<Vec<_>>();
        assert_eq!(
            downstream.len(),
            usize::from(!error),
            "{mode}: {observed:?}"
        );
        if !error {
            assert!(
                downstream[0].contains(if mode == "concept" {
                    "C100"
                } else {
                    "Synthetic"
                }),
                "{mode}: {downstream:?}"
            );
        }
    }
}

fn seed_gtr(root: &std::path::Path) {
    std::fs::create_dir_all(root).unwrap();
    let headers = "test_accession_ver\tnow_current\tlab_test_name\tmanufacturer_test_name\tname_of_laboratory\tname_of_institution\tCLIA_number\tstate_licenses\tfacility_country\ttest_currStat\ttest_pubStat\tmethod_categories\tmethods\tgenes\n";
    let rows = "GTR000000001.1\t1\trequested tumor\t\tSynthetic lab\t\t\t\t\t\t\tMolecular genetics\t\t\nGTR000000002.1\t1\tcanonical tumor\t\tSynthetic lab\t\t\t\t\t\t\tMolecular genetics\t\t\nGTR000000003.1\t1\tfirst alias\t\tSynthetic lab\t\t\t\t\t\t\tMolecular genetics\t\t\nGTR000000004.1\t1\tsecond alias\t\tSynthetic lab\t\t\t\t\t\t\tMolecular genetics\t\t\n";
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    gzip.write_all(format!("{headers}{rows}").as_bytes())
        .unwrap();
    std::fs::write(root.join("test_version.gz"), gzip.finish().unwrap()).unwrap();
    std::fs::write(root.join("test_condition_gene.txt"), "#accession_version\tobject\tobject_name\nGTR000000001.1\tcondition\trequested tumor\nGTR000000002.1\tcondition\tcanonical tumor\nGTR000000003.1\tcondition\tfirst alias\nGTR000000004.1\tcondition\tsecond alias\n").unwrap();
}
async fn diagnostic_resolution(harness: &ContractHarness) {
    for mode in [
        "unique",
        "zero",
        "multiple",
        "query rejected",
        "nonmatching rejected",
        "detail rejected",
    ] {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let fixture = TestHttpFixture::spawn(move |request| {
            captured.lock().unwrap().push(request.lines().next().unwrap().to_owned());
            let body = if request.starts_with("GET /query?") {
                match mode {
                    "zero" => r#"{"total":0,"hits":[]}"#,
                    "multiple" => r#"{"total":2,"hits":[{"_id":"MONDO:1","mondo":{"name":"requested tumor"}},{"_id":"MONDO:2","mondo":{"name":"requested tumor"}}]}"#,
                    "query rejected" => r#"{"total":1,"hits":[{"_id":" "}]}"#,
                    "nonmatching rejected" => r#"{"total":2,"hits":[{"_id":"MONDO:1","mondo":{"name":"requested tumor"}},{"_id":"MONDO:2","mondo":{"name":false}}]}"#,
                    _ => r#"{"total":1,"hits":[{"_id":"MONDO:1","mondo":{"name":"requested tumor"}}]}"#,
                }
            } else if request.starts_with("GET /disease/") {
                if mode == "detail rejected" { r#"{"_id":"MONDO:1","mondo":{"name":false},"canary":"SOURCE-ONLY-CANARY"}"# }
                else { r#"{"_id":"MONDO:1","mondo":{"name":"canonical tumor","synonym":["first alias","second alias"]}}"# }
            } else { r#"{"unexpected":"SOURCE-ONLY-CANARY"}"# };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body.as_bytes()))
        }).await;
        let state = tempfile::tempdir().unwrap();
        let gtr = state.path().join("gtr");
        let who = state.path().join("who");
        let error = mode.contains("rejected");
        if !error {
            seed_gtr(&gtr);
        }
        let mut env = environment(&fixture.base, state.path());
        env.extend([
            ("BIOMCP_GTR_DIR", gtr.display().to_string()),
            ("BIOMCP_WHO_IVD_DIR", who.display().to_string()),
            (
                "BIOMCP_GTR_TEST_VERSION_URL",
                format!("{}/gtr-download", fixture.base),
            ),
            (
                "BIOMCP_GTR_CONDITION_GENE_URL",
                format!("{}/gtr-links", fixture.base),
            ),
        ]);
        let value = command(
            harness,
            &[
                "--json",
                "search",
                "diagnostic",
                "--disease",
                "requested tumor",
                "--source",
                if error { "all" } else { "gtr" },
            ],
            &env,
            error,
        )
        .await;
        let observed = requests.lock().unwrap().clone();
        assert_eq!(
            observed.len(),
            if mode == "unique" || mode == "detail rejected" {
                2
            } else {
                1
            },
            "{mode}: {observed:?}"
        );
        assert!(!who.exists());
        if error {
            assert!(
                !gtr.exists(),
                "GTR readiness must not run after disease refusal"
            );
            assert_eq!(value["error"]["code"], "source_unavailable");
        } else {
            let rows = value["results"].as_array().unwrap();
            assert_eq!(
                rows.len(),
                if mode == "unique" { 4 } else { 1 },
                "{mode}: {value}"
            );
            assert_eq!(rows[0]["disease_match"]["kind"], "requested");
            if mode == "unique" {
                assert_eq!(rows[1]["disease_match"]["kind"], "canonical");
                assert_eq!(rows[2]["disease_match"]["term"], "first alias");
                assert_eq!(rows[3]["disease_match"]["term"], "second alias");
                assert_eq!(rows[1]["disease_match"]["resolved_id"], "MONDO:1");
            } else {
                assert!(rows[0]["disease_match"]["resolved_id"].is_null());
            }
        }
    }
}
