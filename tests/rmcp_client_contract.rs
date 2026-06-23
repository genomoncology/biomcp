use std::collections::BTreeMap;
use std::process::Stdio;
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, RawContent, ReadResourceRequestParams, ResourceContents};
use rmcp::transport::{StreamableHttpClientTransport, TokioChildProcess};
use serde_json::json;
use tokio::process::{Child, Command};

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

fn image_count(content: &[rmcp::model::Content]) -> usize {
    content
        .iter()
        .filter(|chunk| matches!(chunk.raw, RawContent::Image(_)))
        .count()
}

async fn assert_core_contract<T>(
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

    let args = BTreeMap::from([("command".to_string(), json!("biomcp version"))])
        .into_iter()
        .collect();
    let call = client
        .peer()
        .call_tool(CallToolRequestParams::new("biomcp").with_arguments(args))
        .await?;
    assert_ne!(call.is_error, Some(true));
    let text = text_chunks(&call.content);
    assert!(!text.is_empty(), "call_tool returned no text chunks");
    assert_eq!(
        image_count(&call.content),
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

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_child_process_client_verifies_stdio_core_contract() -> anyhow::Result<()> {
    let mut command = Command::new(biomcp_bin());
    command.arg("serve");
    command.env("RUST_MIN_STACK", "8388608");
    command.stderr(Stdio::inherit());

    let transport = TokioChildProcess::new(command)?;
    let client = ().serve(transport).await?;

    assert_core_contract(&client).await?;

    let resources = client.peer().list_resources(Default::default()).await?;
    assert!(
        resources
            .resources
            .iter()
            .any(|resource| resource.uri == "biomcp://help"),
        "help resource was not listed: {:?}",
        resources.resources
    );
    let help = client
        .peer()
        .read_resource(ReadResourceRequestParams::new("biomcp://help"))
        .await?;
    let help_text = help.contents.iter().find_map(|content| match content {
        ResourceContents::TextResourceContents {
            uri,
            mime_type,
            text,
            ..
        } if uri == "biomcp://help" && mime_type.as_deref() == Some("text/markdown") => {
            Some(text.as_str())
        }
        _ => None,
    });
    let help_text = help_text.expect("help resource returned markdown text");
    assert!(help_text.contains("## Routing rules"));

    client.cancel().await?;
    Ok(())
}

async fn spawn_http_server() -> anyhow::Result<(Child, String)> {
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    let port = listener.local_addr()?.port();
    drop(listener);

    let mut child = Command::new(biomcp_bin())
        .arg("serve-http")
        .arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .env("RUST_MIN_STACK", "8388608")
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()?;

    let base_url = format!("http://127.0.0.1:{port}");
    for _ in 0..40 {
        if reqwest::get(format!("{base_url}/health"))
            .await
            .is_ok_and(|response| response.status().is_success())
        {
            return Ok((child, base_url));
        }
        if let Some(status) = child.try_wait()? {
            anyhow::bail!("serve-http exited before healthcheck succeeded: {status}");
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    let _ = child.kill().await;
    anyhow::bail!("serve-http did not become ready at {base_url}/health")
}

#[tokio::test(flavor = "multi_thread")]
async fn rmcp_streamable_http_client_verifies_core_contract() -> anyhow::Result<()> {
    let (mut child, base_url) = spawn_http_server().await?;
    let result = async {
        let transport = StreamableHttpClientTransport::from_uri(format!("{base_url}/mcp"));
        let client = ().serve(transport).await?;
        assert_core_contract(&client).await?;
        let resources = client.peer().list_resources(Default::default()).await?;
        assert!(
            resources
                .resources
                .iter()
                .any(|resource| resource.uri == "biomcp://help")
        );
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    child.kill().await.ok();
    result
}
