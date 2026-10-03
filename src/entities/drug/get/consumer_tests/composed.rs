//! Observe adopted effects in the same process as the real command execution.
//! Public subprocess controls independently retain complete CLI/MCP envelopes.
use super::*;
use crate::sources::mychem::consumer_tests_conversion::assert_conversion;

pub(super) async fn assert_composed_effects() {
    for id in [
        "P5-01/explicit-approvals-json-markdown",
        "P5-02/search-order-count-pagination",
        "P5-02/batch-order",
        "P5-06/typed-adopted-alias-search",
        "P6-05/adopted-empty-union-count",
    ] {
        let case = [5, 6]
            .into_iter()
            .flat_map(table)
            .find(|case| case["id"] == id)
            .unwrap();
        let fixture = CaseHttp::new(&case).await;
        let cache = tempfile::tempdir().unwrap();
        let mut env = TestEnv::new();
        fixture.environment(&mut env, cache.path());
        trial_alias_cache().lock().unwrap().clear();
        assert_eq!(cache_value(), case["cache_before"], "{id}: initial cache");
        let args = if id.contains("typed-adopted") || id.contains("union-count") {
            let mut args = vec![
                "biomcp",
                "--json",
                "search",
                "trial",
                "--intervention",
                "REQ-100",
                "--source",
                "ctgov",
                "--limit",
                "2",
            ];
            if id.contains("union-count") {
                args.push("--count-only");
            }
            args.into_iter().map(str::to_owned).collect::<Vec<_>>()
        } else {
            case["input"]
                .get("argv")
                .or_else(|| case["input"].get("argv_json"))
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|arg| arg.as_str().unwrap().to_owned())
                .collect()
        };
        let output = crate::cli::execute(args).await.unwrap();
        let output: Value = serde_json::from_str(&output).unwrap();
        let expected = &case["expected"];
        let path = expected["cli"]["stdout_json"]
            .as_str()
            .or_else(|| expected["cli_json"]["stdout_json"].as_str())
            .or_else(|| expected["mcp_result"]["content"][0]["text_json"].as_str())
            .unwrap();
        assert_eq!(output, asset(path), "{id}: complete composed output");
        fixture.assert_requests(&case["id"]);
        assert_eq!(
            cache_value(),
            case["cache_after"],
            "{id}: complete composed cache"
        );
        let hits = fixture.admitted_hits.lock().unwrap();
        let refs = hits.iter().collect::<Vec<_>>();
        if id == "P5-02/batch-order" {
            fixture.assert_admitted_dependencies(&case["id"], "used:get");
            for item in expected["items"].as_array().unwrap() {
                let object = if let Some(reference) = item.get("adopted_object_reference") {
                    let referenced = table(5)
                        .into_iter()
                        .find(|row| row["id"] == reference["case"])
                        .unwrap();
                    referenced["expected"].clone()
                } else {
                    asset(item["adopted_object"].as_str().unwrap())
                };
                assert_conversion(&refs, &object["conversion"], "get", &case["id"]);
                let path = object["page"].as_str().unwrap();
                let page_hits = hits
                    .iter()
                    .filter(|hit| hit.page.digest() == asset(path)["digest"])
                    .cloned()
                    .collect::<Vec<_>>();
                let page = crate::sources::mychem::MyChemQueryResponse {
                    total: page_hits[0].page.total() as usize,
                    hits: page_hits,
                };
                assert_page(&page, path);
            }
        } else if id.contains("typed-adopted") || id.contains("union-count") {
            fixture.assert_admitted_dependencies(&case["id"], "used:aliases");
            let reference = &expected["alias_conversion_reference"];
            let alias = table(6)
                .into_iter()
                .find(|row| row["id"] == reference["case"])
                .unwrap();
            assert_conversion(
                &refs,
                &alias["expected"]["conversion"],
                "alias",
                &case["id"],
            );
            // The exact same Get input/candidate extraction and base ledger are
            // completely compared by the owning adopted alias control above.
            assert_page(
                &crate::sources::mychem::MyChemQueryResponse {
                    total: hits[0].page.total() as usize,
                    hits: hits.clone(),
                },
                alias["expected"]["page"].as_str().unwrap(),
            );
        } else {
            assert_conversion(
                &refs,
                &expected["conversion"],
                if id.contains("search-order") {
                    "ranking"
                } else {
                    "get"
                },
                &case["id"],
            );
            if let Some(conversion) = expected.get("identity_lookup_conversion") {
                assert_conversion(&refs, conversion, "get", &case["id"]);
            }
        }
    }
    trial_alias_cache().lock().unwrap().clear();
}
