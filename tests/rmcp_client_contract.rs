use biomcp_mcp_contract_client::{
    ContractHarness, article_fulltext_fixture_env, assert_chart_calls,
    assert_explore_core_contract, assert_initialize_and_tools, assert_invalid_resource_error,
    assert_mcp_fulltext_path_redaction, assert_mcp_provenance_calls,
    assert_read_only_and_policy_calls, assert_resource_inventory_and_reads,
    assert_typed_tool_calls, assert_version_call, provision_article_fulltext_fixture,
    provision_study_fixture, start_ols4_stub, study_dir_from_fixture, terminate_process,
};

fn harness() -> ContractHarness {
    ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"))
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
