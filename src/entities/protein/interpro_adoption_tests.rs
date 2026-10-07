//! Public protein callers retain domain caps, omission and failure outcomes.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub(crate) const PRIVATE: &[u8] = br#"{"results":[{"metadata":{"accession":"IPR1"},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":"private-interpro-marker","end":717}]}]}]}]}"#;

pub(crate) fn body(cap: usize) -> Vec<u8> {
    let mut text = String::from(
        r#"{"ignored":1e400,"results":[{"metadata":{"accession":" "}},{"metadata":{"accession":" IPR000719 ","name":" Protein kinase ","type":" domain "},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":590},{"start":457,"end":717},{"start":590,"end":610}]}]}]},{"metadata":{"accession":"IPR000719","name":" ","type":null},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":600,"end":600}]}]}]},{"metadata":{"accession":"IPR_NO_OVERLAP"},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":1,"end":599}]}]}]}"#,
    );
    for _ in 4..cap {
        text.push_str(",{}");
    }
    text.push_str(r#",{"metadata":{"accession":"IPR_OVER_CAP"},"proteins":[{"entry_protein_locations":[{"fragments":[{"start":600,"end":600}]}]}]}]}"#);
    text.into_bytes()
}

pub(crate) async fn fixture(
    body: Vec<u8>,
    status: &'static str,
    no_change: bool,
) -> (TestHttpFixture, Arc<Mutex<Vec<String>>>) {
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        let target = request.lines().next().unwrap().split_whitespace().nth(1).unwrap();
        if target.starts_with("/entry/interpro/") {
            return TestHttpReply::Bytes(test_http_response(status, "application/json", &body));
        }
        let hit = if no_change { json!({"_id":"chr7:g.140453136A>T","dbnsfp":{"genename":"BRAF"}}) }
        else { json!({"_id":"chr7:g.140453136A>T","dbnsfp":{"genename":"BRAF","hgvsp":"p.Val600Glu"},"snpeff":{"ann":{"genename":"BRAF","feature_id":"NM_004333.6","hgvs_c":"c.1802T>A","hgvs_p":"p.Val601Glu"}}}) };
        let value = if target.starts_with("/variant/") { hit }
        else if target.starts_with("/query") && target.contains("dbnsfp") { json!({"total":1,"hits":[hit]}) }
        else if target.starts_with("/query") { json!({"total":1,"hits":[{"_id":"673","symbol":"BRAF","uniprot":{"Swiss-Prot":"P15056"}}]}) }
        else if target.starts_with("/uniprotkb/") { json!({"primaryAccession":"P15056","uniProtkbId":"BRAF_HUMAN","proteinDescription":{"recommendedName":{"fullName":{"value":"BRAF protein"}}},"sequence":{"length":766},"uniProtKBCrossReferences":[{"database":"PDB","id":"1ABC","properties":[{"key":"Chains","value":"A=1-766"}]}]}) }
        else { json!([]) };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", &serde_json::to_vec(&value).unwrap()))
    }).await;
    (fixture, requests)
}

pub(crate) fn environments(base: &str, cache: &std::path::Path) -> Vec<(&'static str, String)> {
    let mut envs: Vec<_> = [
        "BIOMCP_MYVARIANT_BASE",
        "BIOMCP_MYGENE_BASE",
        "BIOMCP_UNIPROT_BASE",
        "BIOMCP_INTERPRO_BASE",
        "BIOMCP_CANCERHOTSPOTS_BASE",
        "BIOMCP_TEST_UNPACED_ORIGIN",
    ]
    .into_iter()
    .map(|key| (key, base.to_owned()))
    .collect();
    envs.extend([
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.display().to_string()),
        ("RUST_LOG", "warn,reqwest_retry=error".into()),
    ]);
    envs
}

