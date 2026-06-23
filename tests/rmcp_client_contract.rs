use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::Stdio;
use std::thread;
use std::time::Duration;

use base64::Engine;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, RawContent, ReadResourceRequestParams, ResourceContents};
use rmcp::service::ServiceError;
use rmcp::transport::{StreamableHttpClientTransport, TokioChildProcess};
use serde_json::json;
use tokio::process::{Child, Command};

const EXPECTED_HELP_RESOURCE: (&str, &str) = ("biomcp://help", "BioMCP Overview");
const READ_ONLY_MESSAGE: &str = "BioMCP allows read-only commands only";
const CACHE_CLI_ONLY_MESSAGE: &str = "CLI-only over MCP";
const CACHE_FILESYSTEM_MESSAGE: &str = "workstation-local filesystem paths";

fn biomcp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_biomcp").unwrap_or_else(|_| "target/debug/biomcp".to_string())
}

fn text_chunks(content: &[rmcp::model::Content]) -> Vec<&str> {
    content
        .iter()
        .filter_map(|chunk| match &chunk.raw {
            RawContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect()
}

fn image_chunks(content: &[rmcp::model::Content]) -> Vec<(&str, &str)> {
    content
        .iter()
        .filter_map(|chunk| match &chunk.raw {
            RawContent::Image(image) => Some((image.mime_type.as_str(), image.data.as_str())),
            _ => None,
        })
        .collect()
}

fn first_text(content: &[rmcp::model::Content]) -> &str {
    text_chunks(content)
        .into_iter()
        .next()
        .expect("result returned a text content chunk")
}

fn tool_arguments(command: &str) -> serde_json::Map<String, serde_json::Value> {
    BTreeMap::from([("command".to_string(), json!(command))])
        .into_iter()
        .collect()
}

async fn call_biomcp<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
    command: &str,
) -> anyhow::Result<rmcp::model::CallToolResult>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    Ok(client
        .peer()
        .call_tool(CallToolRequestParams::new("biomcp").with_arguments(tool_arguments(command)))
        .await?)
}

