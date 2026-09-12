use super::*;
use axum::{Router, body::Body, http::Response, routing::get};
use serde::Deserialize;
use serde_json::{Value, json, value::RawValue};
use sha2::{Digest, Sha256};

struct SearchFixtureEnv {
    previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    _cache: tempfile::TempDir,
}

impl SearchFixtureEnv {
    fn set(provider: &str, base: &str) -> Self {
        let cache = tempfile::tempdir().expect("search fixture cache");
        let mut values = vec![
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
            ("BIOMCP_TEST_UNPACED_ORIGIN", base.to_owned()),
        ];
        if provider == "ctgov" {
            values.push(("BIOMCP_CTGOV_BASE", base.to_owned()));
        } else {
            values.push(("BIOMCP_NCI_CTS_BASE", base.to_owned()));
            values.push(("NCI_API_KEY", "NCI-CREDENTIAL-SENTINEL-0117".to_owned()));
        }
        let previous = values
            .into_iter()
            .map(|(name, value)| {
                let old = std::env::var_os(name);
                // SAFETY: every caller holds the serial-test environment lock.
                unsafe { std::env::set_var(name, value) };
                (name, old)
            })
            .collect();
        Self {
            previous,
            _cache: cache,
        }
    }
}

impl Drop for SearchFixtureEnv {
    fn drop(&mut self) {
        // SAFETY: every caller holds the serial-test environment lock.
        unsafe {
            for (name, old) in self.previous.drain(..) {
                if let Some(old) = old {
                    std::env::set_var(name, old);
                } else {
                    std::env::remove_var(name);
                }
            }
        }
    }
}

async fn fixture_server(body: &'static [u8]) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind trial search fixture");
    let base = format!("http://{}", listener.local_addr().expect("fixture address"));
    let router = Router::new()
        .route("/studies", get(move || async move { exact_response(body) }))
        .route("/trials", get(move || async move { exact_response(body) }));
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("serve trial search fixture");
    });
    (base, task)
}

fn exact_response(body: &'static [u8]) -> Response<Body> {
    Response::builder()
        .header("content-type", "application/json")
        .header("content-length", body.len())
        .body(Body::from(body))
        .expect("fixture response")
}

fn mcp_text(result: rmcp::model::CallToolResult) -> String {
    serde_json::to_value(result).expect("serialize MCP result")["content"][0]["text"]
        .as_str()
        .expect("MCP text content")
        .to_owned()
}

fn first_raw_digest(body: &[u8], collection: &str) -> String {
    #[derive(Deserialize)]
    struct RawRows {
        #[serde(default)]
        studies: Vec<Box<RawValue>>,
        #[serde(default)]
        data: Vec<Box<RawValue>>,
    }
    let envelope: RawRows = serde_json::from_slice(body).expect("fixture JSON");
    let first = if collection == "studies" {
        &envelope.studies[0]
    } else {
        &envelope.data[0]
    };
    format!("sha256:{:x}", Sha256::digest(first.get().as_bytes()))
}

fn fixture_search_filters() -> biodata::ClinicalTrialSearchFilters {
    biodata::ClinicalTrialSearchFilters::new(
        biodata::ClinicalTrialSearchFilterFields {
            condition: Some("fixture".into()),
            ..Default::default()
        },
        Default::default(),
    )
    .expect("fixture filters")
}

