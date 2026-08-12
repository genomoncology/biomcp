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

async fn call_typed_get(
    client: &rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
    entity: &str,
    id: &str,
    sections: &[&str],
) -> anyhow::Result<rmcp::model::CallToolResult> {
    let arguments = serde_json::Map::from_iter([
        ("entity".to_string(), json!(entity)),
        ("id".to_string(), json!(id)),
        ("sections".to_string(), json!(sections)),
        ("json".to_string(), json!(true)),
    ]);
    Ok(client
        .peer()
        .call_tool(CallToolRequestParams::new("get").with_arguments(arguments))
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

fn json_property_contains(value: &serde_json::Value, property: &str, needle: &str) -> bool {
    fn visit(
        root: &serde_json::Value,
        value: &serde_json::Value,
        property: &str,
        needle: &str,
    ) -> bool {
        match value {
            serde_json::Value::Object(map) => {
                map.get(property).is_some_and(|property_value| {
                    json_contains(property_value, needle)
                        || json_refs_contain(root, property_value, needle)
                }) || map
                    .values()
                    .any(|child| visit(root, child, property, needle))
            }
            serde_json::Value::Array(items) => items
                .iter()
                .any(|child| visit(root, child, property, needle)),
            serde_json::Value::String(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Bool(_)
            | serde_json::Value::Null => false,
        }
    }

    visit(value, value, property, needle)
}

fn json_refs_contain(root: &serde_json::Value, value: &serde_json::Value, needle: &str) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(serde_json::Value::as_str)
                && let Some(target) = reference
                    .strip_prefix('#')
                    .and_then(|pointer| root.pointer(pointer))
            {
                return json_contains(target, needle);
            }
            map.values()
                .any(|child| json_refs_contain(root, child, needle))
        }
        serde_json::Value::Array(items) => items
            .iter()
            .any(|child| json_refs_contain(root, child, needle)),
        serde_json::Value::String(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Bool(_)
        | serde_json::Value::Null => false,
    }
}

fn named_root_property_contains(schema: &serde_json::Value, property: &str, needle: &str) -> bool {
    let Some(value) = schema.pointer(&format!("/properties/{property}")) else {
        return false;
    };
    let value = value
        .get("$ref")
        .and_then(serde_json::Value::as_str)
        .and_then(|reference| reference.strip_prefix('#'))
        .and_then(|pointer| schema.pointer(pointer))
        .unwrap_or(value);
    json_contains(value, needle)
}

#[cfg(test)]
mod tests {
    use super::named_root_property_contains;
    use serde_json::json;

    #[test]
    fn named_root_property_ignores_nested_refs() {
        let schema = json!({
            "properties": {
                "inputs": { "items": { "$ref": "#/$defs/unrelated" } }
            },
            "$defs": { "unrelated": { "maximum": 50 } }
        });

        assert!(!named_root_property_contains(&schema, "inputs", "50"));
    }
}

fn assert_tool_metadata(tools: &[Tool]) -> anyhow::Result<()> {
    for tool in tools {
        let name = tool.name.as_ref();
        let annotations = tool
            .annotations
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("MCP tool {name} is missing annotations"))?;
        if annotations.read_only_hint != Some(true) {
            anyhow::bail!("MCP tool {name} is not marked read-only");
        }
        if annotations
            .title
            .as_deref()
            .is_none_or(|title| title.trim().is_empty())
        {
            anyhow::bail!("MCP tool {name} is missing an annotation title");
        }
        if tool
            .description
            .as_deref()
            .is_none_or(|description| description.trim().is_empty())
        {
            anyhow::bail!("MCP tool {name} is missing a description");
        }
    }
    Ok(())
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
    for required in [
        "biomcp",
        "search",
        "get",
        "variant_normalize_car",
        "variant_erepo",
        "gene_cspec",
        "variant_articles",
    ] {
        if !names.contains(&required) {
            anyhow::bail!("typed MCP surface missing tool: {required}");
        }
    }

    assert_tool_metadata(&tools.tools)?;

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
    let variant_normalize_car = tools
        .tools
        .iter()
        .find(|tool| tool.name == "variant_normalize_car")
        .expect("variant_normalize_car tool checked above");
    let variant_erepo = tools
        .tools
        .iter()
        .find(|tool| tool.name == "variant_erepo")
        .expect("variant_erepo tool checked above");
    let gene_cspec = tools
        .tools
        .iter()
        .find(|tool| tool.name == "gene_cspec")
        .expect("gene_cspec tool checked above");
    let variant_articles = tools
        .tools
        .iter()
        .find(|tool| tool.name == "variant_articles")
        .expect("variant_articles tool checked above");
    let search_schema = tool_schema(search);
    let get_schema = tool_schema(get);
    let variant_normalize_car_schema = tool_schema(variant_normalize_car);
    let variant_erepo_schema = tool_schema(variant_erepo);
    let gene_cspec_schema = tool_schema(gene_cspec);
    let variant_articles_schema = tool_schema(variant_articles);

    if search_schema
        .get("oneOf")
        .and_then(serde_json::Value::as_array)
        .is_none_or(|branches| branches.len() != 8)
    {
        anyhow::bail!("search schema must have eight entity-specific branches");
    }
    if get_schema
        .get("oneOf")
        .and_then(serde_json::Value::as_array)
        .is_none_or(|branches| branches.len() != 12)
    {
        anyhow::bail!("get schema must have twelve entity-specific branches");
    }
    if !json_property_contains(&search_schema, "entity", "gwas") {
        anyhow::bail!("search entity schema missing gwas branch");
    }
    if !json_property_contains(&search_schema, "entity", "author") {
        anyhow::bail!("search entity schema missing author enum");
    }
    if !json_property_contains(&get_schema, "entity", "author") {
        anyhow::bail!("get entity schema missing author enum");
    }
    if !json_property_contains(&search_schema, "limit", "25") {
        anyhow::bail!("search limit schema missing 25 bound");
    }
    if !json_property_contains(&get_schema, "entity", "gene") {
        anyhow::bail!("get entity schema missing gene enum");
    }
    if !json_property_contains(&get_schema, "sections", "pathways") {
        anyhow::bail!("get sections schema missing pathways enum");
    }
    if !json_property_contains(&get_schema, "sections", "indexing") {
        anyhow::bail!("get sections schema missing indexing enum");
    }
    for bound in ["1", "50"] {
        if !named_root_property_contains(&variant_normalize_car_schema, "inputs", bound) {
            anyhow::bail!("variant_normalize_car schema missing {bound} input bound");
        }
        if !named_root_property_contains(&gene_cspec_schema, "limit", bound) {
            anyhow::bail!("gene_cspec schema missing {bound} paging bound");
        }
    }
    for selector in ["caid", "caids"] {
        if !named_root_property_contains(&variant_erepo_schema, selector, "string") {
            anyhow::bail!("variant_erepo schema missing {selector} selector");
        }
    }
    if !named_root_property_contains(&gene_cspec_schema, "capture_id", "string") {
        anyhow::bail!("gene_cspec schema missing capture_id");
    }
    if gene_cspec_schema.pointer("/properties/raw_bytes").is_some() {
        anyhow::bail!("gene_cspec schema must not expose CLI-only raw bytes");
    }
    for control in ["verify_identity", "confirmed_only"] {
        if !named_root_property_contains(&variant_articles_schema, control, "boolean") {
            anyhow::bail!("variant_articles schema missing {control} boolean");
        }
    }

    println!(
        "MCP tools: biomcp, search, get, variant_normalize_car, variant_erepo, gene_cspec, variant_articles"
    );
    println!("ClinGen schemas validate their named root properties");
    println!("all listed MCP tools are read-only annotated");
    println!("all listed MCP tools have titles and descriptions");
    println!("search and get schemas use entity-specific branches");
    println!("search schema includes a bounded limit");
    println!("search and get schemas include author entity");
    println!("get schema includes entity and sections enum");
    println!("variant_articles schema includes identity verification controls");
    println!("indexing");
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools|section-outcome> <port>"
        )
    })?;
    let port = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools|section-outcome> <port>"
        )
    })?;
    if args.next().is_some() {
        anyhow::bail!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools|section-outcome> <port>"
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
        "section-outcome" => {
            let result = call_typed_get(&client, "drug", "fixture-drug", &["approvals"]).await?;
            println!("{}", first_text(&result)?);
        }
        _ => anyhow::bail!("unknown mode: {mode}"),
    }

    client.cancel().await?;
    Ok(())
}
