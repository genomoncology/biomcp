use crate::entities::article::test_support::{TestHttpFixture, TestHttpReply, test_http_response};
use crate::sources::mygene::{decode_get, decode_search};
use biomcp_mcp_contract_client::{ContractHarness, call_biomcp_json, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[path = "../../../tests/support/gene_identity_cases.rs"]
mod controls;

fn assert_public(text: &str, error: bool, label: &str, get: bool, display: Option<&str>) {
    assert!(!text.contains("SOURCE-ONLY-CANARY"), "{label}: {text}");
    if error {
        return;
    }
    let value: Value = serde_json::from_str(text).expect("public JSON");
    // Independent public expectations cover every successful authored page.
    let rows: Vec<(&str, &str, &str)> = match label {
        label if label.starts_with('E') => vec![("BRAF", "Source name", "673")],
        "empty" => vec![],
        "mismatch" => vec![("OTHER", "", "1")],
        "case mismatch" => vec![("braf", "", "673")],
        "missing symbol" => vec![("", "", "673")],
        "null symbol" | "blank symbol" => vec![("", "", "")],
        "missing name" | "scalar alias" => vec![("BRAF", "", "673")],
        "null name" | "blank name" | "null id" | "HGNC equivalent" | "HGNC distinct"
        | "source shape" | "ambiguous" => vec![("BRAF", "", "")],
        "repeated id" => vec![("BRAF", "", "673"), ("OTHER", "", "1")],
        "ranked rows" => vec![
            ("BRAF", "Exact", "673"),
            ("OTHER", "First", "1"),
            ("THIRD", "Last", "3"),
        ],
        _ => panic!("missing independent public expectation for {label}"),
    };
    if get {
        let (symbol, name, entrez) = rows[0];
        assert_eq!(value["symbol"], symbol, "{label}");
        assert_eq!(value["name"], name, "{label}");
        assert_eq!(value["entrez_id"], entrez, "{label}");
        assert_eq!(value["ensembl_id"].as_str(), display, "{label}");
        assert!(value.get("identity").is_none());
        assert!(value.get("qualified").is_none());
    } else {
        let actual = value["results"].as_array().expect("search results");
        assert_eq!(value["count"], rows.len(), "{label}");
        assert_eq!(value["pagination"]["total"], rows.len(), "{label}");
        assert_eq!(actual.len(), rows.len(), "{label}");
        for (row, (symbol, name, entrez)) in actual.iter().zip(rows) {
            assert_eq!(row["symbol"], symbol, "{label}");
            assert_eq!(row["name"], name, "{label}");
            assert_eq!(row["entrez_id"], entrez, "{label}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn cli_raw_typed_gene_identity_table() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| root.join("target/debug/biomcp"));
    let harness = ContractHarness::new(binary, root);
    let filtering = decode_search(br#"{"total":51,"hits":[
        {"symbol":"OTHER","name":"First","entrezgene":1,"type_of_gene":"pseudo","genomic_pos":{"chr":"7","start":1,"end":9}},
        {"symbol":"BRAF","name":"Second","entrezgene":673,"type_of_gene":"protein-coding","genomic_pos":{"chr":"7","start":10,"end":20}},
        {"symbol":"LAST","name":"Third","entrezgene":3,"type_of_gene":"protein-coding","genomic_pos":{"chr":"8","start":10,"end":20}}
    ]}"#).unwrap();
    assert_eq!(filtering.total, 51);
    assert_eq!(
        filtering
            .hits
            .iter()
            .map(|hit| hit.symbol().unwrap())
            .collect::<Vec<_>>(),
        ["OTHER", "BRAF", "LAST"]
    );
    let filtered = super::filtered_gene_results(
        &filtering.hits,
        Some("protein-coding"),
        Some("7"),
        Some(&("7".into(), 15, 25)),
    );
    assert_eq!(
        filtered
            .iter()
            .map(|row| row.symbol.as_str())
            .collect::<Vec<_>>(),
        ["BRAF"]
    );
    assert_eq!(
        filtered[0].genomic_coordinates.as_ref().unwrap().coordinate,
        "7:10-20"
    );
    assert_eq!(filtering.total, 51);
    for case in controls::cases() {
        // Internal custody checks cover the same original page as all public routes.
        let parsed = biodata::parse_mygene_query(&case.bytes, biodata::MyGeneProfile::Get);
        if let Ok(page) = &parsed {
            assert_eq!(
                page.digest(),
                format!("sha256:{:x}", Sha256::digest(&case.bytes))
            );
            let original: Value = serde_json::from_slice(&case.bytes).unwrap();
            assert_eq!(page.total(), original.get("total").and_then(Value::as_u64));
            for (ordinal, disposition) in page.rows().iter().enumerate() {
                match disposition {
                    biodata::MyGeneRowDisposition::Projected(row) => {
                        assert_eq!(row.source().ordinal(), ordinal);
                        assert!(row.identity().qualified().is_none());
                        let document =
                            biodata::Document::GeneIdentity(Box::new(row.identity().clone()));
                        assert_eq!(
                            biodata::Document::from_json(&document.to_json().unwrap()).unwrap(),
                            document
                        );
                    }
                    biodata::MyGeneRowDisposition::Rejected {
                        ordinal: actual, ..
                    } => assert_eq!(*actual, ordinal),
                }
            }
        }
        let get = decode_get(&case.bytes, "BRAF");
        if let Some(expected) = case.get_error {
            let error = get.unwrap_err();
            if expected == "not_found" {
                assert_eq!(error.code(), expected);
            } else {
                assert!(
                    format!("{error:?}").contains(expected),
                    "{}: {error}",
                    case.label
                );
            }
        } else {
            let record = get.unwrap();
            use biodata::MyGeneField;
            match case.label {
                "missing name" => {
                    assert!(matches!(record.row().source().name(), MyGeneField::Missing))
                }
                "null name" => assert!(matches!(record.row().source().name(), MyGeneField::Null)),
                "blank name" => assert!(matches!(record.row().source().name(), MyGeneField::Blank)),
                "null id" => assert!(matches!(
                    record.row().source().provider_id(),
                    MyGeneField::Null
                )),
                "scalar alias" => {
                    assert!(record.row().source().alias_was_scalar());
                    assert_eq!(record.aliases(), ["SOURCEALIAS"]);
                    assert_eq!(record.code("NCBI Gene"), Some("673"));
                }
                "HGNC equivalent" => assert_eq!(
                    record.row().source().hgnc_was_number(),
                    [false, false, true]
                ),
                _ => {}
            }
            if ["missing name", "null name", "blank name"].contains(&case.label) {
                assert_eq!(crate::transform::gene::from_mygene_get(&record).0.name, "");
            }
            if case.label == "null id" {
                assert_eq!(record.code("NCBI Gene"), None);
            }
            if case.label == "E10 heterogeneous" {
                assert!(record.row().source().raw().contains("T1"));
                assert!(record.row().source().raw().contains("P2"));
            }

            assert_eq!(
                record.conversion.ensembl_display.as_deref(),
                case.display,
                "{}",
                case.label
            );
            if case.label.contains("E09")
                || case.label.contains("E10")
                || case.label.contains("E12")
                || case.label.contains("E13")
                || case.label.contains("E03")
            {
                assert!(!record.conversion.losses.is_empty(), "{}", case.label);
            }
            if case.label.contains("E11") {
                assert!(!record.row().losses().is_empty());
            }
            if case.label == "HGNC equivalent" {
                assert_eq!(record.hgnc_ids().unwrap(), ["HGNC:1097"]);
            }
            if case.label == "HGNC distinct" {
                assert_eq!(record.hgnc_ids().unwrap(), ["HGNC:1097", "HGNC:42"]);
            }
        }
        assert_eq!(
            decode_search(&case.bytes).is_err(),
            case.search_error,
            "{}",
            case.label
        );
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = Arc::clone(&requests);
        let body = case.bytes.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            captured
                .lock()
                .unwrap()
                .push(request.lines().next().unwrap_or_default().to_owned());
            let response = if request.starts_with("POST /graphql") {
                br#"{"data":{"search":{"hits":[]}}}"#.as_slice()
            } else if request.starts_with("GET /api/") {
                br#"{"response":{"numFound":0,"start":0,"docs":[]}}"#.as_slice()
            } else {
                &body
            };
            TestHttpReply::Bytes(test_http_response("200 OK", "application/json", response))
        })
        .await;
        let cache = tempfile::tempdir().unwrap();
        let env = [
            ("BIOMCP_MYGENE_BASE", fixture.base.clone()),
            ("BIOMCP_OLS4_BASE", fixture.base.clone()),
            ("BIOMCP_OPENTARGETS_BASE", fixture.base.clone()),
            ("BIOMCP_TEST_UNPACED_ORIGIN", fixture.base.clone()),
            ("BIOMCP_CACHE_DIR", cache.path().display().to_string()),
            ("RUST_LOG", "off,reqwest_retry=error".into()),
        ];
        for (get, args, error) in [
            (
                true,
                vec!["--json", "--no-cache", "get", "gene", "BRAF"],
                case.get_error.is_some(),
            ),
            (
                false,
                vec!["--json", "--no-cache", "search", "gene", "-q", "BRAF"],
                case.search_error,
            ),
        ] {
            let before = requests.lock().unwrap().len();
            let output = tokio::time::timeout(
                std::time::Duration::from_secs(15),
                tokio::process::Command::new(&harness.biomcp_bin)
                    .args(&args)
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
                "{}: {}",
                case.label,
                String::from_utf8_lossy(&output.stdout)
            );
            let text = String::from_utf8(output.stdout).unwrap();
            if error {
                let value: Value = serde_json::from_str(&text).unwrap();
                assert!(value["error"]["code"].is_string());
                assert!(output.stderr.is_empty());
            }
            assert_public(&text, error, case.label, get, case.display);
            if get && case.get_error.is_some_and(|kind| kind != "not_found") {
                assert_eq!(
                    requests.lock().unwrap().len() - before,
                    1,
                    "terminal get must not retry aliases"
                );
            }
        }
        let client = harness.spawn_stdio_client(&env).await.unwrap();
        for (get, tool, command, arguments, error) in [
            (
                true,
                "get",
                "biomcp --no-cache get gene BRAF",
                json!({"entity":"gene","id":"BRAF","json":true}),
                case.get_error.is_some(),
            ),
            (
                false,
                "search",
                "biomcp --no-cache search gene -q BRAF",
                json!({"entity":"gene","query":"BRAF","json":true}),
                case.search_error,
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
                    Some(error),
                    "{}: {}",
                    case.label,
                    first_text(&result.content)
                );
                assert_public(
                    first_text(&result.content),
                    error,
                    case.label,
                    get,
                    case.display,
                );
            }
        }
        if case.label == "E01 omitted" {
            let output = tokio::process::Command::new(&harness.biomcp_bin)
                .args(["--no-cache", "get", "gene", "BRAF"])
                .envs(env.iter().cloned())
                .output()
                .await
                .unwrap();
            assert!(output.status.success());
            assert!(String::from_utf8_lossy(&output.stdout).contains("BRAF"));
        }
        client.cancel().await.unwrap();
        assert!(
            requests
                .lock()
                .unwrap()
                .iter()
                .all(|request| request.starts_with("GET /query?")
                    || request.starts_with("GET /api/")
                    || request.starts_with("POST /graphql"))
        );
    }
}
