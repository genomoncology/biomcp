use biomcp_mcp_contract_client::{
    ContractHarness, article_fulltext_fixture_env, assert_chart_calls,
    assert_explore_core_contract, assert_initialize_and_tools, assert_invalid_resource_error,
    assert_mcp_fulltext_path_redaction, assert_mcp_provenance_calls,
    assert_read_only_and_policy_calls, assert_resource_inventory_and_reads,
    assert_typed_tool_calls, assert_version_call, provision_article_fulltext_fixture,
    provision_study_fixture, start_counting_ols4_stub, start_ols4_stub, study_dir_from_fixture,
    terminate_process,
};
use rmcp::model::CallToolRequestParams;
use serde_json::json;
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;

const FAERS_REPORT_PAGE: &str = r#"{
  "meta":{"results":{"skip":0,"limit":1,"total":1}},
  "results":[{
    "safetyreportid":"1001","serious":"1","receivedate":"20250101",
    "seriousnesshospitalization":"1",
    "patient":{"reaction":[{"reactionmeddrapt":"Rash"}],"drug":[
      {"medicinalproduct":"DRUG NAME","drugcharacterization":"1","drugindication":"LUNG CANCER"},
      {"medicinalproduct":"OTHER DRUG","drugcharacterization":"2"}
    ]}
  }]
}"#;

struct OpenFdaFixture {
    base: String,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

impl OpenFdaFixture {
    fn start() -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind OpenFDA fixture");
        listener
            .set_nonblocking(true)
            .expect("nonblocking OpenFDA fixture");
        let base = format!("http://{}", listener.local_addr().expect("fixture address"));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let thread = thread::spawn(move || {
            while !thread_stop.load(Ordering::SeqCst) {
                let Ok((mut stream, _)) = listener.accept() else {
                    thread::sleep(Duration::from_millis(5));
                    continue;
                };
                let mut request = Vec::new();
                let mut chunk = [0_u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let Ok(read) = stream.read(&mut chunk) else {
                        break;
                    };
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..read]);
                }
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    FAERS_REPORT_PAGE.len(),
                    FAERS_REPORT_PAGE
                );
                let _ = stream.write_all(response.as_bytes());
            }
        });
        Self {
            base,
            stop,
            thread: Some(thread),
        }
    }
}

impl Drop for OpenFdaFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            thread.join().expect("join OpenFDA fixture");
        }
    }
}

fn harness() -> ContractHarness {
    ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"))
}

