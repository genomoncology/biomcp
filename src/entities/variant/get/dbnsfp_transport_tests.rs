//! One complete detail card across callable, CLI and MCP channels.
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

fn row() -> Value {
    json!({"_id":"chr7:g.140453136A>T","clinvar":{"gene":{"symbol":"BRAF"}},
        "cadd":{"phred":32.0},"dbnsfp":{"genename":"BRAF",
            "revel":{"score":[0.94,0.11]},"sift":{"score":0.01,"pred":"D"},
            "phylop":{"100way_vertebrate":{"rankscore":0.92}},
            "bayesdel":{"add_af":{"score":0.399079,"pred":"D"},
                "no_af":{"score":0.335473,"pred":"D"}}}})
}

fn card(present: bool, revel: f64) -> Value {
    let mut expected = json!({"id":"chr7:g.140453136A>T","gene":"BRAF","cadd_score":32.0,
        "genome_build":"GRCh38",
        "section_outcomes":{
            "cancerhotspots":{"outcome":"not_requested","sources":[]},
            "cbioportal":{"outcome":"not_requested","sources":[]},
            "civic":{"outcome":"not_requested","sources":[]},
            "clinvar":{"outcome":"not_requested","sources":[]},
            "gwas":{"outcome":"not_requested","sources":[]},
            "population":{"outcome":"not_requested","sources":[]},
            "predict":{"outcome":"not_requested","sources":[]}}});
    if present {
        expected["sift_pred"] = json!("Deleterious");
        expected["conservation"] = json!({"phylop_100way_vertebrate":0.92});
        expected["expanded_predictions"] = json!([
            {"tool":"REVEL","score":revel}, {"tool":"SIFT","score":0.01,"prediction":"Deleterious"},
            {"tool":"BayesDel add-AF","score":0.399079,"prediction":"D"},
            {"tool":"BayesDel no-AF","score":0.335473,"prediction":"D"}]);
    }
    expected
}

fn native_card() -> Value {
    let mut expected = card(true, 0.94);
    expected["_meta"] = json!({"evidence_urls":[],"next_commands":[
        "biomcp get gene BRAF", "biomcp search drug --target BRAF",
        "biomcp variant trials \"chr7:g.140453136A>T\"", "biomcp variant articles \"chr7:g.140453136A>T\""],
        "section_sources":[
            {"key":"identity","label":"Identity","outcome":"data","sources":["MyVariant.info","ClinVar"]},
            {"key":"conservation","label":"Conservation","outcome":"data","sources":["MyVariant.info"]},
            {"key":"expanded_predictions","label":"Expanded Predictions","outcome":"data","sources":["MyVariant.info"]}]});
    expected
}

fn ledger(requests: &Arc<Mutex<Vec<String>>>) {
    let log = requests.lock().unwrap();
    assert_eq!(log.len(), 1, "{log:?}");
    let words: Vec<_> = log[0].lines().next().unwrap().split_whitespace().collect();
    assert_eq!((words[0], words[2]), ("GET", "HTTP/1.1"));
    let url = reqwest::Url::parse(&format!("http://localhost{}", words[1])).unwrap();
    assert_eq!(url.path(), "/variant/chr7:g.140453136A%3ET");
    let literal = include_str!("../resolution/point_oracles/GET_FIELDS.txt").trim_end();
    assert_eq!(
        url.query_pairs().into_owned().collect::<Vec<_>>(),
        [
            ("fields".into(), literal.into()),
            ("assembly".into(), "hg38".into())
        ]
    );
    assert_eq!(log[0].split_once("\r\n\r\n").unwrap().1, "");
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn dbnsfp_complete_detail_card_callable_cli_typed_and_raw() {
    let response = Arc::new(Mutex::new(row()));
    let served = response.clone();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let captured = requests.clone();
    let fixture = TestHttpFixture::spawn(move |request| {
        captured.lock().unwrap().push(request.to_owned());
        TestHttpReply::Bytes(test_http_response(
            "200 OK",
            "application/json",
            &serde_json::to_vec(&*served.lock().unwrap()).unwrap(),
        ))
    })
    .await;
    let cache = tempfile::tempdir().unwrap();
    let envs = [
        ("BIOMCP_MYVARIANT_BASE", fixture.base.clone()),
        ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
        ("BIOMCP_CACHE_MODE", "off".into()),
        ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
        ("RUST_LOG", "off,reqwest_retry=error".into()),
        ("ONCOKB_TOKEN", "".into()),
    ];
    let mut env = TestEnv::new();
    for (key, value) in &envs {
        env.set(key, value);
    }
    let sections = ["predictions".into(), "conservation".into()];
    for (present, score) in [(true, 0.94), (true, 0.73), (false, 0.94)] {
        let mut input = row();
        if present {
            input["dbnsfp"]["revel"]["score"] = json!([score, 0.11]);
        } else {
            input.as_object_mut().unwrap().remove("dbnsfp");
        }
        *response.lock().unwrap() = input;
        requests.lock().unwrap().clear();
        let actual = super::get("chr7:g.140453136A>T", &sections).await.unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(), card(present, score));
        ledger(&requests);
    }
    *response.lock().unwrap() = row();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let harness = ContractHarness::new(std::env::var_os("BIOMCP_BIN").unwrap(), root);
    requests.lock().unwrap().clear();
    let output = tokio::process::Command::new(&harness.biomcp_bin)
        .args([
            "get",
            "variant",
            "chr7:g.140453136A>T",
            "predictions",
            "conservation",
            "--json",
        ])
        .envs(envs.iter().cloned())
        .output()
        .await
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        native_card()
    );
    ledger(&requests);
    let client = harness.spawn_stdio_client(&envs).await.unwrap();
    for (tool, args) in [
        (
            "get",
            json!({"entity":"variant","id":"chr7:g.140453136A>T","sections":["predictions","conservation"],"json":true}),
        ),
        (
            "biomcp",
            json!({"json":true,"command":"biomcp get variant 'chr7:g.140453136A>T' predictions conservation --json"}),
        ),
    ] {
        requests.lock().unwrap().clear();
        let result = client
            .peer()
            .call_tool(
                CallToolRequestParams::new(tool).with_arguments(args.as_object().unwrap().clone()),
            )
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(first_text(&result.content)).unwrap(),
            native_card()
        );
        let mut actual = serde_json::to_value(result).unwrap();
        actual["content"][0]["text"] = native_card();
        assert_eq!(
            actual,
            json!({"content":[{"type":"text","text":native_card()}],"isError":false})
        );
        ledger(&requests);
    }
    client.cancel().await.unwrap();
}