fn expected_skill_resources() -> anyhow::Result<Vec<(String, String)>> {
    let mut resources = Vec::new();
    let skills_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/use-cases");
    let mut paths = std::fs::read_dir(skills_dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    for path in paths {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.ends_with(".md") || file_name.len() < 6 {
            continue;
        }
        if !file_name.as_bytes()[0].is_ascii_digit()
            || !file_name.as_bytes()[1].is_ascii_digit()
            || file_name.as_bytes()[2] != b'-'
        {
            continue;
        }
        let body = std::fs::read_to_string(&path)?;
        let title = body
            .lines()
            .find_map(|line| line.strip_prefix("# "))
            .expect("skill file has a title")
            .trim();
        let name = if title.to_ascii_lowercase().starts_with("pattern:") {
            title.to_string()
        } else {
            format!("Pattern: {title}")
        };
        let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap();
        resources.push((format!("biomcp://skill/{}", &stem[3..]), name));
    }
    Ok(resources)
}

async fn assert_initialize_and_tools<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let initialize = client
        .peer()
        .peer_info()
        .expect("rmcp client stores initialize result as peer info");
    assert!(initialize.capabilities.tools.is_some());
    assert!(initialize.capabilities.resources.is_some());
    let instructions = initialize.instructions.as_deref().unwrap_or_default();
    assert!(instructions.contains("leading public biomedical data sources"));
    assert!(!instructions.contains("15 sources"));
    assert!(!instructions.contains("15 biomedical sources"));
    assert!(!instructions.contains("biomcp skill list"));
    assert!(instructions.contains("biomcp suggest \"<question>\""));
    assert!(instructions.contains("biomcp skill"));

    let tools = client.peer().list_tools(Default::default()).await?;
    let tool_names = tools
        .tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"biomcp"));
    assert!(!tool_names.contains(&"shell"));
    let biomcp = tools
        .tools
        .iter()
        .find(|tool| tool.name == "biomcp")
        .expect("biomcp tool listed");
    let annotations = biomcp.annotations.as_ref().expect("biomcp annotations");
    assert_eq!(annotations.title.as_deref(), Some("BioMCP"));
    assert_eq!(annotations.read_only_hint, Some(true));

    let description = biomcp
        .description
        .as_deref()
        .expect("biomcp tool description");
    assert!(description.to_ascii_lowercase().contains("read-only"));
    let list_contract = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/list_reference.md"),
    )?;
    let required = [
        "BioMCP Command Reference",
        "search <entity> [query|filters]",
        "search trial [filters]",
        "get <entity> <id> [section...]",
        "suggest \"What drugs treat melanoma?\"",
        "search phenotype \"seizure, developmental delay\"",
    ];
    let article_markers = [
        "Turn a literature question into article filters",
        "known gene/disease/drug anchors go in `-g/-d/--drug`; free-text concepts go in `-k`",
        "PubMed ESearch cleans question-format terms provider-locally",
    ];
    let article_details = [
        "## Query formulation",
        "apoptosis gene regulation",
        "photosensitivity mechanism",
        "TCGA mutation analysis dataset",
    ];
    for marker in required {
        assert!(
            list_contract.contains(marker),
            "list contract missing {marker}"
        );
        assert!(
            description.contains(marker),
            "tool description missing {marker}"
        );
    }
    for marker in article_markers {
        assert!(
            list_contract.contains(marker),
            "list contract missing {marker}"
        );
        assert!(
            description.contains(marker),
            "tool description missing {marker}"
        );
    }
    for detail in article_details {
        assert!(
            !list_contract.contains(detail),
            "list contract leaked {detail}"
        );
        assert!(
            !description.contains(detail),
            "tool description leaked {detail}"
        );
    }
    assert!(description.contains("leading public biomedical data sources"));
    assert!(!description.contains("15 biomedical sources"));
    assert!(description.contains("SEARCH FILTERS:"));
    assert!(description.contains("AGENT GUIDANCE:"));
    assert!(description.contains("biomcp list"));
    for forbidden in [
        "ema sync",
        "who sync",
        "cvx sync",
        "gtr sync",
        "who-ivd sync",
        "skill install",
        "study download <study_id>",
        "study download [--list] [<study_id>]",
        "cache path",
        "cache stats",
        "cache clean",
        "cache clear",
        "uninstall",
    ] {
        assert!(
            !description.contains(forbidden),
            "tool description leaked {forbidden}"
        );
    }
    assert!(description.contains("study download --list"));
    assert!(
        !description
            .lines()
            .any(|line| line.trim().starts_with("- `update "))
    );

    Ok(())
}

async fn assert_version_call<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let call = call_biomcp(client, "biomcp version").await?;
    assert_ne!(call.is_error, Some(true));
    let text = text_chunks(&call.content);
    assert!(!text.is_empty(), "call_tool returned no text chunks");
    assert_eq!(
        image_chunks(&call.content).len(),
        0,
        "version should not return images"
    );
    assert!(
        text.iter()
            .any(|chunk| chunk.to_ascii_lowercase().contains("biomcp") || chunk.contains("0.8.")),
        "call_tool text did not include a version marker: {text:?}"
    );
    Ok(())
}

