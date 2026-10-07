//! Structure keeps other context when recurrence is empty, inapplicable or unavailable.
use super::*;
use crate::entities::article::test_support::TestEnv;
use crate::entities::variant::get::cancerhotspots_transport_tests::{
    CAPTURE, PRIVATE, environments, expected, fixture, hotspot_requests,
};
use biomcp_mcp_contract_client::{ContractHarness, first_text};
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

fn check(text: &str, json_mode: bool, state: &str) {
    assert!(!text.contains("private-hotspots-marker"), "{text}");
    if json_mode {
        let card: Value = serde_json::from_str(text).unwrap();
        assert_eq!(card["lookup_outcomes"]["cancerhotspots"]["outcome"], state);
        let healthy = state == "data" || state == "empty";
        assert_eq!(
            card["lookup_outcomes"]["cancerhotspots"]["sources"],
            if healthy {
                json!(["cancerhotspots.org"])
            } else {
                json!([])
            }
        );
        assert_eq!(
            card["cancerhotspots"],
            if healthy {
                expected(state == "data")
            } else {
                Value::Null
            }
        );
        assert_eq!(card["protein"]["accession"], "P15056");
        assert_eq!(card["structures"]["pdb"][0]["id"], "1ABC");
        if state != "inapplicable" {
            assert_eq!(card["domains"][0]["accession"], "IPR000719");
            assert_eq!(card["lookup_outcomes"]["domains"]["outcome"], "data");
        }
    } else {
        assert!(text.contains("## Cancerhotspots"), "{text}");
        assert!(text.contains("P15056"), "{text}");
        assert!(text.contains("1ABC"), "{text}");
        if state == "data" {
            assert!(text.contains("Position count: 897"), "{text}");
        }
    }
}
#[tokio::test(flavor = "multi_thread")]
#[serial_test::serial(source_env)]
async fn hotspots_structure_reaches_native_cli_and_raw_mcp() {
    for (body, state, no_change) in [
        (CAPTURE, "data", false),
        (&b"[]"[..], "empty", false),
        (PRIVATE, "unavailable", false),
        (CAPTURE, "inapplicable", true),
    ] {
        let (fixture, requests) = fixture(body.to_vec(), no_change).await;
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
        if let Some(section) = &native.cancerhotspots {
            let _: &biodata::CancerHotspotRecurrenceProjection = section.recurrence();
        }
        if state == "data" {
            let base = serde_json::to_value(&native).unwrap();
            for target in [
                expected(false),
                json!({"source":"custom source","position_count":1,"same_aa_count":null,"matched_transcript":"  "}),
                Value::Null,
            ] {
                let mut card = base.clone();
                card["cancerhotspots"] = target.clone();
                let restored: VariantStructureResult = serde_json::from_value(card).unwrap();
                assert_eq!(
                    serde_json::to_value(&restored).unwrap()["cancerhotspots"],
                    target
                );
                crate::render::markdown::variant_structure_markdown(&restored);
            }
            let mut card = base.clone();
            card.as_object_mut().unwrap().remove("cancerhotspots");
            assert!(
                serde_json::from_value::<VariantStructureResult>(card)
                    .unwrap()
                    .cancerhotspots
                    .is_none()
            );
            let mut card = base;
            card["cancerhotspots"] = json!({"position_count":1});
            assert!(serde_json::from_value::<VariantStructureResult>(card).is_err());
        }
        check(&serde_json::to_string(&native).unwrap(), true, state);
        check(
            &crate::render::markdown::variant_structure_markdown(&native),
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
            assert!(!String::from_utf8_lossy(&output.stderr).contains("private-hotspots-marker"));
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
        hotspot_requests(&requests, if no_change { 0 } else { 5 });
    }
}
