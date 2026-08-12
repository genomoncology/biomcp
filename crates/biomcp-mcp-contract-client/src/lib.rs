use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::thread;
use std::time::Duration;

use base64::Engine;
use rmcp::ServiceExt;
use rmcp::model::{
    CallToolRequestParams, RawContent, ReadResourceRequestParams, ResourceContents, Tool,
};
use rmcp::service::ServiceError;
use rmcp::transport::{StreamableHttpClientTransport, TokioChildProcess};
use serde_json::json;
use tokio::process::{Child, Command};

pub type EnvVar = (&'static str, String);
pub type RunningClient<T> = rmcp::service::RunningService<rmcp::RoleClient, T>;

pub struct ArticleFulltextFixture {
    workspace: tempfile::TempDir,
    repo_root: PathBuf,
    pub base_url: String,
    pub cache_dir: PathBuf,
}

impl Drop for ArticleFulltextFixture {
    fn drop(&mut self) {
        let _ = std::process::Command::new("bash")
            .arg(
                self.repo_root
                    .join("spec/fixtures/cleanup-article-fulltext-source-fixture.sh"),
            )
            .arg(self.workspace.path())
            .status();
    }
}

#[derive(Debug, Clone)]
pub struct ContractHarness {
    pub biomcp_bin: PathBuf,
    pub repo_root: PathBuf,
}

impl ContractHarness {
    pub fn new(biomcp_bin: impl Into<PathBuf>, repo_root: impl Into<PathBuf>) -> Self {
        Self {
            biomcp_bin: biomcp_bin.into(),
            repo_root: repo_root.into(),
        }
    }

    pub fn from_repo_root(repo_root: impl Into<PathBuf>) -> Self {
        let repo_root = repo_root.into();
        let biomcp_bin = std::env::var_os("CARGO_BIN_EXE_biomcp")
            .map(PathBuf::from)
            .unwrap_or_else(|| repo_root.join("target/debug/biomcp"));
        Self {
            biomcp_bin,
            repo_root,
        }
    }
}

const EXPECTED_HELP_RESOURCE: (&str, &str) = ("biomcp://help", "BioMCP Overview");
const READ_ONLY_MESSAGE: &str = "BioMCP allows read-only commands only";
const CACHE_CLI_ONLY_MESSAGE: &str = "CLI-only over MCP";
const CACHE_FILESYSTEM_MESSAGE: &str = "workstation-local filesystem paths";

fn assert_tool_metadata(tools: &[Tool]) {
    for tool in tools {
        let name = tool.name.as_ref();
        let annotations = tool
            .annotations
            .as_ref()
            .unwrap_or_else(|| panic!("MCP tool {name} is missing annotations"));
        assert_eq!(
            annotations.read_only_hint,
            Some(true),
            "MCP tool {name} is not marked read-only"
        );
        assert!(
            annotations
                .title
                .as_deref()
                .is_some_and(|title| !title.trim().is_empty()),
            "MCP tool {name} is missing an annotation title"
        );
        assert!(
            tool.description
                .as_deref()
                .is_some_and(|description| !description.trim().is_empty()),
            "MCP tool {name} is missing a description"
        );
    }
}

pub fn text_chunks(content: &[rmcp::model::Content]) -> Vec<&str> {
    content
        .iter()
        .filter_map(|chunk| match &chunk.raw {
            RawContent::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect()
}

pub fn image_chunks(content: &[rmcp::model::Content]) -> Vec<(&str, &str)> {
    content
        .iter()
        .filter_map(|chunk| match &chunk.raw {
            RawContent::Image(image) => Some((image.mime_type.as_str(), image.data.as_str())),
            _ => None,
        })
        .collect()
}

pub fn first_text(content: &[rmcp::model::Content]) -> &str {
    text_chunks(content)
        .into_iter()
        .next()
        .expect("result returned a text content chunk")
}

pub fn tool_arguments(command: &str) -> serde_json::Map<String, serde_json::Value> {
    BTreeMap::from([("command".to_string(), json!(command))])
        .into_iter()
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

fn json_refs_contain(root: &serde_json::Value, value: &serde_json::Value, needle: &str) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(serde_json::Value::as_str) {
                if let Some(target) = reference
                    .strip_prefix('#')
                    .and_then(|pointer| root.pointer(pointer))
                {
                    return json_contains(target, needle);
                }
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

pub async fn call_biomcp<T>(
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

pub async fn call_biomcp_json<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
    command: &str,
) -> anyhow::Result<rmcp::model::CallToolResult>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let mut arguments = tool_arguments(command);
    arguments.insert("json".to_string(), json!(true));
    Ok(client
        .peer()
        .call_tool(CallToolRequestParams::new("biomcp").with_arguments(arguments))
        .await?)
}

pub async fn assert_typed_tool_calls<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let search = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("search").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("pathway")),
                    ("query".to_string(), json!("MAPK signaling")),
                    ("limit".to_string(), json!(1)),
                    ("json".to_string(), json!(true)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    assert_eq!(search.is_error, Some(false));
    assert!(!first_text(&search.content).trim().is_empty());

    let get = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("get").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("pathway")),
                    ("id".to_string(), json!("R-HSA-5673001")),
                    ("sections".to_string(), json!(["genes"])),
                    ("json".to_string(), json!(true)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await?;
    assert_eq!(get.is_error, Some(false));
    assert!(!first_text(&get.content).trim().is_empty());

    let invalid = client
        .peer()
        .call_tool(
            CallToolRequestParams::new("search").with_arguments(
                BTreeMap::from([
                    ("entity".to_string(), json!("pathway")),
                    ("query".to_string(), json!("MAPK")),
                    ("limit".to_string(), json!(50)),
                ])
                .into_iter()
                .collect(),
            ),
        )
        .await
        .expect_err("out-of-schema typed search limit should be rejected");
    match invalid {
        ServiceError::McpError(data) => assert!(data.message.contains("typed search limit")),
        other => panic!("expected MCP invalid params error, got {other:?}"),
    }
    Ok(())
}

fn expected_skill_resources(
    repo_root: impl AsRef<std::path::Path>,
) -> anyhow::Result<Vec<(String, String)>> {
    let mut resources = Vec::new();
    let skills_dir = repo_root.as_ref().join("skills/use-cases");
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

pub async fn assert_initialize_and_tools<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
    repo_root: impl AsRef<Path>,
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
    assert!(instructions.contains("biomcp skill list"));
    assert!(!instructions.contains("biomcp suggest"));
    assert!(instructions.contains("biomcp skill"));

    let tools = client.peer().list_tools(Default::default()).await?;
    assert_tool_metadata(&tools.tools);
    let tool_names = tools
        .tools
        .iter()
        .map(|tool| tool.name.as_ref())
        .collect::<Vec<_>>();
    assert!(tool_names.contains(&"biomcp"));
    assert!(tool_names.contains(&"search"));
    assert!(tool_names.contains(&"get"));
    assert!(!tool_names.contains(&"shell"));
    let search = tools
        .tools
        .iter()
        .find(|tool| tool.name == "search")
        .expect("typed search tool listed");
    let get = tools
        .tools
        .iter()
        .find(|tool| tool.name == "get")
        .expect("typed get tool listed");
    let search_schema = serde_json::to_value(&search.input_schema)?;
    assert!(
        json_property_contains(&search_schema, "entity", "pathway"),
        "typed search entity schema missing pathway enum: {search_schema}"
    );
    assert!(
        json_property_contains(&search_schema, "limit", "25"),
        "typed search limit schema missing 25 bound: {search_schema}"
    );
    let get_schema = serde_json::to_value(&get.input_schema)?;
    assert!(
        json_property_contains(&get_schema, "entity", "gene"),
        "typed get entity schema missing gene enum: {get_schema}"
    );
    assert!(
        json_property_contains(&get_schema, "sections", "pathways"),
        "typed get sections schema missing pathways enum: {get_schema}"
    );
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
    let list_contract =
        std::fs::read_to_string(repo_root.as_ref().join("src/cli/list_reference.md"))?;
    let required = [
        "BioMCP Command Reference",
        "search <entity> [query|filters]",
        "search trial [filters]",
        "get <entity> <id> [section...]",
        "skill list",
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
    assert!(description.contains("TYPED MCP TOOLS:"));
    assert!(description.contains("Prefer typed `search` and `get`"));
    assert!(description.contains("raw `biomcp` as an escape hatch"));
    assert!(description.contains("SEARCH FILTERS:"));
    assert!(description.contains("MCP RESPONSE METADATA:"));
    assert!(description.contains("json: true"));
    assert!(description.contains("_meta.section_sources"));
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

pub async fn assert_version_call<T>(
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

pub async fn assert_explore_core_contract<T>(
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
    assert!(instructions.contains("biomcp skill list"));
    assert!(!instructions.contains("biomcp suggest"));
    assert!(!instructions.contains("15 sources"));

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

pub async fn assert_resource_inventory_and_reads<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
    repo_root: impl AsRef<std::path::Path>,
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
    expected.extend(expected_skill_resources(repo_root)?);
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

pub async fn assert_read_only_and_policy_calls<T>(
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

pub async fn assert_mcp_fulltext_path_redaction<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
    fixture: &ArticleFulltextFixture,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let command = "biomcp get article 22663011 fulltext";
    let text_call = call_biomcp(client, command).await?;
    assert_eq!(text_call.is_error, Some(false));
    let text = first_text(&text_call.content);

    let json_call = call_biomcp_json(client, command).await?;
    assert_eq!(json_call.is_error, Some(false));
    let json_text = first_text(&json_call.content);
    let value: serde_json::Value = serde_json::from_str(json_text)?;

    let cache_root = fixture.cache_dir.to_string_lossy();
    let text_leaked_cache_root = text.contains(cache_root.as_ref());
    let json_leaked_cache_root = json_text.contains(cache_root.as_ref());
    assert!(
        !text_leaked_cache_root && !json_leaked_cache_root,
        "MCP responses exposed adversarial cache root: text={text_leaked_cache_root}, json={json_leaked_cache_root}"
    );
    assert!(
        !text.contains("file://") && !json_text.contains("file://"),
        "MCP response exposed a file URI"
    );
    assert!(
        text.contains("Full text: available (local cache path withheld over MCP)"),
        "MCP text lacked transport-neutral availability: {text}"
    );
    assert!(
        text.contains("Europe PMC XML"),
        "MCP text lost provenance: {text}"
    );
    assert!(
        !text.contains("Saved to:"),
        "MCP text disclosed save path: {text}"
    );
    assert_eq!(value["full_text_available"], true, "json={value}");
    assert!(value.get("full_text_path").is_none(), "json={value}");
    assert!(value["full_text_source"].is_object(), "json={value}");
    Ok(())
}

pub async fn assert_mcp_provenance_calls<T>(
    client: &rmcp::service::RunningService<rmcp::RoleClient, T>,
) -> anyhow::Result<()>
where
    T: rmcp::Service<rmcp::RoleClient>,
{
    let default_call = call_biomcp(client, "biomcp discover BRCA1").await?;
    assert_eq!(default_call.is_error, Some(false));
    let default_text = first_text(&default_call.content);
    assert!(
        default_text.contains("## Sources"),
        "default MCP text lacked Sources footer: {default_text}"
    );
    assert!(
        default_text.contains("Structured Concepts") && default_text.contains("OLS4"),
        "default MCP text lacked upstream resolver attribution: {default_text}"
    );
    assert!(
        default_text.contains("## Next commands") && default_text.contains("biomcp get gene BRCA1"),
        "default MCP text lacked next-command hints: {default_text}"
    );

    let json_call = call_biomcp_json(client, "biomcp discover BRCA1").await?;
    assert_eq!(json_call.is_error, Some(false));
    let value: serde_json::Value = serde_json::from_str(first_text(&json_call.content))?;
    assert!(
        value["_meta"]["section_sources"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "json:true response lacked _meta.section_sources: {value}"
    );
    assert!(
        value["_meta"]["next_commands"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "json:true response lacked _meta.next_commands: {value}"
    );
    assert!(
        value["_meta"]["evidence_urls"]
            .as_array()
            .is_some_and(|items| !items.is_empty()),
        "json:true response lacked _meta.evidence_urls: {value}"
    );

    let command_json = call_biomcp(client, "biomcp discover BRCA1 --json").await?;
    assert_eq!(command_json.is_error, Some(false));
    let command_value: serde_json::Value = serde_json::from_str(first_text(&command_json.content))?;
    assert!(command_value["_meta"]["section_sources"].is_array());

    Ok(())
}

pub async fn assert_invalid_resource_error<T>(
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

pub async fn assert_chart_calls<T>(
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

pub fn provision_article_fulltext_fixture(
    repo_root: impl AsRef<Path>,
) -> anyhow::Result<ArticleFulltextFixture> {
    let repo_root = repo_root.as_ref().to_path_buf();
    let workspace = tempfile::Builder::new()
        .prefix("biomcp-rmcp-fulltext-")
        .tempdir_in(&repo_root)?;
    let output = std::process::Command::new("bash")
        .arg(repo_root.join("spec/fixtures/setup-article-fulltext-source-fixture.sh"))
        .arg(workspace.path())
        .current_dir(&repo_root)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "article full-text fixture setup failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let fixture_root = PathBuf::from(String::from_utf8(output.stdout)?.trim());
    let base_url = std::fs::read_to_string(fixture_root.join("base-url"))?
        .trim()
        .to_string();
    let cache_dir = workspace.path().join("cache path naïve 🧬");
    Ok(ArticleFulltextFixture {
        workspace,
        repo_root,
        base_url,
        cache_dir,
    })
}

pub fn article_fulltext_fixture_env(fixture: &ArticleFulltextFixture) -> Vec<EnvVar> {
    let mut env = vec![(
        "BIOMCP_CACHE_DIR",
        fixture.cache_dir.to_string_lossy().into_owned(),
    )];
    for name in [
        "BIOMCP_TEST_UNPACED_ORIGIN",
        "BIOMCP_PUBTATOR_BASE",
        "BIOMCP_EUROPEPMC_BASE",
        "BIOMCP_PUBMED_BASE",
        "BIOMCP_PMC_OA_BASE",
        "BIOMCP_PMC_HTML_BASE",
        "BIOMCP_NCBI_IDCONV_BASE",
        "BIOMCP_S2_BASE",
        "BIOMCP_FIGSHARE_BASE",
    ] {
        env.push((name, fixture.base_url.clone()));
    }
    env
}

pub fn provision_study_fixture(repo_root: impl AsRef<Path>) -> anyhow::Result<tempfile::TempDir> {
    let repo_root = repo_root.as_ref();
    let temp = tempfile::Builder::new()
        .prefix("biomcp-study-rmcp-tests-")
        .tempdir()?;
    let script = repo_root.join("spec/fixtures/setup-study-spec-fixture.sh");
    let status = std::process::Command::new("bash")
        .arg(script)
        .arg(temp.path())
        .current_dir(repo_root)
        .status()?;
    anyhow::ensure!(status.success(), "study fixture setup failed: {status}");
    Ok(temp)
}

pub fn study_dir_from_fixture(root: &std::path::Path) -> anyhow::Result<String> {
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

pub fn start_ols4_stub() -> anyhow::Result<(thread::JoinHandle<()>, String)> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(false)?;
    let url = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let handle = thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            handle_ols4_connection(stream);
        }
    });
    Ok((handle, url))
}

pub fn start_counting_ols4_stub()
-> anyhow::Result<(thread::JoinHandle<()>, String, Arc<AtomicUsize>)> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    listener.set_nonblocking(false)?;
    let url = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&requests);
    let handle = thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            observed.fetch_add(1, Ordering::SeqCst);
            handle_ols4_connection(stream);
        }
    });
    Ok((handle, url, requests))
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

impl ContractHarness {
    fn base_server_command(&self, extra_env: &[EnvVar]) -> Command {
        let mut command = Command::new(&self.biomcp_bin);
        command.env_remove("RUST_MIN_STACK");
        command.env("UMLS_API_KEY", "");
        for (key, value) in extra_env {
            command.env(key, value);
        }
        command.stderr(Stdio::inherit());
        command
    }

    pub async fn spawn_stdio_client(
        &self,
        extra_env: &[EnvVar],
    ) -> anyhow::Result<RunningClient<impl rmcp::Service<rmcp::RoleClient>>> {
        let (client, _) = self.spawn_stdio_client_with_pid(extra_env).await?;
        Ok(client)
    }

    pub async fn spawn_stdio_client_with_pid(
        &self,
        extra_env: &[EnvVar],
    ) -> anyhow::Result<(
        RunningClient<impl rmcp::Service<rmcp::RoleClient>>,
        Option<u32>,
    )> {
        let mut command = self.base_server_command(extra_env);
        command.arg("serve");
        let transport = TokioChildProcess::new(command)?;
        let pid = transport.id();
        Ok((().serve(transport).await?, pid))
    }

    pub async fn spawn_http_server(&self, extra_env: &[EnvVar]) -> anyhow::Result<(Child, String)> {
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
        let port = listener.local_addr()?.port();
        drop(listener);

        let mut command = self.base_server_command(extra_env);
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

    pub async fn http_client(
        &self,
        mcp_url: impl Into<String>,
    ) -> anyhow::Result<RunningClient<impl rmcp::Service<rmcp::RoleClient>>> {
        let transport = StreamableHttpClientTransport::from_uri(mcp_url.into());
        Ok(().serve(transport).await?)
    }
}

pub fn terminate_process(pid: Option<u32>) -> anyhow::Result<()> {
    if let Some(pid) = pid {
        let status = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .status()?;
        anyhow::ensure!(status.success(), "failed to terminate child process {pid}");
    }
    Ok(())
}
