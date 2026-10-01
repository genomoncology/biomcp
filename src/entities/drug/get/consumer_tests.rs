//! Consumer controls execute immutable independent alternatives.
use super::*;
use crate::entities::article::test_support::{
    TestEnv, TestHttpFixture, TestHttpReply, test_http_response,
};
use crate::sources::mychem::consumer_tests::{
    assert_page, asset, bytes, product_value, profile, root, search_value, table,
};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

struct CaseHttp {
    fixture: TestHttpFixture,
    requests: Arc<Mutex<Vec<String>>>,
    expected: Vec<Value>,
}
impl CaseHttp {
    async fn new(case: &Value) -> Self {
        let responses = case["responses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|response| response.get("body").is_some())
            .cloned()
            .collect::<Vec<_>>();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let fixture = TestHttpFixture::spawn(move |request| {
            let mut captured = captured.lock().unwrap();
            let index = captured.len();
            captured.push(request.lines().next().unwrap_or("").to_owned());
            let Some(response) = responses.get(index) else {
                return TestHttpReply::Bytes(test_http_response(
                    "400 Unexpected request",
                    "application/json",
                    b"{}",
                ));
            };
            let bytes =
                std::fs::read(root().join(response["body"]["path"].as_str().unwrap())).unwrap();
            TestHttpReply::Bytes(test_http_response(
                &format!("{} Fixture", response["status"]),
                response["content_type"].as_str().unwrap(),
                &bytes,
            ))
        })
        .await;
        let expected = case["requests_expected"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|request| request.get("method").is_some())
            .cloned()
            .collect();
        Self {
            fixture,
            requests,
            expected,
        }
    }
    fn environment(&self, env: &mut TestEnv, cache: &std::path::Path) {
        env.set("BIOMCP_MYCHEM_BASE", format!("{}/v1", self.fixture.base));
        for name in [
            "BIOMCP_OPENFDA_BASE",
            "BIOMCP_CTGOV_BASE",
            "BIOMCP_CIVIC_BASE",
            "BIOMCP_OLS4_BASE",
            "BIOMCP_CHEMBL_BASE",
            "BIOMCP_OPENTARGETS_BASE",
        ] {
            env.set(name, &self.fixture.base);
        }
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &self.fixture.base);
        env.set("BIOMCP_CACHE_DIR", cache);
    }
    fn assert_requests(&self, id: &Value) {
        let actual = self.requests.lock().unwrap();
        assert_eq!(actual.len(), self.expected.len(), "{id}: {actual:?}");
        for (actual, expected) in actual.iter().zip(&self.expected) {
            let tokens = actual.split_whitespace().collect::<Vec<_>>();
            assert_eq!(tokens[0], expected["method"].as_str().unwrap(), "{id}");
            let url = reqwest::Url::parse(&format!("http://fixture{}", tokens[1])).unwrap();
            assert_eq!(url.path(), expected["path"].as_str().unwrap(), "{id}");
            if let Some(query) = expected["query"].as_array() {
                assert_eq!(
                    json!(
                        url.query_pairs()
                            .map(|(key, value)| (key.into_owned(), value.into_owned()))
                            .collect::<Vec<_>>()
                    ),
                    json!(query),
                    "{id}"
                );
            }
        }
    }
}
fn failure(result: BioMcpError, wanted: &Value, id: &Value) {
    let wanted = if let Some(path) = wanted.as_str() {
        asset(path)
    } else {
        wanted.clone()
    };
    let wanted = wanted.get("product").unwrap_or(&wanted);
    assert_eq!(result.code(), wanted["code"].as_str().unwrap(), "{id}");
    if let Some(display) = wanted["display"].as_str() {
        assert_eq!(result.to_string(), display, "{id}");
    }
    for channel in [result.to_string(), format!("{result:?}")] {
        for sentinel in ["QUERY_SENTINEL_0224", "SOURCE_SENTINEL_0224"] {
            assert!(!channel.contains(sentinel), "{id}: {channel}");
        }
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn adopted_request_search_and_nested_failure_table() {
    let _cache_mode = crate::sources::test_cache_mode::off();
    for number in [1, 2, 4] {
        for case in table(number) {
            let op = case["input"]["operation"].as_str().unwrap_or("");
            if !matches!(
                op,
                "direct_drug_lookup"
                    | "search_page"
                    | "resolve_drug_base"
                    | "search_name_query_with_region"
            ) {
                continue;
            }
            let fixture = CaseHttp::new(&case).await;
            let cache = tempfile::tempdir().unwrap();
            let mut env = TestEnv::new();
            fixture.environment(&mut env, cache.path());
            let input = &case["input"];
            let expected = &case["expected"];
            let id = &case["id"];
            let name = input["name"]
                .as_str()
                .or_else(|| input["query"].as_str())
                .unwrap_or("sampledrug");
            match op {
                "direct_drug_lookup" => {
                    let result = direct_drug_lookup(name).await;
                    if !expected["failure"].is_null() {
                        failure(result.unwrap_err(), &expected["failure"], id);
                    } else {
                        assert_page(&result.unwrap(), expected["page"].as_str().unwrap());
                    }
                }
                "search_page" => {
                    let filters: DrugSearchFilters =
                        serde_json::from_value(input["filters"].clone()).unwrap();
                    let result = search_page(
                        &filters,
                        input["limit"].as_u64().unwrap() as usize,
                        input["offset"].as_u64().unwrap() as usize,
                    )
                    .await;
                    if !expected["failure"].is_null() {
                        failure(result.unwrap_err(), &expected["failure"], id);
                    } else {
                        let page = result.unwrap();
                        assert_eq!(
                            json!(page.results.iter().map(search_value).collect::<Vec<_>>()),
                            expected["results"],
                            "{id}"
                        );
                        assert_eq!(json!(page.total), expected["total"], "{id}");
                    }
                }
                "resolve_drug_base" => {
                    let injection = &input["discover_resolver_injection"]["result"]["Ok"]["path"];
                    let result = resolve_drug_base_with_discover(
                        name,
                        input["fetch_label_response"].as_bool().unwrap_or(false),
                        input["label_required"].as_bool().unwrap_or(false),
                        async {
                            if let Some(path) = injection.as_str() {
                                let result: crate::entities::discover::DiscoverResult =
                                    serde_json::from_value(asset(path)).unwrap();
                                classify_sparse_drug_rescue(&result)
                            } else {
                                SparseDrugDiscoverRescue::None
                            }
                        },
                    )
                    .await;
                    if !expected["failure"].is_null() {
                        failure(
                            result.err().expect("terminal failure"),
                            &expected["failure"],
                            id,
                        );
                    } else {
                        let resolved = result.unwrap();
                        let wanted = expected
                            .get("product")
                            .unwrap_or(&expected["resolved_base"]["drug"]);
                        assert_eq!(product_value(&resolved.drug), *wanted, "{id}");
                        assert_eq!(
                            json!(
                                resolved
                                    .selected_hits
                                    .iter()
                                    .map(|hit| hit.row.source().ordinal())
                                    .collect::<Vec<_>>()
                            ),
                            expected["selected_ordinals"],
                            "{id}"
                        );
                    }
                }
                _ => {
                    let region = match input["region"].as_str().unwrap() {
                        "eu" => DrugRegion::Eu,
                        "who" => DrugRegion::Who,
                        "all" => DrugRegion::All,
                        _ => panic!("region"),
                    };
                    let result = super::super::search::search_name_query_with_region(
                        name,
                        2,
                        0,
                        region,
                        WhoProductTypeFilter::Both,
                    )
                    .await;
                    failure(
                        result.err().expect("terminal failure"),
                        &expected["failure"],
                        id,
                    );
                }
            }
            fixture.assert_requests(id);
        }
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn adopted_alias_cache_and_downstream_table() {
    let _cache_mode = crate::sources::test_cache_mode::off();
    let case = table(6).remove(0);
    let fixture = CaseHttp::new(&case).await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = TestEnv::new();
    fixture.environment(&mut env, cache.path());
    trial_alias_cache().lock().unwrap().clear();
    let requested = case["input"]["requested_name"].as_str().unwrap();
    let resolution = resolve_trial_alias_resolution(requested).await.unwrap();
    let actual = resolution
        .aliases
        .iter()
        .map(|alias| json!({"label":alias.label,"source":format!("{:?}",alias.source)}))
        .collect::<Vec<_>>();
    assert_eq!(json!(actual), case["expected"]["aliases"]);
    assert_eq!(
        resolution.canonical_name,
        case["expected"]["canonical_name"]
    );
    assert_eq!(trial_alias_cache().lock().unwrap().len(), 1);
    fixture.assert_requests(&case["id"]);
    let repeated = resolve_trial_alias_resolution("req-100").await.unwrap();
    assert_eq!(repeated.aliases[0].label, "req-100");
    fixture.assert_requests(&case["id"]);
    trial_alias_cache().lock().unwrap().clear();
    let rejection = crate::sources::mychem::projection::decode(
        &std::fs::read(root().join("inputs/mixed-rejected.json")).unwrap(),
        biodata::MyChemProfile::Get,
    )
    .unwrap_err();
    assert!(
        resolve_trial_alias_resolution_with_lookup("sampledrug", async { Err(rejection) })
            .await
            .is_err()
    );
    assert!(trial_alias_cache().lock().unwrap().is_empty());
    for case in table(6).into_iter().filter(|case| {
        case["input"]["operation"] == "admitted_page_then_fda_orphan_alias_admission"
    }) {
        let response =
            crate::sources::mychem::projection::decode(&bytes(&case), profile(&case)).unwrap();
        let mut drug = crate::transform::drug::merge_mychem_hits(&[], "sampledrug");
        drug.drugbank_id = case["input"]["resolved"]["drugbank_id"]
            .as_str()
            .map(str::to_owned);
        drug.chembl_id = case["input"]["resolved"]["chembl_id"]
            .as_str()
            .map(str::to_owned);
        drug.unii = case["input"]["resolved"]["unii"]
            .as_str()
            .map(str::to_owned);
        let aliases = orphan_aliases(
            case["input"]["requested_name"].as_str().unwrap(),
            &drug,
            &response.hits,
        );
        assert_eq!(json!(aliases), case["expected"]["aliases"]);
    }
}

mod surfaces;

mod absence;