pub(crate) fn requests(requests: &Arc<Mutex<Vec<String>>>, cap: usize, requested: bool) {
    let requests = requests.lock().unwrap();
    let hits: Vec<_> = requests
        .iter()
        .filter(|r| r.contains("/entry/interpro/"))
        .collect();
    if !requested {
        assert!(hits.is_empty(), "{requests:?}");
        return;
    }
    assert!(!hits.is_empty(), "{requests:?}");
    for request in hits {
        assert_eq!(
            request.lines().next().unwrap(),
            format!("GET /entry/interpro/protein/uniprot/P15056/?page_size={cap} HTTP/1.1")
        );
    }
}

fn check(text: &str, json_mode: bool, state: &str) {
    assert!(!text.contains("private-interpro-marker"), "{text}");
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["accession"], "P15056");
        assert_eq!(card["section_outcomes"]["domains"]["outcome"], state);
        assert_eq!(
            card["section_outcomes"]["domains"]["sources"],
            if state == "unavailable" {
                json!([])
            } else {
                json!(["InterPro"])
            }
        );
        if state == "data" {
            assert_eq!(
                card["domains"],
                json!([{"accession":"IPR000719","name":"Protein kinase","domain_type":"domain"},{"accession":"IPR000719"},{"accession":"IPR_NO_OVERLAP"}])
            );
        } else {
            assert_eq!(card["domains"], json!([]));
        }
    } else {
        assert!(text.contains("P15056"), "{text}");
        if state == "data" {
            assert!(text.contains("Protein kinase"), "{text}");
            assert!(!text.contains("IPR_OVER_CAP"), "{text}");
        }
        if state == "unavailable" {
            assert!(text.contains("unavailable"), "{text}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn interpro_protein_reaches_native_cli_and_typed_raw_mcp() {
    for (body, status, state) in [
        (body(20), "200 OK", "data"),
        (
            b"unavailable".to_vec(),
            "503 Service Unavailable",
            "unavailable",
        ),
    ] {
        let (fixture, captured) = fixture(body, status, false).await;
        let cache = tempfile::tempdir().unwrap();
        let envs = environments(&fixture.base, cache.path());
        let mut env = TestEnv::new();
        for (key, value) in &envs {
            env.set(key, value);
        }
        let native = get("P15056", &["domains".into()]).await.unwrap();
        let restored: Protein =
            serde_json::from_value(serde_json::to_value(&native).unwrap()).unwrap();
        check(&serde_json::to_string(&restored).unwrap(), true, state);
        check(
            &crate::render::markdown::protein_markdown(&restored, &["domains".into()]).unwrap(),
            false,
            state,
        );
        let harness = ContractHarness::new(
            std::env::var_os("BIOMCP_BIN").unwrap(),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        );
        let client = harness.spawn_stdio_client(&envs).await.unwrap();
        for json_mode in [true, false] {
            let mut args = vec!["get", "protein", "P15056", "domains"];
            if json_mode {
                args.push("--json");
            }
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(args)
                .envs(envs.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success(), "{output:?}");
            check(
                std::str::from_utf8(&output.stdout).unwrap(),
                json_mode,
                state,
            );
            for (tool, args) in [
                (
                    "get",
                    json!({"entity":"protein","id":"P15056","sections":["domains"],"json":json_mode}),
                ),
                (
                    "biomcp",
                    json!({"command":"biomcp get protein P15056 domains","json":json_mode}),
                ),
            ] {
                let result = client
                    .peer()
                    .call_tool(
                        CallToolRequestParams::new(tool)
                            .with_arguments(args.as_object().unwrap().clone()),
                    )
                    .await
                    .unwrap();
                assert!(!result.is_error.unwrap_or_default(), "{result:?}");
                check(first_text(&result.content), json_mode, state);
            }
        }
        client.cancel().await.unwrap();
        requests(&captured, 20, true);
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn interpro_original_byte_errors_are_private() {
    let (fixture, _) = fixture(PRIVATE.to_vec(), "200 OK", false).await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    for (key, value) in environments(&fixture.base, cache.path()) {
        env.set(key, value);
    }
    let error = InterProClient::new()
        .unwrap()
        .domains("P15056", 20)
        .await
        .unwrap_err();
    assert!(
        !format!(
            "{error} {error:?} {}",
            crate::render::json::to_error_json(&error).unwrap()
        )
        .contains("private-interpro-marker")
    );
}
