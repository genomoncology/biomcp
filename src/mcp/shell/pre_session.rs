use std::io::Cursor;

use anyhow::Context;
use rmcp::{ServerHandler, model::ServerInfo};
use serde_json::{Value, json};
use tokio::io::{
    AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader, Chain,
};

use super::BioMcpServer;

pub(super) const STDIO_DISCOVERY_VERSIONS: [&str; 2] = ["2025-06-18", "2025-11-25"];

pub(super) async fn stdio_transport() -> anyhow::Result<(
    Chain<Cursor<Vec<u8>>, BufReader<tokio::io::Stdin>>,
    tokio::io::Stdout,
)> {
    await_initialize(tokio::io::stdin(), tokio::io::stdout()).await
}

async fn await_initialize<R, W>(
    reader: R,
    mut writer: W,
) -> anyhow::Result<(Chain<Cursor<Vec<u8>>, BufReader<R>>, W)>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    loop {
        let mut frame = Vec::new();
        if reader.read_until(b'\n', &mut frame).await? == 0 {
            anyhow::bail!(super::mcp_stdio_guidance());
        }

        let request: Value = match serde_json::from_slice(&frame) {
            Ok(request) => request,
            Err(_) => {
                write_response(
                    &mut writer,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {"code": -32700, "message": "Parse error"}
                    }),
                )
                .await?;
                continue;
            }
        };
        let Some(object) = request.as_object() else {
            write_invalid_request(&mut writer, Value::Null).await?;
            continue;
        };
        let id = object.get("id").cloned();
        let method = object.get("method").and_then(Value::as_str);
        let valid =
            object.get("jsonrpc").and_then(Value::as_str) == Some("2.0") && method.is_some();
        if !valid {
            write_invalid_request(&mut writer, id.unwrap_or(Value::Null)).await?;
            continue;
        }

        if method == Some("initialize") && id.is_some() {
            return Ok((Cursor::new(frame).chain(reader), writer));
        }
        let Some(id) = id else {
            continue;
        };
        if method == Some("server/discover") {
            write_response(&mut writer, &discovery_response(id)).await?;
        } else {
            write_response(
                &mut writer,
                &json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "MCP initialization is required before this request"
                    }
                }),
            )
            .await?;
        }
    }
}

fn discovery_response(id: Value) -> Value {
    let ServerInfo {
        capabilities,
        server_info,
        ..
    } = BioMcpServer::new().get_info();
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "supportedVersions": STDIO_DISCOVERY_VERSIONS,
            "capabilities": capabilities,
            "_meta": {"io.modelcontextprotocol/serverInfo": server_info},
            "resultType": "complete",
            "ttlMs": 300000,
            "cacheScope": "public"
        }
    })
}

async fn write_invalid_request<W>(writer: &mut W, id: Value) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    write_response(
        writer,
        &json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {"code": -32600, "message": "Invalid Request"}
        }),
    )
    .await
}

async fn write_response<W>(writer: &mut W, response: &Value) -> anyhow::Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut bytes = serde_json::to_vec(response).context("failed to serialize MCP response")?;
    bytes.push(b'\n');
    writer.write_all(&bytes).await?;
    writer.flush().await?;
    Ok(())
}