async fn assert_search_surfaces(provider: &str, body: &'static [u8], collection: &str) {
    let (base, server) = fixture_server(body).await;
    let _env = SearchFixtureEnv::set(provider, &base);
    let raw_command = if provider == "ctgov" {
        "biomcp search trial --source ctgov --condition SURFACE-CONDITION-SENTINEL-0117 --limit 1"
    } else {
        "biomcp search trial --source nci --mutation BRAF --phase 3 --limit 1"
    };
    let raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: raw_command.into(),
            json: true,
        }))
        .await
        .expect("raw MCP search");
    let raw_markdown = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: raw_command.into(),
            json: false,
        }))
        .await
        .expect("raw MCP Markdown search");
    let typed_input = if provider == "ctgov" {
        json!({"entity":"trial", "source":"ctgov", "condition":"SURFACE-CONDITION-SENTINEL-0117", "limit":1, "json":true})
    } else {
        json!({"entity":"trial", "source":"nci", "mutation":["BRAF"], "phase":"3", "limit":1, "json":true})
    };
    let typed = BioMcpServer::new()
        .search(rmcp::handler::server::wrapper::Parameters(TypedSearch(
            typed_input,
        )))
        .await
        .expect("typed MCP search");
    server.abort();

    let raw_value: Value = serde_json::from_str(&mcp_text(raw)).expect("raw MCP JSON");
    let typed_value: Value = serde_json::from_str(&mcp_text(typed)).expect("typed MCP JSON");
    assert_eq!(raw_value, typed_value);
    assert_eq!(raw_value["count"], 1);
    assert_eq!(raw_value["pagination"]["total"], 1);
    assert_eq!(raw_value["pagination"]["total_precision"], "exact");
    assert!(raw_value["pagination"].get("total_reason").is_none());
    assert_eq!(raw_value["pagination"]["continuation_status"], "terminal");
    assert_eq!(raw_value["pagination"]["has_more"], false);
    assert_eq!(raw_value["pagination"]["next_page_token"], Value::Null);
    assert!(raw_value["pagination"].get("next_offset").is_none());
    assert!(raw_value["pagination"].get("continuation_reason").is_none());
    let markdown = mcp_text(raw_markdown);
    let nct_id = raw_value["results"][0]["nct_id"]
        .as_str()
        .expect("result identity");
    assert!(markdown.contains(nct_id));
    assert!(markdown.contains("Total: 1."));
    assert!(!markdown.contains("Use --offset"));
    assert!(!markdown.contains("Use --next-page"));

    let page_digest = if provider == "ctgov" {
        let filters = fixture_search_filters();
        let plan = biodata::ClinicalTrialsGovApiV2SearchPlan::new(&filters, 1, None, true).unwrap();
        let page = crate::sources::clinicaltrials::ClinicalTrialsClient::decode_search_response(
            &plan,
            reqwest::StatusCode::OK,
            body,
        )
        .expect("CTGov fixture page");
        page.results().expect("CTGov rows")[0]
            .projection()
            .capture()
            .digest()
            .to_owned()
    } else {
        let filters = fixture_search_filters();
        let plan = biodata::NciCtsV2SearchPlan::new(&filters, None, 1, 0).unwrap();
        let page = crate::sources::nci_cts::NciCtsClient::decode_search_response(
            &plan,
            reqwest::StatusCode::OK,
            body,
        )
        .expect("NCI fixture page");
        page.results().expect("NCI rows")[0]
            .projection()
            .capture()
            .digest()
            .to_owned()
    };
    assert_eq!(page_digest, first_raw_digest(body, collection));
    let public = format!("{markdown}{}{raw_value}{typed_value}", provider);
    assert!(!public.contains("NCI-CREDENTIAL-SENTINEL-0117"));
}

struct ExpectedTrialState<'a> {
    total_precision: &'a str,
    total_reason: Option<&'a str>,
    continuation_status: &'a str,
    continuation_reason: Option<&'a str>,
}

