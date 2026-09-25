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
    call_typed_get_with_output(client, entity, id, sections, true).await
}

async fn call_typed_get_with_output(
    client: &rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
    entity: &str,
    id: &str,
    sections: &[&str],
    json_output: bool,
) -> anyhow::Result<rmcp::model::CallToolResult> {
    let arguments = serde_json::Map::from_iter([
        ("entity".to_string(), json!(entity)),
        ("id".to_string(), json!(id)),
        ("sections".to_string(), json!(sections)),
        ("json".to_string(), json!(json_output)),
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

async fn interaction_surfaces(
    client: &rmcp::service::RunningService<rmcp::RoleClient, impl rmcp::Service<rmcp::RoleClient>>,
    id: &str,
) -> anyhow::Result<serde_json::Value> {
    let typed_json = call_typed_get(client, "drug", id, &["interactions"]).await?;
    let typed_markdown =
        call_typed_get_with_output(client, "drug", id, &["interactions"], false).await?;
    let raw_json =
        call_biomcp(client, &format!("biomcp --json get drug {id} interactions")).await?;
    let raw_markdown = call_biomcp(client, &format!("biomcp get drug {id} interactions")).await?;
    Ok(json!({
        "id": id,
        "typed_json": serde_json::from_str::<serde_json::Value>(first_text(&typed_json)?)?,
        "typed_json_error": typed_json.is_error,
        "typed_markdown": first_text(&typed_markdown)?,
        "typed_markdown_error": typed_markdown.is_error,
        "raw_json": serde_json::from_str::<serde_json::Value>(first_text(&raw_json)?)?,
        "raw_json_error": raw_json.is_error,
        "raw_markdown": first_text(&raw_markdown)?,
        "raw_markdown_error": raw_markdown.is_error,
    }))
}

fn tool_schema(tool: &Tool) -> serde_json::Value {
    serde_json::to_value(&tool.input_schema).unwrap_or_else(|_| json!({}))
}

fn flat_entity_enum(schema: &serde_json::Value) -> anyhow::Result<&[serde_json::Value]> {
    schema
        .pointer("/properties/entity/enum")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| anyhow::anyhow!("schema missing flat entity enum"))
}

fn flat_section_enum(schema: &serde_json::Value) -> anyhow::Result<Vec<&str>> {
    schema
        .pointer("/properties/sections/items/enum")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("schema missing merged sections enum"))?
        .iter()
        .map(|section| {
            section
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("merged sections enum has a non-string section"))
        })
        .collect()
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

fn named_property_contains(schema: &serde_json::Value, property: &str, needle: &str) -> bool {
    fn visit(
        root: &serde_json::Value,
        value: &serde_json::Value,
        property: &str,
        needle: &str,
    ) -> bool {
        let value_contains = |value: &serde_json::Value| {
            let value = value
                .get("$ref")
                .and_then(serde_json::Value::as_str)
                .and_then(|reference| reference.strip_prefix('#'))
                .and_then(|pointer| root.pointer(pointer))
                .unwrap_or(value);
            json_contains(value, needle)
        };
        value
            .get("properties")
            .and_then(serde_json::Value::as_object)
            .and_then(|properties| properties.get(property))
            .is_some_and(value_contains)
            || ["oneOf", "anyOf", "allOf"].iter().any(|keyword| {
                value
                    .get(keyword)
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|branches| {
                        branches
                            .iter()
                            .any(|branch| visit(root, branch, property, needle))
                    })
            })
    }

    visit(schema, schema, property, needle)
}

#[cfg(test)]
mod tests {
    use super::named_property_contains;
    use serde_json::json;

