use std::collections::BTreeMap;

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, RawContent, Tool};
use rmcp::transport::StreamableHttpClientTransport;
use serde_json::json;

fn tool_arguments(command: &str) -> serde_json::Map<String, serde_json::Value> {
    BTreeMap::from([("command".to_string(), json!(command))])
        .into_iter()
        .collect()
}

async fn call_biomcp(
    client: &rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
    command: &str,
) -> anyhow::Result<rmcp::model::CallToolResult> {
    Ok(client
        .peer()
        .call_tool(CallToolRequestParams::new("biomcp").with_arguments(tool_arguments(command)))
        .await?)
}

fn first_text(result: &rmcp::model::CallToolResult) -> anyhow::Result<&str> {
    result
        .content
        .iter()
        .find_map(|content| match &content.raw {
            RawContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("tool call returned no text content"))
}

fn first_image_mime(result: &rmcp::model::CallToolResult) -> anyhow::Result<&str> {
    result
        .content
        .iter()
        .find_map(|content| match &content.raw {
            RawContent::Image(image) => Some(image.mime_type.as_str()),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("tool call returned no image content"))
}

fn tool_schema(tool: &Tool) -> serde_json::Value {
    serde_json::to_value(&tool.input_schema).unwrap_or_else(|_| json!({}))
}

fn json_contains(value: &serde_json::Value, needle: &str) -> bool {
    match value {
        serde_json::Value::String(text) => text == needle,
        serde_json::Value::Array(items) => items.iter().any(|item| json_contains(item, needle)),
        serde_json::Value::Object(map) => map
            .iter()
            .any(|(key, value)| key == needle || json_contains(value, needle)),
        serde_json::Value::Number(number) => number.to_string() == needle,
        serde_json::Value::Bool(_) | serde_json::Value::Null => false,
    }
}

async fn print_typed_tool_surface(
    client: &rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
) -> anyhow::Result<()> {
    let tools = client.peer().list_tools(Default::default()).await?;
    let names = tools
        .tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();
    for required in ["biomcp", "search", "get"] {
        if !names.contains(&required) {
            anyhow::bail!("typed MCP surface missing tool: {required}");
        }
    }

    let search = tools
        .tools
        .iter()
        .find(|tool| tool.name == "search")
        .expect("search tool checked above");
    let get = tools
        .tools
        .iter()
        .find(|tool| tool.name == "get")
        .expect("get tool checked above");
    let search_schema = tool_schema(search);
    let get_schema = tool_schema(get);

    for required in ["entity", "pathway", "limit", "25"] {
        if !json_contains(&search_schema, required) {
            anyhow::bail!("search schema missing {required}");
        }
    }
    for required in ["entity", "gene", "sections", "pathways"] {
        if !json_contains(&get_schema, required) {
            anyhow::bail!("get schema missing {required}");
        }
    }

    println!("MCP typed tools: biomcp, search, get");
    println!("search schema includes entity enum and bounded limit");
    println!("get schema includes entity and sections enum");
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools> <port>"
        )
    })?;
    let port = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools> <port>"
        )
    })?;
    if args.next().is_some() {
        anyhow::bail!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools> <port>"
        );
    }

    let transport = StreamableHttpClientTransport::from_uri(format!("http://127.0.0.1:{port}/mcp"));
    let client = ().serve(transport).await?;

    match mode.as_str() {
        "remote-workflow" => {
            let command = "biomcp study query --study msk_impact_2017 --gene TP53 --type mutations";
            let result = call_biomcp(&client, command).await?;
            println!("Command: {command}");
            println!("{}", first_text(&result)?);
        }
        "boundaries" => {
            let reject = call_biomcp(&client, "biomcp cache path").await?;
            let unknown_skill = call_biomcp(&client, "biomcp skill sync").await?;
            let chart = call_biomcp(
                &client,
                "biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar",
            )
            .await?;
            println!("{}", first_text(&reject)?);
            println!("{}", first_text(&unknown_skill)?);
            let first_line = first_text(&chart)?.lines().next().unwrap_or_default();
            println!("{first_line}");
            println!("IMAGE: {}", first_image_mime(&chart)?);
        }
        "typed-tools" => print_typed_tool_surface(&client).await?,
        _ => anyhow::bail!("unknown mode: {mode}"),
    }

    client.cancel().await?;
    Ok(())
}