async fn assert_trial_state_surfaces(
    provider: &str,
    body: &'static [u8],
    raw_command: &str,
    typed_input: Value,
    expected: ExpectedTrialState<'_>,
) {
    let (base, server) = fixture_server(body).await;
    let _env = SearchFixtureEnv::set(provider, &base);
    let raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: raw_command.into(),
            json: true,
        }))
        .await
        .unwrap();
    let markdown = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: raw_command.into(),
            json: false,
        }))
        .await
        .unwrap();
    let typed = BioMcpServer::new()
        .search(rmcp::handler::server::wrapper::Parameters(TypedSearch(
            typed_input,
        )))
        .await
        .unwrap();
    server.abort();

    let raw: Value = serde_json::from_str(&mcp_text(raw)).unwrap();
    let typed: Value = serde_json::from_str(&mcp_text(typed)).unwrap();
    assert_eq!(raw, typed);
    let pagination = &raw["pagination"];
    assert_eq!(pagination["total_precision"], expected.total_precision);
    assert_eq!(
        pagination.get("total_reason").and_then(Value::as_str),
        expected.total_reason
    );
    assert_eq!(
        pagination["continuation_status"],
        expected.continuation_status
    );
    assert_eq!(
        pagination
            .get("continuation_reason")
            .and_then(Value::as_str),
        expected.continuation_reason
    );
    assert_eq!(
        pagination["has_more"],
        matches!(expected.continuation_status, "cursor" | "offset")
    );
    let markdown = mcp_text(markdown);
    if expected.total_precision == "approximate" {
        assert!(markdown.contains("approximate"));
    }
    if expected.total_precision == "unknown" {
        assert!(markdown.contains("Total: unknown"));
    }
    match expected.continuation_status {
        "cursor" => assert!(markdown.contains("Use --next-page")),
        "offset" => assert!(markdown.contains("Use --offset")),
        "unavailable" | "terminal" => {
            assert!(!markdown.contains("Use --next-page"));
            assert!(!markdown.contains("Use --offset"));
        }
        _ => unreachable!(),
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ctgov_search_cross_surface_uses_exact_result_capture() {
    assert_search_surfaces(
        "ctgov",
        br#"{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT41300001","briefTitle":"Local CTGov search result"},"statusModule":{"overallStatus":"RECRUITING"}}}],"totalCount":1}"#,
        "studies",
    )
    .await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_search_cross_surface_uses_exact_result_capture() {
    assert_search_surfaces(
        "nci",
        br#"{"data":[{"nct_id":"NCT05929768","brief_title":"Local NCI search result","current_trial_status":"Active","phase":"III","diseases":[{"name":"Melanoma"}],"lead_org":"NCI fixture sponsor"}],"total":1}"#,
        "data",
    )
    .await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ctgov_approximate_cursor_state_crosses_cli_raw_and_typed_surfaces() {
    assert_trial_state_surfaces(
        "ctgov",
        br#"{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT41300001","briefTitle":"Approximate"},"statusModule":{"overallStatus":"RECRUITING"},"eligibilityModule":{"minimumAge":"18 Years","maximumAge":"75 Years"}}}],"totalCount":5,"nextPageToken":"page-two"}"#,
        "biomcp search trial --condition melanoma --age 50 --limit 1",
        json!({"entity":"trial","condition":["melanoma"],"age":50,"limit":1,"json":true}),
        ExpectedTrialState {
            total_precision: "approximate",
            total_reason: Some("before_local_filtering"),
            continuation_status: "cursor",
            continuation_reason: None,
        },
    )
    .await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_unknown_offset_state_crosses_cli_raw_and_typed_surfaces() {
    assert_trial_state_surfaces(
        "nci",
        br#"{"data":[{"nct_id":"NCT05929768","brief_title":"Missing total","current_trial_status":"Active","phase":"III"}]}"#,
        "biomcp search trial --source nci --mutation BRAF --limit 1",
        json!({"entity":"trial","source":"nci","mutation":["BRAF"],"limit":1,"json":true}),
        ExpectedTrialState {
            total_precision: "unknown",
            total_reason: Some("provider_omitted_total"),
            continuation_status: "offset",
            continuation_reason: None,
        },
    )
    .await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ctgov_unknown_unavailable_state_crosses_cli_raw_and_typed_surfaces() {
    assert_trial_state_surfaces(
        "ctgov",
        br#"{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT41300001","briefTitle":"Unavailable"},"statusModule":{"overallStatus":"RECRUITING"}}}],"nextPageToken":"  "}"#,
        "biomcp search trial --condition melanoma --limit 1",
        json!({"entity":"trial","condition":["melanoma"],"limit":1,"json":true}),
        ExpectedTrialState {
            total_precision: "unknown",
            total_reason: Some("provider_omitted_total"),
            continuation_status: "unavailable",
            continuation_reason: Some("unusable_provider_cursor"),
        },
    )
    .await;
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn typed_trial_count_matches_raw_count_output() {
    let (base, server) = fixture_server(br#"{"studies":[],"totalCount":7}"#).await;
    let _env = SearchFixtureEnv::set("ctgov", &base);
    let raw = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp search trial --condition melanoma --count-only".into(),
            json: true,
        }))
        .await
        .unwrap();
    let typed = BioMcpServer::new()
        .search(rmcp::handler::server::wrapper::Parameters(TypedSearch(
            json!({
                "entity":"trial", "condition":["melanoma"], "count_only":true, "json":true
            }),
        )))
        .await
        .unwrap();
    server.abort();
    let raw: Value = serde_json::from_str(&mcp_text(raw)).unwrap();
    let typed: Value = serde_json::from_str(&mcp_text(typed)).unwrap();
    assert_eq!(raw, typed);
    assert_eq!(typed["total"], 7);
    assert_eq!(typed["total_precision"], "exact");
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ctgov_source_client_transport_preserves_exact_result_digest() {
    const BODY: &[u8] = br#"{"studies":[ { "protocolSection":{"identificationModule":{"nctId":"NCT41300001","briefTitle":"Transport CTGov result"},"statusModule":{"overallStatus":"RECRUITING"}} }],"totalCount":1}"#;
    let (base, server) = fixture_server(BODY).await;
    let _env = SearchFixtureEnv::set("ctgov", &base);
    let page = crate::sources::clinicaltrials::ClinicalTrialsClient::new()
        .expect("CTGov client")
        .search(
            &biodata::ClinicalTrialsGovApiV2SearchPlan::new(
                &biodata::ClinicalTrialSearchFilters::new(
                    biodata::ClinicalTrialSearchFilterFields {
                        condition: Some("transport condition".into()),
                        ..Default::default()
                    },
                    Default::default(),
                )
                .expect("filters"),
                1,
                None,
                false,
            )
            .expect("plan"),
        )
        .await
        .expect("transported CTGov page");
    server.abort();
    assert_eq!(
        page.results().expect("CTGov result")[0]
            .projection()
            .capture()
            .digest(),
        first_raw_digest(BODY, "studies")
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn nci_source_client_transport_preserves_exact_result_digest() {
    const BODY: &[u8] = br#"{"data":[ { "nct_id":"NCT05929768","brief_title":"Transport NCI result","current_trial_status":"Active","diseases":[{"name":"Melanoma"}] }],"total":1}"#;
    let (base, server) = fixture_server(BODY).await;
    let _env = SearchFixtureEnv::set("nci", &base);
    let page = crate::sources::nci_cts::NciCtsClient::new()
        .expect("NCI client")
        .search(
            &biodata::NciCtsV2SearchPlan::new(
                &biodata::ClinicalTrialSearchFilters::new(
                    biodata::ClinicalTrialSearchFilterFields {
                        biomarker: Some("BRAF".into()),
                        ..Default::default()
                    },
                    Default::default(),
                )
                .expect("filters"),
                None,
                1,
                0,
            )
            .expect("plan"),
        )
        .await
        .expect("transported NCI page");
    server.abort();
    assert_eq!(
        page.results().expect("NCI result")[0]
            .projection()
            .capture()
            .digest(),
        first_raw_digest(BODY, "data")
    );
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn ticket_0118_refusals_are_safe_across_raw_typed_and_markdown_search() {
    const BODY: &[u8] = br#"{"studies":[{"protocolSection":{"identificationModule":{"nctId":"NCT41300018","briefTitle":"Typed 0118 result"},"statusModule":{"overallStatus":"RECRUITING"}}}],"totalCount":1}"#;
    let (base, server) = fixture_server(BODY).await;
    let _env = SearchFixtureEnv::set("ctgov", &base);
    let command = "biomcp search trial --status active --limit 1";
    let raw_json = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: command.into(),
            json: true,
        }))
        .await
        .expect("raw MCP JSON refusal");
    let raw_markdown = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: command.into(),
            json: false,
        }))
        .await
        .expect("raw MCP Markdown refusal");
    let typed = BioMcpServer::new()
        .search(rmcp::handler::server::wrapper::Parameters(TypedSearch(
            json!({"entity":"trial", "source":"ctgov", "condition":["melanoma"], "limit":1, "json":true}),
        )))
        .await
        .expect("typed MCP search");
    server.abort();

    for result in [raw_json, raw_markdown] {
        let public = mcp_text(result);
        assert!(public.contains("recruiting"));
        assert!(public.contains("active_not_recruiting"));
        assert!(!public.contains("status: active"));
    }
    let typed_public = mcp_text(typed);
    assert!(typed_public.contains("NCT41300018"));
}
