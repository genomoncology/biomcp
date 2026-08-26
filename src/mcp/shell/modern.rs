use rmcp::{
    ServerHandler,
    model::{CallToolRequestParams, NumberOrString, ServerInfo},
    service::RequestContext,
};
use serde_json::{Map, Value, json};

use super::super::{BioMcpServer, build_resource_list, read_resource_markdown};

pub(in crate::mcp::shell) const MODERN_PROTOCOL_VERSION: &str = "2026-07-28";
pub(in crate::mcp::shell) const SUPPORTED_VERSIONS: [&str; 3] =
    ["2025-06-18", "2025-11-25", MODERN_PROTOCOL_VERSION];
const CACHE_TTL_MS: u64 = 300_000;
const PROTOCOL_META: &str = "io.modelcontextprotocol/protocolVersion";
const CAPABILITIES_META: &str = "io.modelcontextprotocol/clientCapabilities";

pub(in crate::mcp::shell) fn protocol_version(request: &Value) -> Option<&str> {
    request
        .get("params")?
        .get("_meta")?
        .get(PROTOCOL_META)?
        .as_str()
}

pub(in crate::mcp::shell) fn has_protocol_metadata(request: &Value) -> bool {
    request
        .get("params")
        .and_then(|params| params.get("_meta"))
        .and_then(|meta| meta.get(PROTOCOL_META))
        .is_some()
}

pub(in crate::mcp::shell) async fn dispatch(request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return error(id, -32602, "Missing method", None);
    };
    let Some(params) = request.get("params").and_then(Value::as_object) else {
        return error(id, -32602, "Missing request params", None);
    };
    let Some(meta) = params.get("_meta").and_then(Value::as_object) else {
        return error(id, -32602, "Missing request metadata", None);
    };
    let Some(version) = meta.get(PROTOCOL_META).and_then(Value::as_str) else {
        return error(id, -32602, "Missing per-request protocol version", None);
    };
    if !meta.get(CAPABILITIES_META).is_some_and(Value::is_object) {
        return error(id, -32602, "Missing per-request client capabilities", None);
    }
    if version != MODERN_PROTOCOL_VERSION {
        return error(
            id,
            -32022,
            "UnsupportedProtocolVersionError",
            Some(json!({"requested": version, "supported": SUPPORTED_VERSIONS})),
        );
    }

    match dispatch_result(method, params, &id).await {
        Ok(DispatchResult::Result(mut result, cacheable)) => {
            decorate(&mut result, cacheable);
            json!({"jsonrpc": "2.0", "id": id, "result": result})
        }
        Ok(DispatchResult::Notification(notification)) => notification,
        Err(mut rpc_error) => {
            if method == "resources/read" && rpc_error["code"] == -32002 {
                rpc_error["code"] = json!(-32602);
            }
            json!({"jsonrpc": "2.0", "id": id, "error": rpc_error})
        }
    }
}

enum DispatchResult {
    Result(Value, bool),
    Notification(Value),
}

async fn dispatch_result(
    method: &str,
    params: &Map<String, Value>,
    id: &Value,
) -> Result<DispatchResult, Value> {
    let server = BioMcpServer::new();
    let result = match method {
        "server/discover" => {
            let ServerInfo {
                capabilities,
                server_info,
                ..
            } = server.get_info();
            json!({
                "supportedVersions": SUPPORTED_VERSIONS,
                "capabilities": capabilities,
                "_meta": {"io.modelcontextprotocol/serverInfo": server_info}
            })
        }
        "tools/list" => json!({"tools": super::super::super::catalog::list(&server.tool_router)}),
        "resources/list" => json!({
            "resources": build_resource_list()
                .into_iter()
                .map(rmcp::model::AnnotateAble::no_annotation)
                .collect::<Vec<_>>()
        }),
        "resources/templates/list" => json!({"resourceTemplates": []}),
        "resources/read" => {
            let uri = required_string(params, "uri")?;
            serde_json::to_value(read_resource_markdown(uri).map_err(error_value)?)
                .map_err(internal_serialization_error)?
        }
        "tools/call" => {
            let name = required_string(params, "name")?;
            let arguments = match params.get("arguments") {
                Some(Value::Object(arguments)) => Some(arguments.clone()),
                Some(_) => return Err(invalid_params("Tool arguments must be an object")),
                None => None,
            };
            let request = CallToolRequestParams::new(name.to_string());
            let request = match arguments {
                Some(arguments) => request.with_arguments(arguments),
                None => request,
            };
            let (transport, _remote) = tokio::io::duplex(64);
            let running = rmcp::service::serve_directly::<rmcp::RoleServer, _, _, _, _>(
                BioMcpServer::new(),
                transport,
                None,
            );
            let context = RequestContext::new(
                NumberOrString::String("modern".into()),
                running.peer().clone(),
            );
            let called = server
                .call_tool(request, context)
                .await
                .map_err(error_value)?;
            serde_json::to_value(called).map_err(internal_serialization_error)?
        }
        "subscriptions/listen" => {
            return Ok(DispatchResult::Notification(json!({
                "jsonrpc": "2.0",
                "method": "notifications/subscriptions/acknowledged",
                "params": {
                    "notifications": {},
                    "_meta": {"io.modelcontextprotocol/subscriptionId": id}
                }
            })));
        }
        _ => return Err(json!({"code": -32601, "message": "Method not found"})),
    };
    Ok(DispatchResult::Result(
        result,
        matches!(
            method,
            "server/discover"
                | "tools/list"
                | "resources/list"
                | "resources/templates/list"
                | "resources/read"
        ),
    ))
}

fn required_string<'a>(params: &'a Map<String, Value>, name: &str) -> Result<&'a str, Value> {
    params
        .get(name)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| invalid_params(&format!("Missing {name}")))
}

fn invalid_params(message: &str) -> Value {
    json!({"code": -32602, "message": message})
}

fn error_value(error: rmcp::ErrorData) -> Value {
    serde_json::to_value(error)
        .unwrap_or_else(|_| json!({"code": -32603, "message": "Failed to serialize MCP error"}))
}

fn internal_serialization_error(error: serde_json::Error) -> Value {
    json!({"code": -32603, "message": format!("Failed to serialize MCP result: {error}")})
}

fn decorate(result: &mut Value, cacheable: bool) {
    let ServerInfo { server_info, .. } = BioMcpServer::new().get_info();
    let object = result
        .as_object_mut()
        .expect("modern MCP result payloads are objects");
    object.insert("resultType".into(), json!("complete"));
    let meta = object
        .entry("_meta")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .expect("modern result metadata is an object");
    meta.insert(
        "io.modelcontextprotocol/serverInfo".into(),
        serde_json::to_value(server_info).expect("server identity serializes"),
    );
    if cacheable {
        object.insert("ttlMs".into(), json!(CACHE_TTL_MS));
        object.insert("cacheScope".into(), json!("public"));
    }
}

pub(in crate::mcp::shell) fn error(
    id: Value,
    code: i32,
    message: &str,
    data: Option<Value>,
) -> Value {
    let mut rpc_error = json!({"code": code, "message": message});
    if let Some(data) = data {
        rpc_error["data"] = data;
    }
    json!({"jsonrpc": "2.0", "id": id, "error": rpc_error})
}