    #[test]
    fn named_property_ignores_nested_refs() {
        let schema = json!({
            "properties": {
                "inputs": { "items": { "$ref": "#/$defs/unrelated" } }
            },
            "$defs": { "unrelated": { "maximum": 50 } }
        });

        assert!(!named_property_contains(&schema, "inputs", "50"));
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

    // Flat roots (ADR 0002): one object schema per tool, no root
    // combinators, entity enum plus the merged union of branch properties.
    for (tool, schema) in [("search", &search_schema), ("get", &get_schema)] {
        if schema.get("type").and_then(serde_json::Value::as_str) != Some("object") {
            anyhow::bail!("{tool} schema must declare a top-level object type");
        }
        for combinator in ["oneOf", "anyOf", "allOf"] {
            if schema.get(combinator).is_some() {
                anyhow::bail!("{tool} schema must publish a flat root without {combinator}");
            }
        }
    }
    let search_entities = flat_entity_enum(&search_schema)?;
    if search_entities.len() != 8 {
        anyhow::bail!("search entity enum must list the eight search entities");
    }
    let get_entities = flat_entity_enum(&get_schema)?;
    if get_entities.len() != 13 {
        anyhow::bail!("get entity enum must list the thirteen gettable entities");
    }
    for entity in ["gwas", "author", "gene"] {
        if !search_entities.contains(&json!(entity)) {
            anyhow::bail!("search entity enum missing {entity}");
        }
    }
    if !get_entities.contains(&json!("author")) {
        anyhow::bail!("get entity enum missing author");
    }
    // Gene's branch fields ride in the merged union.
    for field in ["query", "region", "gene_type", "chromosome"] {
        if search_schema
            .pointer(&format!("/properties/{field}"))
            .is_none()
        {
            anyhow::bail!("search flat root missing gene field {field}");
        }
    }
    if !json_property_contains(&search_schema, "limit", "25") {
        anyhow::bail!("search limit schema missing 25 bound");
    }
    // The merged sections enum carries every entity's sections and must
    // not expose the CLI-only terminal forms.
    let sections = flat_section_enum(&get_schema)?;
    for section in ["ontology", "conditions", "guidelines", "guidance", "assets"] {
        if !sections.contains(&section) {
            anyhow::bail!("get merged sections must contain {section}");
        }
    }
    for banned in ["asset", "document", "documents"] {
        if sections.contains(&banned) {
            anyhow::bail!("get merged sections must not expose CLI-only {banned}");
        }
    }
    for bound in ["1", "50"] {
        if !named_property_contains(&variant_normalize_car_schema, "inputs", bound) {
            anyhow::bail!("variant_normalize_car schema missing {bound} input bound");
        }
        if !named_property_contains(&gene_cspec_schema, "limit", bound) {
            anyhow::bail!("gene_cspec schema missing {bound} paging bound");
        }
    }
    for selector in ["caid", "caids"] {
        if !named_property_contains(&variant_erepo_schema, selector, "string") {
            anyhow::bail!("variant_erepo schema missing {selector} selector");
        }
    }
    if !named_property_contains(&gene_cspec_schema, "capture_id", "string") {
        anyhow::bail!("gene_cspec schema missing capture_id");
    }
    if gene_cspec_schema.pointer("/properties/raw_bytes").is_some() {
        anyhow::bail!("gene_cspec schema must not expose CLI-only raw bytes");
    }
    for control in ["verify_identity", "confirmed_only"] {
        if !named_property_contains(&variant_articles_schema, control, "boolean") {
            anyhow::bail!("variant_articles schema missing {control} boolean");
        }
    }

    println!(
        "MCP tools: biomcp, search, get, variant_normalize_car, variant_erepo, gene_cspec, variant_articles"
    );
    println!("ClinGen schemas validate their named properties");
    println!("all listed MCP tools are read-only annotated");
    println!("all listed MCP tools have titles and descriptions");
    println!("search and get schemas publish flat roots without combinators");
    println!("search and get schemas declare object roots");
    println!("search schema includes a bounded limit");
    println!("search and get schemas include author entity");
    println!("get schema merges per-entity sections and hides CLI-only forms");
    println!("article schema exposes assets manifest but not asset download");
    println!("variant_articles schema includes identity verification controls");
    println!("indexing");
    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let mode = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools|section-outcome|section-outcome-interactions|clingen-surfaces|gencc-surfaces> <port>"
        )
    })?;
    let port = args.next().ok_or_else(|| {
        anyhow::anyhow!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools|section-outcome|section-outcome-interactions|clingen-surfaces|gencc-surfaces> <port>"
        )
    })?;
    if args.next().is_some() {
        anyhow::bail!(
            "usage: rmcp_streamable_http_contract <remote-workflow|boundaries|typed-tools|section-outcome|section-outcome-interactions|clingen-surfaces|gencc-surfaces> <port>"
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
        "section-outcome-interactions" => {
            let ids: &[&str] = if std::env::var("BIOMCP_DDINTER_DIR")
                .is_ok_and(|path| path.contains("unavailable"))
            {
                &[
                    "fixture-drug-label",
                    "fixture-drug-empty",
                    "fixture-drug-empty-openfda-fail",
                ]
            } else {
                &[
                    "fixture-drug-ddinter-openfda-fail",
                    "fixture-drug-drugbank-openfda-fail",
                    "fixture-drug-empty-openfda-fail",
                ]
            };
            let mut surfaces = Vec::new();
            for id in ids {
                surfaces.push(interaction_surfaces(&client, id).await?);
            }
            println!("{}", serde_json::to_string(&surfaces)?);
        }
        "clingen-surfaces" => {
            let raw_text = call_biomcp(&client, "biomcp get gene TP53 clingen").await?;
            let raw_json = call_biomcp(&client, "biomcp --json get gene TP53 clingen").await?;
            let typed = call_typed_get(&client, "gene", "TP53", &["clingen"]).await?;
            println!("RAW TEXT\n{}", first_text(&raw_text)?);
            println!("RAW JSON\n{}", first_text(&raw_json)?);
            println!("TYPED JSON\n{}", first_text(&typed)?);
        }
        "gencc-surfaces" => {
            let symbol =
                std::env::var("BIOMCP_GENCC_SURFACES_GENE").unwrap_or_else(|_| "ODC1".to_string());
            let raw_text = call_biomcp(&client, &format!("biomcp get gene {symbol} gencc")).await?;
            let raw_json =
                call_biomcp(&client, &format!("biomcp --json get gene {symbol} gencc")).await?;
            let typed = call_typed_get(&client, "gene", &symbol, &["gencc"]).await?;
            println!(
                "{}",
                serde_json::json!({
                    "raw_text": first_text(&raw_text)?,
                    "raw_json": serde_json::from_str::<serde_json::Value>(first_text(&raw_json)?)?,
                    "typed_json": serde_json::from_str::<serde_json::Value>(first_text(&typed)?)?,
                })
            );
        }
        _ => anyhow::bail!("unknown mode: {mode}"),
    }

    client.cancel().await?;
    Ok(())
}
