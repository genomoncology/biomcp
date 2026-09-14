use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

pub(super) fn binary_download_rejection(
    message: String,
    json: bool,
) -> Result<CallToolResult, McpError> {
    if !json {
        return Ok(super::BioMcpServer::tool_error(message));
    }
    let error =
        crate::error::BioMcpError::InvalidArgument(crate::render::human::sanitize_inline(&message));
    let text = crate::render::json::to_error_json(&error).map_err(|error| {
        McpError::internal_error(
            format!("Failed to render MCP JSON rejection: {error}"),
            None,
        )
    })?;
    Ok(CallToolResult::error(vec![Content::text(text)]))
}
