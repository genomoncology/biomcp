use std::fs;
use std::path::PathBuf;

use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_json(relative_path: &str) -> Value {
    let path = repo_root().join(relative_path);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    serde_json::from_str(&contents)
        .unwrap_or_else(|err| panic!("failed to parse {} as JSON: {err}", path.display()))
}

#[test]
fn claude_plugin_marketplace_wires_biomcp_stdio_server() {
    let marketplace = read_json(".claude-plugin/marketplace.json");
    let plugins = marketplace
        .get("plugins")
        .and_then(Value::as_array)
        .expect("marketplace plugins must be an array");
    let biomcp_plugin = plugins
        .iter()
        .find(|plugin| plugin.get("name").and_then(Value::as_str) == Some("biomcp"))
        .expect("marketplace must include the biomcp plugin");
    let biomcp_server = biomcp_plugin
        .pointer("/mcpServers/biomcp")
        .expect("biomcp plugin must declare its MCP server inline");

    assert_eq!(
        biomcp_server.get("command").and_then(Value::as_str),
        Some("biomcp")
    );
    assert_eq!(
        biomcp_server.get("args"),
        Some(&Value::Array(vec![Value::String("serve".to_string())]))
    );
}

#[test]
fn claude_plugin_manifest_does_not_duplicate_mcp_server_declaration() {
    let plugin = read_json(".claude-plugin/plugin.json");

    assert_eq!(plugin.get("name").and_then(Value::as_str), Some("biomcp"));
    assert!(
        plugin.get("mcpServers").is_none(),
        ".claude-plugin/plugin.json must not declare mcpServers; marketplace.json is the single source of truth"
    );
}
