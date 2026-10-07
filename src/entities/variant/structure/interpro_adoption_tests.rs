//! Real structure calls own first inclusive overlap and public failure states.
use super::*;
use crate::entities::article::test_support::TestEnv;
use crate::entities::protein::interpro_adoption_tests::{
    PRIVATE, body, environments, fixture, requests,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

fn check(text: &str, json_mode: bool, state: &str) {
    assert!(!text.contains("private-interpro-marker"), "{text}");
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["protein"]["accession"], "P15056");
        assert_eq!(card["structures"]["pdb"][0]["id"], "1ABC");
        assert_eq!(card["lookup_outcomes"]["domains"]["outcome"], state);
        assert_eq!(
            card["lookup_outcomes"]["domains"]["sources"],
            if state == "data" || state == "empty" {
                json!(["InterPro"])
            } else {
                json!([])
            }
        );
        assert_eq!(
            card["domains"],
            if state == "data" {
                json!([{"accession":"IPR000719","name":"Protein kinase","type":"domain","start":457,"end":717,"source":"InterPro"},{"accession":"IPR000719","start":600,"end":600,"source":"InterPro"}])
            } else {
                json!([])
            }
        );
        if state != "inapplicable" {
            assert_eq!(card["residue"]["position"], 600);
            assert_eq!(card["residue"]["other_source_positions"], json!([601]));
            assert!(!card["warnings"].as_array().unwrap().is_empty());
            assert!(
                card["residue"]["matched_hgvsp"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("p.Val600Glu"))
            );
        }
    } else {
        assert!(text.contains("P15056") && text.contains("1ABC"), "{text}");
        if state == "data" {
            assert!(text.contains("Protein kinase"), "{text}");
            assert!(
                !text.contains("IPR_OVER_CAP") && !text.contains("IPR_NO_OVERLAP"),
                "{text}"
            );
        }
        if state == "unavailable" {
            assert!(text.contains("unavailable"), "{text}");
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn interpro_structure_reaches_native_cli_and_raw_mcp() {
    for (body, state, no_change) in [
        (body(25), "data", false),
        (b"{\"results\":[]}".to_vec(), "empty", false),
        (PRIVATE.to_vec(), "unavailable", false),
        (body(25), "inapplicable", true),
    ] {
        let (fixture, captured) = fixture(body, "200 OK", no_change).await;
        let cache = tempfile::tempdir().unwrap();
        let envs = environments(&fixture.base, cache.path());
        let mut env = TestEnv::new();
        for (key, value) in &envs {
            env.set(key, value);
        }
        let id = if no_change {
            "chr7:g.140453136A>T"
        } else {
            "BRAF V600E"
        };
        let native = structure(id).await.unwrap();
        let restored: VariantStructureResult =
            serde_json::from_value(serde_json::to_value(&native).unwrap()).unwrap();
        check(&serde_json::to_string(&restored).unwrap(), true, state);
        check(
            &crate::render::markdown::variant_structure_markdown(&restored),
            false,
            state,
        );
        let harness = ContractHarness::new(
            std::env::var_os("BIOMCP_BIN").unwrap(),
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        );
        let client = harness.spawn_stdio_client(&envs).await.unwrap();
        for json_mode in [true, false] {
            let mut args = vec!["variant", "structure", id];
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
            assert!(!String::from_utf8_lossy(&output.stderr).contains("private-interpro-marker"));
            check(
                std::str::from_utf8(&output.stdout).unwrap(),
                json_mode,
                state,
            );
            let result = client.peer().call_tool(CallToolRequestParams::new("biomcp").with_arguments(json!({"command":format!("biomcp variant structure '{id}'"),"json":json_mode}).as_object().unwrap().clone())).await.unwrap();
            assert!(!result.is_error.unwrap_or_default(), "{result:?}");
            check(first_text(&result.content), json_mode, state);
        }
        client.cancel().await.unwrap();
        requests(&captured, 25, !no_change);
    }
}