async fn assert_explore_core_contract<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let initialize = client
        .peer()
        .peer_info()
        .expect("rmcp client stores initialize result as peer info");
    assert!(initialize.capabilities.tools.is_some());
    assert!(initialize.capabilities.resources.is_some());
    let instructions = initialize.instructions.as_deref().unwrap_or_default();
    assert!(instructions.contains("leading public biomedical data sources"));
    assert!(instructions.contains("biomcp suggest \"<question>\""));
    assert!(!instructions.contains("15 sources"));
    assert!(!instructions.contains("biomcp skill list"));

    let tools = client.peer().list_tools(Default::default()).await?;
    let names = tools
        .tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();
    assert!(names.contains(&"biomcp"));
    assert!(!names.contains(&"shell"));
    let biomcp = tools
        .tools
        .iter()
        .find(|tool| tool.name == "biomcp")
        .expect("biomcp tool listed");
    let annotations = biomcp.annotations.as_ref().expect("biomcp annotations");
    assert_eq!(annotations.title.as_deref(), Some("BioMCP"));
    assert_eq!(annotations.read_only_hint, Some(true));

    assert_version_call(client).await?;

    let resources = client.peer().list_resources(Default::default()).await?;
    assert!(
        resources
            .resources
            .iter()
            .any(|resource| resource.uri == EXPECTED_HELP_RESOURCE.0),
        "help resource was not listed: {:?}",
        resources.resources
    );
    let help = client
        .peer()
        .read_resource(ReadResourceRequestParams::new(EXPECTED_HELP_RESOURCE.0))
        .await?;
    let help_text = help.contents.iter().find_map(|content| match content {
        ResourceContents::TextResourceContents {
            uri,
            mime_type,
            text,
            ..
        } if uri == EXPECTED_HELP_RESOURCE.0 && mime_type.as_deref() == Some("text/markdown") => {
            Some(text.as_str())
        }
        _ => None,
    });
    assert!(
        help_text.is_some_and(|text| text.contains("## Routing rules")),
        "help resource returned markdown text with routing rules"
    );
    Ok(())
}

async fn assert_resource_inventory_and_reads<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let resources = client.peer().list_resources(Default::default()).await?;
    let actual = resources
        .resources
        .iter()
        .map(|resource| (resource.uri.to_string(), resource.name.to_string()))
        .collect::<Vec<_>>();
    let mut expected = vec![(
        EXPECTED_HELP_RESOURCE.0.to_string(),
        EXPECTED_HELP_RESOURCE.1.to_string(),
    )];
    expected.extend(expected_skill_resources()?);
    assert_eq!(actual, expected);

    for (uri, _) in actual {
        let result = client
            .peer()
            .read_resource(ReadResourceRequestParams::new(uri.clone()))
            .await?;
        assert!(!result.contents.is_empty(), "{uri} returned no content");
        let mut found_text = false;
        for content in result.contents {
            if let ResourceContents::TextResourceContents {
                uri: content_uri,
                mime_type,
                text,
                ..
            } = content
            {
                found_text = true;
                assert_eq!(content_uri, uri);
                assert_eq!(mime_type.as_deref(), Some("text/markdown"));
                assert!(!text.trim().is_empty());
                if uri == EXPECTED_HELP_RESOURCE.0 {
                    assert!(text.contains("## Routing rules"));
                    assert!(text.contains("## How-to reference"));
                    assert!(!text.contains("../docs/"));
                    assert!(!text.contains(".md)"));
                }
            }
        }
        assert!(found_text, "{uri} did not return markdown text");
    }
    Ok(())
}

