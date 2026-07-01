use serde::Serialize;

use crate::cli::system::{McpClient, McpConfigArgs};

const DEFAULT_COMMAND: &str = "biomcp";
const SERVER_NAME: &str = "biomcp";
const SERVER_ARGS: [&str; 1] = ["serve"];

#[derive(Serialize)]
struct McpServersConfig<'a> {
    #[serde(rename = "mcpServers")]
    mcp_servers: std::collections::BTreeMap<&'static str, McpServerConfig<'a>>,
}

#[derive(Serialize)]
struct McpServerConfig<'a> {
    command: &'a str,
    args: Vec<&'static str>,
}

pub(crate) fn run(args: McpConfigArgs) -> anyhow::Result<String> {
    let command = if args.absolute_path {
        std::env::current_exe()?.display().to_string()
    } else {
        DEFAULT_COMMAND.to_string()
    };
    render(args.client, &command)
}

fn render(client: Option<McpClient>, command: &str) -> anyhow::Result<String> {
    Ok(match client {
        Some(McpClient::Codex) => format!("codex mcp add {SERVER_NAME} -- {command} serve\n"),
        Some(McpClient::ClaudeDesktop) => json_config(command)?,
        Some(McpClient::ClaudeCode) => json_config(command)?,
        Some(McpClient::Cursor) => json_config(command)?,
        Some(McpClient::Cline) => json_config(command)?,
        Some(McpClient::Vscode) => json_config(command)?,
        Some(McpClient::Json) => json_config(command)?,
        None => discovery_page(),
    })
}

fn json_config(command: &str) -> anyhow::Result<String> {
    let mut mcp_servers = std::collections::BTreeMap::new();
    mcp_servers.insert(
        SERVER_NAME,
        McpServerConfig {
            command,
            args: SERVER_ARGS.to_vec(),
        },
    );
    Ok(format!(
        "{}\n",
        crate::render::json::to_pretty(&McpServersConfig { mcp_servers })?
    ))
}

fn discovery_page() -> String {
    "Supported MCP clients:\n\
- codex\n\
- claude-desktop\n\
- claude-code\n\
- cursor\n\
- cline\n\
- vscode\n\
- json\n\
\n\
Examples:\n\
  biomcp mcp-config --client claude-desktop\n\
  biomcp mcp-config --client codex\n\
  biomcp mcp-config --client json --absolute-path\n"
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn json_value(client: McpClient, command: &str) -> serde_json::Value {
        serde_json::from_str(&render(Some(client), command).expect("render config"))
            .expect("valid json")
    }

    #[test]
    fn codex_uses_default_biomcp_serve_invocation() {
        let out = render(Some(McpClient::Codex), DEFAULT_COMMAND).expect("render codex");
        assert_eq!(out, "codex mcp add biomcp -- biomcp serve\n");
    }

    #[test]
    fn json_clients_use_biomcp_serve_by_default() {
        for client in [
            McpClient::ClaudeDesktop,
            McpClient::ClaudeCode,
            McpClient::Cursor,
            McpClient::Cline,
            McpClient::Vscode,
            McpClient::Json,
        ] {
            let value = json_value(client, DEFAULT_COMMAND);
            assert_eq!(
                value["mcpServers"]["biomcp"]["command"],
                serde_json::Value::String("biomcp".to_string())
            );
            assert_eq!(
                value["mcpServers"]["biomcp"]["args"],
                serde_json::json!(["serve"])
            );
        }
    }

    #[test]
    fn absolute_path_command_stays_valid_json() {
        let value = json_value(McpClient::ClaudeDesktop, "/tmp/Bio MCP/bin/biomcp");
        assert_eq!(
            value["mcpServers"]["biomcp"]["command"],
            serde_json::Value::String("/tmp/Bio MCP/bin/biomcp".to_string())
        );
        assert_eq!(
            value["mcpServers"]["biomcp"]["args"],
            serde_json::json!(["serve"])
        );
    }

    #[test]
    fn no_client_output_lists_clients_and_examples() {
        let out = render(None, DEFAULT_COMMAND).expect("render discovery");
        assert!(out.contains("Supported MCP clients:"));
        assert!(out.contains("codex"));
        assert!(out.contains("claude-desktop"));
        assert!(out.contains("biomcp mcp-config --client claude-desktop"));
    }
}