#[tokio::test(flavor = "multi_thread")]
async fn human_mcp_command_dispatches_to_provider_once() -> anyhow::Result<()> {
    let harness = harness();
    let (_thread, ols_url, requests) = start_counting_ols4_stub()?;
    let (_medline_thread, medline_url) = start_ols4_stub()?;
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_OLS4_BASE", ols_url),
            ("BIOMCP_MEDLINEPLUS_BASE", medline_url),
        ])
        .await?;

    let result = biomcp_mcp_contract_client::call_biomcp(&client, "biomcp discover BRCA1").await?;
    assert_eq!(result.is_error, Some(false));
    assert_eq!(requests.load(Ordering::SeqCst), 1);

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_and_typed_mcp_reject_unknown_adverse_event_sections_before_provider_work()
-> anyhow::Result<()> {
    let harness = harness();
    let client = harness
        .spawn_stdio_client(&[("BIOMCP_OPENFDA_BASE", "http://127.0.0.1:9".to_string())])
        .await?;

    let raw = biomcp_mcp_contract_client::call_biomcp(
        &client,
        "biomcp get adverse-event 1001 not-a-section",
    )
    .await?;
    assert_eq!(raw.is_error, Some(true));
    assert!(biomcp_mcp_contract_client::first_text(&raw.content).contains("Unknown section"));

    let typed = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("adverse-event")),
                    ("id".to_string(), json!("1001")),
                    ("sections".to_string(), json!(["not-a-section"])),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await
        .expect_err("typed MCP rejects an unknown section at its schema boundary");
    assert!(typed.to_string().contains("invalid adverse-event section"));

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_mcp_rejects_contradictory_variant_filters_and_non_trial_batch_source()
-> anyhow::Result<()> {
    let harness = harness();
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_MYVARIANT_BASE", "http://127.0.0.1:9".to_string()),
            ("BIOMCP_MYGENE_BASE", "http://127.0.0.1:9".to_string()),
        ])
        .await?;

    let variant = biomcp_mcp_contract_client::call_biomcp(
        &client,
        "biomcp search variant --min-cadd 10 --missing cadd",
    )
    .await?;
    assert_eq!(variant.is_error, Some(true));
    assert!(
        biomcp_mcp_contract_client::first_text(&variant.content)
            .contains("cannot be combined with --missing")
    );

    let batch =
        biomcp_mcp_contract_client::call_biomcp(&client, "biomcp batch gene BRAF --source ctgov")
            .await?;
    assert_eq!(batch.is_error, Some(true));
    assert!(
        biomcp_mcp_contract_client::first_text(&batch.content)
            .contains("--source is only supported for trial batches")
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn raw_and_typed_mcp_preserve_adverse_event_subset_and_full_json_contracts()
-> anyhow::Result<()> {
    let harness = harness();
    let fixture = OpenFdaFixture::start();
    let cache = tempfile::tempdir()?;
    let client = harness
        .spawn_stdio_client(&[
            ("BIOMCP_OPENFDA_BASE", fixture.base.clone()),
            (
                "BIOMCP_CACHE_DIR",
                cache.path().to_string_lossy().into_owned(),
            ),
        ])
        .await?;

    let raw_subset = biomcp_mcp_contract_client::call_biomcp_json(
        &client,
        "biomcp get adverse-event 1001 reactions reactions guidance guidance",
    )
    .await?;
    assert_eq!(raw_subset.is_error, Some(false));
    let raw_subset: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&raw_subset.content))?;

    let typed_subset = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("adverse-event")),
                    ("id".to_string(), json!("1002")),
                    (
                        "sections".to_string(),
                        json!(["guidance", "reactions", "guidance", "reactions"]),
                    ),
                    ("json".to_string(), json!(true)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    assert_eq!(typed_subset.is_error, Some(false));
    let typed_subset: serde_json::Value = serde_json::from_str(
        biomcp_mcp_contract_client::first_text(&typed_subset.content),
    )?;
    assert_eq!(typed_subset, raw_subset);
    assert!(typed_subset["data"].get("reactions").is_some());
    assert!(typed_subset["data"].get("outcomes").is_none());
    assert!(typed_subset["data"].get("patient").is_none());
    assert_eq!(
        typed_subset["_meta"]["next_commands"]
            .as_array()
            .expect("guidance commands")
            .len(),
        4
    );

    let raw_full =
        biomcp_mcp_contract_client::call_biomcp_json(&client, "biomcp get adverse-event 1003")
            .await?;
    assert_eq!(raw_full.is_error, Some(false));
    let raw_full: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&raw_full.content))?;

    let typed_all = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("adverse-event")),
                    ("id".to_string(), json!("1004")),
                    ("sections".to_string(), json!(["all"])),
                    ("json".to_string(), json!(true)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    assert_eq!(typed_all.is_error, Some(false));
    let typed_all: serde_json::Value =
        serde_json::from_str(biomcp_mcp_contract_client::first_text(&typed_all.content))?;
    assert_eq!(typed_all, raw_full);
    for key in ["indication", "serious", "date"] {
        assert!(
            typed_all["data"].get(key).is_some(),
            "missing {key}: {typed_all}"
        );
    }
    assert!(
        typed_all["_meta"]["section_sources"]
            .as_array()
            .is_some_and(|sources| sources.iter().any(|source| source["key"] == "overview"))
    );

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_core_contract() -> anyhow::Result<()> {
    let harness = harness();
    let client = harness.spawn_stdio_client(&[]).await?;
    assert_explore_core_contract(&client).await?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live external-service full contract; run through make verify"]
async fn rmcp_child_process_client_verifies_stdio_full_contract() -> anyhow::Result<()> {
    let harness = harness();
    let (_ols_thread, ols_url) = start_ols4_stub()?;
    let (client, pid) = harness
        .spawn_stdio_client_with_pid(&[
            ("BIOMCP_OLS4_BASE", ols_url.clone()),
            ("BIOMCP_MEDLINEPLUS_BASE", ols_url),
        ])
        .await?;

    assert_initialize_and_tools(&client, &harness.repo_root).await?;
    assert_version_call(&client).await?;
    assert_resource_inventory_and_reads(&client, &harness.repo_root).await?;
    assert_read_only_and_policy_calls(&client).await?;
    assert_mcp_provenance_calls(&client).await?;
    assert_typed_tool_calls(&client).await?;
    assert_invalid_resource_error(&client).await?;

    terminate_process(pid)?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_redacts_fulltext_paths_from_text_and_json() -> anyhow::Result<()> {
    let harness = harness();
    let fixture = provision_article_fulltext_fixture(&harness.repo_root)?;
    let env = article_fulltext_fixture_env(&fixture);
    let client = harness.spawn_stdio_client(&env).await?;

    assert_mcp_fulltext_path_redaction(&client, &fixture).await?;

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_redacts_fulltext_paths_from_text_and_json()
-> anyhow::Result<()> {
    let harness = harness();
    let fixture = provision_article_fulltext_fixture(&harness.repo_root)?;
    let env = article_fulltext_fixture_env(&fixture);
    let (mut child, base_url) = harness.spawn_http_server(&env).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_mcp_fulltext_path_redaction(&client, &fixture).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_chart_contract() -> anyhow::Result<()> {
    let harness = harness();
    let fixture_root = provision_study_fixture(&harness.repo_root)?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let client = harness
        .spawn_stdio_client(&[("BIOMCP_STUDY_DIR", study_dir)])
        .await?;

    assert_chart_calls(&client).await?;

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_verifies_core_contract() -> anyhow::Result<()> {
    let harness = harness();
    let (mut child, base_url) = harness.spawn_http_server(&[]).await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_explore_core_contract(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live external-service full contract; run through make verify"]
async fn rmcp_streamable_http_client_verifies_full_contract() -> anyhow::Result<()> {
    let harness = harness();
    let (_ols_thread, ols_url) = start_ols4_stub()?;
    let (mut child, base_url) = harness
        .spawn_http_server(&[
            ("BIOMCP_OLS4_BASE", ols_url.clone()),
            ("BIOMCP_MEDLINEPLUS_BASE", ols_url),
        ])
        .await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_initialize_and_tools(&client, &harness.repo_root).await?;
        assert_version_call(&client).await?;
        assert_resource_inventory_and_reads(&client, &harness.repo_root).await?;
        assert_read_only_and_policy_calls(&client).await?;
        assert_mcp_provenance_calls(&client).await?;
        assert_typed_tool_calls(&client).await?;
        assert_invalid_resource_error(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_verifies_chart_contract() -> anyhow::Result<()> {
    let harness = harness();
    let fixture_root = provision_study_fixture(&harness.repo_root)?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let (mut child, base_url) = harness
        .spawn_http_server(&[("BIOMCP_STUDY_DIR", study_dir)])
        .await?;
    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_chart_calls(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}