async fn assert_read_only_and_policy_calls<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let render = call_biomcp(client, "biomcp skill render").await?;
    assert_eq!(render.is_error, Some(false));
    let help_resource = client
        .peer()
        .read_resource(ReadResourceRequestParams::new(EXPECTED_HELP_RESOURCE.0))
        .await?;
    let help_text = help_resource
        .contents
        .iter()
        .find_map(|content| match content {
            ResourceContents::TextResourceContents { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .expect("help resource text");
    assert_eq!(first_text(&render.content), help_text);

    let skill_list = call_biomcp(client, "biomcp skill list").await?;
    assert_eq!(skill_list.is_error, Some(false));
    assert!(first_text(&skill_list.content).contains("# BioMCP Worked Examples"));

    let discover = call_biomcp(client, "biomcp discover BRCA1").await?;
    assert_eq!(discover.is_error, Some(false));
    assert!(first_text(&discover.content).contains("BRCA1"));

    let suggest = call_biomcp(client, "biomcp suggest \"What drugs treat melanoma?\"").await?;
    assert_eq!(suggest.is_error, Some(false));
    let suggest_text = first_text(&suggest.content);
    assert!(suggest_text.contains("treatment-lookup"));
    assert!(suggest_text.contains("biomcp skill treatment-lookup"));

    for command in [
        "biomcp skill sync",
        "biomcp skill install /tmp/biomcp-skills",
        "biomcp study download msk_impact_2017",
        "biomcp gtr sync",
        "biomcp who-ivd sync",
    ] {
        let result = call_biomcp(client, command).await?;
        assert_eq!(result.is_error, Some(true), "{command} should be rejected");
        assert!(first_text(&result.content).contains(READ_ONLY_MESSAGE));
    }

    for command in [
        "biomcp cache path",
        "biomcp cache stats",
        "biomcp cache clean",
        "biomcp cache clear",
    ] {
        let result = call_biomcp(client, command).await?;
        assert_eq!(result.is_error, Some(true), "{command} should be rejected");
        let text = first_text(&result.content);
        assert!(text.contains(CACHE_CLI_ONLY_MESSAGE));
        assert!(text.contains(CACHE_FILESYSTEM_MESSAGE));
    }

    Ok(())
}

async fn assert_invalid_resource_error<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let error = client
        .peer()
        .read_resource(ReadResourceRequestParams::new(
            "biomcp://skill/not-a-real-resource",
        ))
        .await
        .expect_err("invalid resource should return an MCP error");
    match error {
        ServiceError::McpError(data) => {
            assert_eq!(data.code.0, -32002);
            assert!(data.message.contains("Unknown resource:"));
        }
        other => panic!("expected MCP error for invalid resource, got {other:?}"),
    }
    Ok(())
}

async fn assert_chart_calls<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let chart = call_biomcp(
        client,
        "biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar",
    )
    .await?;
    assert_eq!(chart.is_error, Some(false));
    assert_eq!(chart.content.len(), 2);
    let text = first_text(&chart.content);
    assert!(text.contains("# Study Mutation Frequency: TP53 (msk_impact_2017)"));
    let images = image_chunks(&chart.content);
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].0, "image/svg+xml");
    let svg = base64::engine::general_purpose::STANDARD.decode(images[0].1)?;
    let svg = String::from_utf8(svg)?;
    assert!(svg.trim_start().starts_with("<svg") || svg.trim_start().starts_with("<?xml"));
    assert!(svg.contains("<svg"));

    let rejected = call_biomcp(
        client,
        "biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar --output out.svg",
    )
    .await?;
    assert_eq!(rejected.is_error, Some(true));
    assert!(first_text(&rejected.content).contains("MCP chart responses do not support --output"));
    Ok(())
}

fn provision_study_fixture() -> anyhow::Result<tempfile::TempDir> {
    let temp = tempfile::Builder::new()
        .prefix("biomcp-study-rmcp-tests-")
        .tempdir()?;
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("spec/fixtures/setup-study-spec-fixture.sh");
    let status = std::process::Command::new("bash")
        .arg(script)
        .arg(temp.path())
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()?;
    anyhow::ensure!(status.success(), "study fixture setup failed: {status}");
    Ok(temp)
}

fn study_dir_from_fixture(root: &std::path::Path) -> anyhow::Result<String> {
    let output = std::process::Command::new("bash")
        .arg("-lc")
        .arg("source .cache/spec-study-env && printf '%s' \"$BIOMCP_STUDY_DIR\"")
        .current_dir(root)
        .output()?;
    anyhow::ensure!(output.status.success(), "could not read study fixture env");
    let study_dir = String::from_utf8(output.stdout)?.trim().to_string();
    anyhow::ensure!(
        !study_dir.is_empty(),
        "study fixture did not set BIOMCP_STUDY_DIR"
    );
    Ok(study_dir)
}

fn start_ols4_stub() -> anyhow::Result<(thread::JoinHandle<()>, String)> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(false)?;
    let url = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let handle = thread::spawn(move || {
        for stream in listener.incoming().take(8).flatten() {
            handle_ols4_connection(stream);
        }
    });
    Ok((handle, url))
}

fn handle_ols4_connection(mut stream: TcpStream) {
    let mut buffer = [0_u8; 2048];
    let _ = stream.read(&mut buffer);
    let body = json!({
        "response": {
            "numFound": 1,
            "start": 0,
            "docs": [{
                "iri": "http://identifiers.org/hgnc/1100",
                "ontology_name": "hgnc",
                "ontology_prefix": "HGNC",
                "short_form": "HGNC_1100",
                "obo_id": "HGNC:1100",
                "label": "BRCA1",
                "description": ["BRCA1 DNA repair associated"],
                "type": "class",
                "is_defining_ontology": true
            }]
        }
    })
    .to_string();
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
}

