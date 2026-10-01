//! Compact adopted-path controls. All payloads are original synthetic MIT source.
use crate::entities::article::test_support::{TestHttpFixture, TestHttpReply, test_http_response};
use crate::sources::mydisease::projection::{decode_get, decode_search};
use biomcp_mcp_contract_client::{ContractHarness, call_biomcp_json, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

mod boundaries;
#[path = "../../../tests/support/disease_identity_cases.rs"]
pub(crate) mod controls;

fn public(text: &str, error: bool, get: bool, name: &str) {
    assert!(!text.contains("SOURCE-ONLY-CANARY"), "{text}");
    if error {
        // Existing MCP failure envelope uses safe text; CLI errors are checked separately.
        assert!(text.starts_with("Error:"), "{text}");
        assert!(text.contains("MyDisease"), "{text}");
        return;
    }
    let value: Value = serde_json::from_str(text).expect("public JSON");
    if get {
        assert_eq!(value["id"], "MONDO:1");
        assert_eq!(value["name"], name);
        assert!(value.get("row").is_none());
        assert!(value.get("identity").is_none());
    } else {
        assert_eq!(value["results"][0]["id"], "MONDO:1");
        assert_eq!(value["results"][0]["name"], name);
        assert_eq!(value["pagination"]["total"], 1);
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn cli_raw_typed_disease_identity_table() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root);
    for case in controls::cases() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let get = case.get.clone();
        let search = case.search.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured
                .lock()
                .unwrap()
                .push(request.lines().next().unwrap().to_owned());
            let body = if request.starts_with("GET /query?") {
                search.as_slice()
            } else if request.starts_with("GET /disease/") {
                get.as_slice()
            } else {
                br#"{"response":{"numFound":0,"docs":[]},"data":{}}"#.as_slice()
            };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body))
        })
        .await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYDISEASE_BASE", fixture.base.clone()),
            ("BIOMCP_OLS4_BASE", fixture.base.clone()),
            ("BIOMCP_CIVIC_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
        ];
        for (get, args) in [
            (
                true,
                vec!["--json", "--no-cache", "get", "disease", "MONDO:1", "civic"],
            ),
            (
                false,
                vec!["--json", "--no-cache", "search", "disease", "-q", "MONDO:1"],
            ),
        ] {
            let before = requests.lock().unwrap().len();
            let output = tokio::time::timeout(
                std::time::Duration::from_secs(15),
                tokio::process::Command::new(&harness.biomcp_bin)
                    .args(args)
                    .envs(env.iter().cloned())
                    .kill_on_drop(true)
                    .output(),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(
                output.status.success(),
                !case.error,
                "{}: {}",
                case.label,
                String::from_utf8_lossy(&output.stdout)
            );
            assert!(
                output.stderr.is_empty(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let text = String::from_utf8(output.stdout).unwrap();
            if case.error {
                let value: Value = serde_json::from_str(&text).unwrap();
                assert_eq!(value["error"]["code"], "api");
                assert!(!text.contains("SOURCE-ONLY-CANARY"));
            } else {
                public(&text, false, get, case.name);
            }
            if case.error {
                assert_eq!(
                    requests.lock().unwrap().len() - before,
                    1,
                    "terminal response rescued"
                );
            }
        }
        let client = harness.spawn_stdio_client(&env).await.unwrap();
        for (get, tool, command, arguments) in [
            (
                true,
                "get",
                "biomcp --no-cache get disease MONDO:1 civic",
                json!({"entity":"disease","id":"MONDO:1","sections":["civic"],"json":true}),
            ),
            (
                false,
                "search",
                "biomcp --no-cache search disease -q MONDO:1",
                json!({"entity":"disease","query":"MONDO:1","json":true}),
            ),
        ] {
            let raw = call_biomcp_json(&client, command).await.unwrap();
            let typed = client
                .peer()
                .call_tool(
                    CallToolRequestParams::new(tool)
                        .with_arguments(arguments.as_object().unwrap().clone()),
                )
                .await
                .unwrap();
            for result in [raw, typed] {
                assert_eq!(
                    result.is_error,
                    Some(case.error),
                    "{}: {}",
                    case.label,
                    first_text(&result.content)
                );
                public(first_text(&result.content), case.error, get, case.name);
                // Existing MCP envelope carries structured JSON in its text content.
                assert!(result.structured_content.is_none());
            }
        }
        if case.label == "DO priority" {
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(["--no-cache", "get", "disease", "MONDO:1", "civic"])
                .envs(env.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success());
            let text = String::from_utf8_lossy(&output.stdout);
            assert!(text.contains("Synthetic tumor"));
            assert!(text.contains("MONDO:1"));
        }
        client.cancel().await.unwrap();
    }
    boundaries::nested_refusal_table(&harness).await;
    boundaries::trial_and_diagnostic_table(&harness).await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn disease_search_identity_policy_table() {
    use crate::entities::article::test_support::TestEnv;
    let captured = Arc::new(Mutex::new(Vec::new()));
    let requests = Arc::clone(&captured);
    let fixture = TestHttpFixture::spawn(move |request| {
        requests.lock().unwrap().push(request.lines().next().unwrap().to_owned());
        let body = if request.contains("carcinoma") {
            br#"{"total":9,"hits":[{"_id":"MONDO:2","mondo":{"name":"other"}},{"_id":"MONDO:3","mondo":{"name":"Synthetic cancer"}}]}"#.as_slice()
        } else {
            br#"{"total":4,"hits":[{"_id":"MONDO:1","mondo":{"name":"Synthetic cancer"}},{"_id":"MONDO:2","mondo":{"name":"Synthetic cancer subtype"}}]}"#.as_slice()
        };
        TestHttpReply::Bytes(test_http_response("200 OK", "application/json", body))
    }).await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    env.set("BIOMCP_MYDISEASE_BASE", &fixture.base);
    env.set("BIOMCP_TEST_UNPACED_ORIGIN", &fixture.base);
    env.set("BIOMCP_CACHE_DIR", cache.path());
    let page = super::search_page(
        &super::DiseaseSearchFilters {
            query: Some("Synthetic cancer".into()),
            ..Default::default()
        },
        1,
        1,
    )
    .await
    .unwrap();
    assert_eq!(page.total, Some(9));
    assert_eq!(page.results[0].id, "MONDO:3");
    assert_eq!(captured.lock().unwrap().len(), 2);
    // First row custody remains before product ranking, including first-seen duplicate ID.
    let first = decode_search(br#"{"total":3,"hits":[{"_id":"MONDO:2","mondo":{"name":"first"}},{"_id":"MONDO:1","mondo":{"name":"Synthetic cancer"}}]}"#).unwrap();
    let second =
        decode_search(br#"{"total":8,"hits":[{"_id":"MONDO:2","mondo":{"name":"replacement"}}]}"#)
            .unwrap();
    let ranked = super::resolution::rerank_disease_search_hits(
        "Synthetic cancer",
        vec![(0, first.hits), (1, second.hits)],
    );
    assert_eq!(ranked[0].id, "MONDO:1");
    assert_eq!(
        crate::transform::disease::name_from_mydisease_hit(&ranked[1]),
        "first"
    );
    assert_eq!(ranked[1].row.source().ordinal(), 0);
}

#[test]
fn disease_identity_document_loss_table() {
    use biodata::MyDiseaseField;
    for case in controls::cases().into_iter().filter(|case| !case.error) {
        let hit = decode_get(&case.get).unwrap();
        let document = biodata::Document::DiseaseIdentity(Box::new(hit.row.identity().clone()));
        assert_eq!(
            biodata::Document::from_json(&document.to_json().unwrap()).unwrap(),
            document
        );
        assert!(hit.row.identity().qualified_name().is_none());
        assert!(!hit.row.report().is_empty());
        assert!(!hit.conversion.losses.is_empty() || case.label == "MONDO");
        match case.label {
            "missing" => assert_eq!(hit.row.source().mondo_name(), &MyDiseaseField::Missing),
            "null" => assert_eq!(hit.row.source().mondo_name(), &MyDiseaseField::Null),
            "blank" => assert!(matches!(
                hit.row.source().mondo_name(),
                MyDiseaseField::Blank(_)
            )),
            "DO priority" | "equal" => {
                assert_eq!(hit.row.identity().names().len(), 2);
                assert_eq!(
                    hit.row.identity().names()[0].section(),
                    biodata::DiseaseNameSection::Mondo
                );
            }
            _ => {}
        }
    }
    let hit = decode_get(br#"{"_id":"MONDO:1","mondo":{"name":"Synthetic tumor","definition":"A definition","parents":["MONDO:2"],"synonym":["alias"],"xrefs":{"omim":["OMIM:123"]}},"disgenet":{"genes_related_to_disease":[{"gene_symbol":"SYNTH","score":1}]},"hpo":{"phenotype_related_to_disease":{"hpo_id":"HP:1"}}}"#).unwrap();
    let disease = crate::transform::disease::from_mydisease_hit(hit);
    assert_eq!(disease.definition.as_deref(), Some("A definition"));
    assert_eq!(disease.synonyms, ["alias"]);
    assert_eq!(disease.parents, ["MONDO:2"]);
    assert_eq!(disease.xrefs["OMIM"], "123");
    assert_eq!(disease.associated_genes, ["SYNTH"]);
    assert_eq!(disease.phenotypes[0].hpo_id, "HP:1");
}
