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

pub(crate) struct CaseHttp {
    fixture: TestHttpFixture,
    requests: Arc<Mutex<Vec<String>>>,
    expected: Vec<Value>,
    order: Option<Value>,
    events: Arc<Mutex<Vec<(usize, &'static str)>>>,
    pub(crate) admitted_hits: Arc<Mutex<Vec<MyChemHit>>>,
    _observer: crate::sources::mychem::test_observer::Guard,
}
impl CaseHttp {
    pub(crate) async fn new(case: &Value) -> Self {
        let responses = case["responses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|response| response.get("body").is_some())
            .cloned()
            .collect::<Vec<_>>();
        let expected: Vec<Value> = case["requests_expected"]
            .as_array()
            .unwrap()
            .iter()
            .map(|request| {
                if let Some(reference) = request.get("request_reference") {
                    (1..=6)
                        .flat_map(table)
                        .find(|row| row["id"] == reference["case"])
                        .unwrap()
                        .pointer(reference["pointer"].as_str().unwrap())
                        .unwrap()
                        .clone()
                } else {
                    request.clone()
                }
            })
            .collect();
        let expected: Vec<Value> = expected
            .into_iter()
            .filter(|request| request.get("method").is_some())
            .collect();
        let plans = expected.clone();
        let consumed = Mutex::new(vec![false; responses.len()]);
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let events = Arc::new(Mutex::new(Vec::new()));
        let captured_events = events.clone();
        let response_digests = responses
            .iter()
            .map(|response| response["body"]["digest"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let observed_events = events.clone();
        let admitted_hits = Arc::new(Mutex::new(Vec::new()));
        let observed_hits = admitted_hits.clone();
        let observer = crate::sources::mychem::test_observer::observe(move |hits, stage| {
            let Some(hit) = hits.first() else {
                return;
            };
            let mut events = observed_events.lock().unwrap();
            let Some(index) = events.iter().rev().find_map(|(index, event)| {
                (*event == "request" && response_digests[*index] == hit.page.digest())
                    .then_some(*index)
            }) else {
                return;
            };
            if stage == "admitted" {
                observed_hits.lock().unwrap().extend_from_slice(hits);
            }
            events.push((index, stage));
        });
        let fixture = TestHttpFixture::spawn(move |request| {
            let mut captured = captured.lock().unwrap();
            captured.push(request.to_owned());
            let mut consumed = consumed.lock().unwrap();
            let index = plans.iter().enumerate().position(|(index, plan)| {
                !consumed.get(index).copied().unwrap_or(true) && request_matches(request, plan)
            });
            if let Some(index) = index {
                captured_events.lock().unwrap().push((index, "request"));
            }
            let Some(response) = index.and_then(|index| {
                consumed[index] = true;
                responses.get(index)
            }) else {
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
        Self {
            fixture,
            requests,
            expected,
            order: case.get("request_order_contract").cloned().or_else(|| {
                if case["input"]["request_order"] == "MyChem first; CTGov worker plans in listed order, dispatched concurrently; network arrival order is not contractual" {
                    Some(table(5).into_iter().find(|row| row["id"] == "P5-06/typed-adopted-alias-search").unwrap()["request_order_contract"].clone())
                } else { None }
            }),
            events,
            admitted_hits,
            _observer: observer,
        }
    }
    pub(crate) fn assert_admitted_dependencies(&self, id: &Value, use_stage: &'static str) {
        let contract = self.order.as_ref().unwrap();
        let ids = contract["request_ids"].as_array().unwrap();
        let events = self.events.lock().unwrap();
        for edge in contract["before"].as_array().unwrap() {
            let before = ids.iter().position(|value| value == &edge[0]).unwrap();
            let after = ids.iter().position(|value| value == &edge[1]).unwrap();
            let admitted = events
                .iter()
                .position(|event| *event == (before, "admitted"))
                .unwrap_or_else(|| {
                    panic!("{id}: no successfully admitted prerequisite: {events:?}")
                });
            let used = events
                .iter()
                .position(|event| *event == (before, use_stage))
                .unwrap_or_else(|| panic!("{id}: prerequisite values not consumed: {events:?}"));
            let dispatched = events
                .iter()
                .position(|event| *event == (after, "request"))
                .unwrap();
            assert!(
                admitted < used && used < dispatched,
                "{id}: receipt/admission/use before dependent dispatch: {events:?}"
            );
        }
    }
    pub(crate) fn environment(&self, env: &mut TestEnv, cache: &std::path::Path) {
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
        env.set("BIOMCP_CTGOV_BASE", format!("{}/api/v2", self.fixture.base));
        env.set("BIOMCP_CIVIC_BASE", format!("{}/api", self.fixture.base));
        env.set("BIOMCP_TEST_UNPACED_ORIGIN", &self.fixture.base);
        env.set("BIOMCP_CACHE_DIR", cache);
        for key in [
            "NCI_API_KEY",
            "OPENFDA_API_KEY",
            "UMLS_API_KEY",
            "NCBI_API_KEY",
            "S2_API_KEY",
        ] {
            env.set(key, "");
        }
    }
    pub(crate) fn assert_requests(&self, id: &Value) {
        let actual = self.requests.lock().unwrap();
        assert_eq!(actual.len(), self.expected.len(), "{id}: {actual:?}");
        let mut matched = vec![false; actual.len()];
        let positions = self
            .expected
            .iter()
            .map(|expected| {
                let index = actual
                    .iter()
                    .enumerate()
                    .position(|(index, actual)| {
                        !matched[index] && request_matches(actual, expected)
                    })
                    .unwrap_or_else(|| panic!("{id}: missing exact request {expected}"));
                matched[index] = true;
                index
            })
            .collect::<Vec<_>>();
        if let Some(contract) = &self.order {
            let ids = contract["request_ids"].as_array().unwrap();
            assert_eq!(
                ids.len(),
                positions.len(),
                "{id}: complete request identities"
            );
            for edge in contract["before"].as_array().unwrap() {
                let before = ids.iter().position(|value| value == &edge[0]).unwrap();
                let after = ids.iter().position(|value| value == &edge[1]).unwrap();
                assert!(
                    positions[before] < positions[after],
                    "{id}: causal edge {edge}"
                );
            }
            assert_eq!(
                contract["response_indices"],
                json!((0..positions.len()).collect::<Vec<_>>())
            );
        } else {
            assert_eq!(
                positions,
                (0..actual.len()).collect::<Vec<_>>(),
                "{id}: sequential request order"
            );
        }
    }
}
fn request_matches(request: &str, expected: &Value) -> bool {
    let tokens = request
        .lines()
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>();
    if tokens.len() < 2 || Some(tokens[0]) != expected["method"].as_str() {
        return false;
    }
    let url = reqwest::Url::parse(&format!("http://fixture{}", tokens[1])).unwrap();
    if Some(url.path()) != expected["path"].as_str() {
        return false;
    }
    if expected.get("query").is_some_and(|query| {
        json!(
            url.query_pairs()
                .map(|(key, value)| (key.into_owned(), value.into_owned()))
                .collect::<Vec<_>>()
        ) != *query
    }) {
        return false;
    }
    if let Some(body) = expected.get("body") {
        let Some((_, actual)) = request.split_once("\r\n\r\n") else {
            return false;
        };
        let Ok(actual) = serde_json::from_str::<Value>(actual) else {
            return false;
        };
        let query =
            std::fs::read_to_string(root().join(body["query_file"]["path"].as_str().unwrap()))
                .unwrap();
        if actual != json!({"query":query,"variables":body["variables"]}) {
            return false;
        }
    }
    true
}
fn failure(result: BioMcpError, wanted: &Value, id: &Value) {
    let wanted = if let Some(path) = wanted.as_str() {
        asset(path)
    } else {
        wanted.clone()
    };
    let wanted = wanted.get("product").unwrap_or(&wanted);
    assert_eq!(result.code(), wanted["code"].as_str().unwrap(), "{id}");
    fn variants(error: &BioMcpError) -> Vec<&'static str> {
        match error {
            BioMcpError::WithSourceContext { source, .. } => {
                let mut values = vec!["WithSourceContext"];
                values.extend(variants(source));
                values
            }
            BioMcpError::Api { .. } => vec!["Api"],
            BioMcpError::ApiJson { .. } => vec!["ApiJson"],
            BioMcpError::BodyLimit { .. } => vec!["BodyLimit"],
            BioMcpError::InvalidArgument(_) => vec!["InvalidArgument"],
            BioMcpError::SourceUnavailable { .. } => vec!["SourceUnavailable"],
            BioMcpError::NotFound { .. } => vec!["NotFound"],
            BioMcpError::ProviderResponseLimit { .. } => vec!["ProviderResponseLimit"],
            _ => vec!["unexpected"],
        }
    }
    if let Some(wanted_context) = wanted.get("context") {
        let BioMcpError::WithSourceContext { context, .. } = &result else {
            panic!("{id}: missing source context");
        };
        assert_eq!(
            json!({"provider":context.provider().label(),"recovery":format!("{:?}",context.recovery())}),
            *wanted_context,
            "{id}"
        );
    }
    if let Some(chain) = wanted.get("variant_chain") {
        assert_eq!(json!(variants(&result)), *chain, "{id}");
    }
    if let Some(variant) = wanted.get("variant") {
        assert_eq!(json!(variants(&result).last().unwrap()), *variant, "{id}");
    }
    if let Some(exit) = wanted["exit"].as_u64() {
        assert_eq!(u64::from(result.exit_code()), exit, "{id}");
    }
    if let Some(public) = wanted.get("public_json") {
        assert_eq!(
            serde_json::from_str::<Value>(&crate::render::json::to_error_json(&result).unwrap())
                .unwrap(),
            *public,
            "{id}"
        );
    }
    if let Some(contains) = wanted["diagnostic_contains"].as_str() {
        assert!(result.to_string().contains(contains), "{id}");
    }

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
                    let result = if let Some(max) = input["transport_max_body_bytes"].as_u64() {
                        crate::sources::mychem::MyChemClient::new()
                            .unwrap()
                            .test_get_with_body_limit(name, max as usize)
                            .await
                    } else {
                        direct_drug_lookup(name).await
                    };
                    if !expected["failure"].is_null() {
                        failure(result.unwrap_err(), &expected["failure"], id);
                    } else {
                        assert_page(&result.unwrap(), expected["page"].as_str().unwrap());
                    }
                }
                "search_page" => {
                    let filters: DrugSearchFilters =
                        serde_json::from_value(input["filters"].clone()).unwrap();
                    let result = super::super::search::search_page_with_custody(
                        &filters,
                        input["limit"].as_u64().unwrap() as usize,
                        input["offset"].as_u64().unwrap() as usize,
                    )
                    .await;
                    if !expected["failure"].is_null() {
                        failure(result.unwrap_err(), &expected["failure"], id);
                    } else {
                        let (page, custody) = result.unwrap();
                        if let Some(path) = expected["page"].as_str() {
                            assert_page(&custody, path);
                        }
                        if let Some(wanted) = expected.get("conversion") {
                            crate::sources::mychem::consumer_tests_conversion::assert_conversion(
                                &custody.hits.iter().collect::<Vec<_>>(),
                                wanted,
                                "search",
                                id,
                            );
                        }
                        assert_eq!(
                            json!(page.results.iter().map(search_value).collect::<Vec<_>>()),
                            *expected.get("results").unwrap_or(&expected["rows"]),
                            "{id}"
                        );
                        assert_eq!(json!(page.total), expected["total"], "{id}");
                    }
                }
                "resolve_drug_base" => {
                    let injection = &input["discover_resolver_injection"]["result"]["Ok"]["path"];
                    let mut report = DrugResolverReport::default();
                    let result = resolve_drug_base_with_discover_report(
                        name,
                        input["fetch_label_response"].as_bool().unwrap_or(false),
                        input["label_required"].as_bool().unwrap_or(false),
                        async {
                            if let Some(path) = injection.as_str() {
                                let result = discover_input(path);
                                classify_sparse_drug_rescue(&result)
                            } else {
                                SparseDrugDiscoverRescue::none()
                            }
                        },
                        &mut report,
                    )
                    .await;
                    if let Some(wanted) = expected.get("conversion") {
                        let discovery = wanted
                            .as_array()
                            .unwrap()
                            .iter()
                            .filter(|entry| {
                                entry.get("source_pointer").is_some_and(|pointer| {
                                    pointer.as_str().unwrap_or("").starts_with("/concepts/")
                                })
                            })
                            .cloned()
                            .collect::<Vec<_>>();
                        assert_eq!(
                            json!(report.discovery),
                            json!(discovery),
                            "{id}: complete discovery contributions"
                        );
                        if !report.discarded_hits.is_empty() {
                            let discarded = wanted
                                .as_array()
                                .unwrap()
                                .iter()
                                .filter(|entry| entry["action"] == "discard_sparse_product")
                                .cloned()
                                .collect::<Vec<_>>();
                            crate::sources::mychem::consumer_tests_conversion::assert_conversion(
                                &report.discarded_hits.iter().collect::<Vec<_>>(),
                                &json!(discarded),
                                "get",
                                id,
                            );
                        }
                    }
                    if !expected["failure"].is_null() {
                        failure(
                            result.err().expect("terminal failure"),
                            &expected["failure"],
                            id,
                        );
                    } else {
                        let resolved = result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
                        if let Some(wanted) = expected.get("conversion") {
                            let fallback_entries = wanted
                                .as_array()
                                .unwrap()
                                .iter()
                                .filter(|entry| entry["action"] == "fallback")
                                .collect::<Vec<_>>();
                            assert_eq!(
                                resolved.fallbacks.len(),
                                fallback_entries.len(),
                                "{id}: all fallback decisions"
                            );
                            for (actual, wanted) in resolved.fallbacks.iter().zip(fallback_entries)
                            {
                                assert_eq!(actual.from, wanted["from"], "{id}");
                                assert_eq!(actual.to, wanted["to"], "{id}");
                                assert_eq!(actual.reason, wanted["reason"], "{id}");
                                if let Some(origin) = wanted.get("candidate_origin") {
                                    assert_eq!(json!(actual.candidate_origin.as_ref().map(|(digest, ordinal)| json!({"digest":digest,"ordinal":ordinal}))), *origin, "{id}");
                                }
                            }
                            let claims = wanted
                                .as_array()
                                .unwrap()
                                .iter()
                                .filter(|entry| {
                                    entry["action"] != "fallback"
                                        && entry.get("source_pointer").is_none_or(|pointer| {
                                            !pointer
                                                .as_str()
                                                .unwrap_or("")
                                                .starts_with("/concepts/")
                                        })
                                        && entry.get("response_digest").is_some()
                                })
                                .cloned()
                                .collect::<Vec<_>>();
                            crate::sources::mychem::consumer_tests_conversion::assert_conversion_with_signals(
                                &resolved
                                    .source_pages
                                    .iter()
                                    .flat_map(|page| page.hits.iter())
                                    .collect::<Vec<_>>(),
                                &resolved.label_signals,
                                &json!(claims),
                                "get",
                                id,
                            );
                        }

                        if let Some(paths) = expected["prior_pages"].as_array() {
                            assert!(resolved.source_pages.len() >= paths.len(), "{id}");
                            for (page, path) in resolved.source_pages.iter().zip(paths) {
                                assert_page(page, path.as_str().unwrap());
                            }
                        }
                        if let Some(path) = expected["page"].as_str() {
                            assert_page(resolved.source_pages.last().unwrap(), path);
                        }
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
    let captured = std::sync::Mutex::new(Vec::new());
    let resolution = resolve_trial_alias_resolution_with_custody(requested, async {
        let resolved = resolve_drug_base(requested, false, false).await?;
        captured.lock().unwrap().extend(
            resolved
                .source_pages
                .iter()
                .flat_map(|page| page.hits.clone()),
        );
        Ok((
            TrialAliasLookup {
                canonical_name: resolved.drug.name,
                candidates: resolved.trial_alias_candidates,
            },
            resolved.selected_hits,
        ))
    })
    .await
    .unwrap();
    crate::sources::mychem::consumer_tests_conversion::assert_conversion(
        &captured.lock().unwrap().iter().collect::<Vec<_>>(),
        &case["expected"]["conversion"],
        "alias",
        &case["id"],
    );
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
    for case in table(6)
        .into_iter()
        .filter(|case| case["input"]["operation"] == "get_drug_with_sections")
    {
        let fixture = CaseHttp::new(&case).await;
        let cache = tempfile::tempdir().unwrap();
        fixture.environment(&mut env, cache.path());
        let sections = case["input"]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        let drug = get(case["input"]["name"].as_str().unwrap(), &sections)
            .await
            .unwrap();
        assert_eq!(
            product_value(&drug),
            case["expected"]["product"],
            "{}",
            case["id"]
        );
        fixture.assert_requests(&case["id"]);
    }
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

fn discover_input(path: &str) -> crate::entities::discover::DiscoverResult {
    use crate::entities::discover::*;
    let input = asset(path);
    assert_eq!(input["intent"], "General");
    assert!(input["plain_language"].is_null() && input["article_search"].is_null());
    let concepts = input["concepts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| {
            assert_eq!(value["primary_type"], "Drug");
            assert_eq!(value["match_tier"], "Exact");
            assert_eq!(value["confidence"], "CanonicalId");
            assert_eq!(value["xrefs"], json!([]));
            assert_eq!(value["sources"], json!([]));
            DiscoverConcept {
                label: value["label"].as_str().unwrap().to_owned(),
                primary_id: value["primary_id"].as_str().map(str::to_owned),
                primary_type: DiscoverType::Drug,
                synonyms: serde_json::from_value(value["synonyms"].clone()).unwrap(),
                xrefs: vec![],
                sources: vec![],
                match_tier: MatchTier::Exact,
                confidence: DiscoverConfidence::CanonicalId,
            }
        })
        .collect();
    let stats = |value: &Value| DiscoverPreviewStats {
        returned: value["returned"].as_u64().unwrap() as usize,
        total: value["total"].as_u64().unwrap() as usize,
        has_more: value["has_more"].as_bool().unwrap(),
        omitted_oversized: value["omitted_oversized"].as_u64().unwrap() as usize,
    };
    let preview_meta = input["preview_meta"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| DiscoverConceptPreviewMeta {
            label_truncated: value["label_truncated"].as_bool().unwrap(),
            synonyms: stats(&value["synonyms"]),
            xrefs: stats(&value["xrefs"]),
        })
        .collect();
    DiscoverResult {
        query: input["query"].as_str().unwrap().to_owned(),
        normalized_query: input["normalized_query"].as_str().unwrap().to_owned(),
        concepts,
        plain_language: None,
        next_commands: serde_json::from_value(input["next_commands"].clone()).unwrap(),
        notes: serde_json::from_value(input["notes"].clone()).unwrap(),
        ambiguous: input["ambiguous"].as_bool().unwrap(),
        intent: DiscoverIntent::General,
        offset: input["offset"].as_u64().unwrap() as usize,
        limit: input["limit"].as_u64().unwrap() as usize,
        returned: input["returned"].as_u64().unwrap() as usize,
        has_more: input["has_more"].as_bool().unwrap(),
        next_offset: input["next_offset"].as_u64().map(|value| value as usize),
        budget_truncated: input["budget_truncated"].as_bool().unwrap(),
        malformed_candidates: input["malformed_candidates"].as_u64().unwrap() as usize,
        continuation_command: input["continuation_command"].as_str().map(str::to_owned),
        preview_meta,
        full: input["full"].as_bool().unwrap(),
        article_search: None,
    }
}