fn base_server_command(extra_env: &[(&str, String)]) -> Command {
    let mut command = Command::new(biomcp_bin());
    command.env("RUST_MIN_STACK", "8388608");
    command.env("UMLS_API_KEY", "");
    for (key, value) in extra_env {
        command.env(key, value);
    }
    command.stderr(Stdio::inherit());
    command
}

async fn spawn_stdio_client(
    extra_env: &[(&str, String)],
) -> anyhow::Result<
    rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
> {
    let (client, _) = spawn_stdio_client_with_pid(extra_env).await?;
    Ok(client)
}

async fn spawn_stdio_client_with_pid(
    extra_env: &[(&str, String)],
) -> anyhow::Result<(
    rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
    Option<u32>,
)> {
    let mut command = base_server_command(extra_env);
    command.arg("serve");
    let transport = TokioChildProcess::new(command)?;
    let pid = transport.id();
    Ok((().serve(transport).await?, pid))
}

fn terminate_process(pid: Option<u32>) -> anyhow::Result<()> {
    if let Some(pid) = pid {
        let status = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()?;
        anyhow::ensure!(status.success(), "failed to terminate child process {pid}");
    }
    Ok(())
}

async fn spawn_http_server(extra_env: &[(&str, String)]) -> anyhow::Result<(Child, String)> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let mut command = base_server_command(extra_env);
    command
        .arg("serve-http")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null());
    let mut child = command.spawn()?;

    let base_url = format!("http://127.0.0.1:{port}");
    for _ in 0..200 {
        if reqwest::get(format!("{base_url}/health"))
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return Ok((child, base_url));
        }
        if let Some(status) = child.try_wait()? {
            anyhow::bail!("serve-http exited before healthcheck succeeded: {status}");
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }

    let _ = child.kill().await;
    anyhow::bail!("serve-http did not become ready at {base_url}/health")
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_core_contract() -> anyhow::Result<()> {
    let client = spawn_stdio_client(&[]).await?;
    assert_explore_core_contract(&client).await?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_full_contract() -> anyhow::Result<()> {
    let (_ols_thread, ols_url) = start_ols4_stub()?;
    let (client, pid) = spawn_stdio_client_with_pid(&[("BIOMCP_OLS4_BASE", ols_url)]).await?;

    assert_initialize_and_tools(&client).await?;
    assert_version_call(&client).await?;
    assert_resource_inventory_and_reads(&client).await?;
    assert_read_only_and_policy_calls(&client).await?;
    assert_invalid_resource_error(&client).await?;

    terminate_process(pid)?;
    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_chart_contract() -> anyhow::Result<()> {
    let fixture_root = provision_study_fixture()?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let client = spawn_stdio_client(&[("BIOMCP_STUDY_DIR", study_dir)]).await?;

    assert_chart_calls(&client).await?;

    client.cancel().await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_verifies_core_contract() -> anyhow::Result<()> {
    let (mut child, base_url) = spawn_http_server(&[]).await?;
    let result = async {
        let transport = StreamableHttpClientTransport::from_uri(format!("{base_url}/mcp"));
        let client = ().serve(transport).await?;
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
    let (_ols_thread, ols_url) = start_ols4_stub()?;
    let (mut child, base_url) = spawn_http_server(&[("BIOMCP_OLS4_BASE", ols_url)]).await?;
    let result = async {
        let transport = StreamableHttpClientTransport::from_uri(format!("{base_url}/mcp"));
        let client = ().serve(transport).await?;
        assert_initialize_and_tools(&client).await?;
        assert_version_call(&client).await?;
        assert_resource_inventory_and_reads(&client).await?;
        assert_read_only_and_policy_calls(&client).await?;
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
    let fixture_root = provision_study_fixture()?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let (mut child, base_url) = spawn_http_server(&[("BIOMCP_STUDY_DIR", study_dir)]).await?;
    let result = async {
        let transport = StreamableHttpClientTransport::from_uri(format!("{base_url}/mcp"));
        let client = ().serve(transport).await?;
        assert_chart_calls(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}
