use super::{http_allowed_hosts, http_router};
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use serde_json::json;
use std::{net::Ipv4Addr, sync::Arc};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn rejected_http_initializations_do_not_accumulate_sessions() {
    let sessions = Arc::new(LocalSessionManager::default());
    let shutdown = CancellationToken::new();
    let _cleanup = shutdown.clone().drop_guard();
    let router = http_router(
        http_allowed_hosts(Ipv4Addr::LOCALHOST.into(), vec![], false).unwrap(),
        &shutdown,
        sessions.clone(),
    );
    let listener = tokio::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server_shutdown = shutdown.clone();
    let server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(server_shutdown.cancelled_owned())
            .await
            .unwrap();
    });
    let client = reqwest::Client::new();
    let endpoint = format!("http://{address}/mcp");
    let initialized = client
        .post(&endpoint)
        .header("Accept", "application/json, text/event-stream")
        .json(&json!({
            "jsonrpc": "2.0", "id": "valid", "method": "initialize",
            "params": {"protocolVersion": "2025-06-18", "capabilities": {},
                       "clientInfo": {"name": "session-security-test", "version": "0"}}
        }))
        .send()
        .await
        .unwrap();
    assert!(initialized.status().is_success());
    let session_id = initialized.headers()["mcp-session-id"].clone();
    let _ = initialized.bytes().await.unwrap();
    assert_eq!(sessions.sessions.read().await.len(), 1);
    let terminated = client
        .delete(&endpoint)
        .header("Mcp-Session-Id", session_id)
        .header("MCP-Protocol-Version", "2025-06-18")
        .send()
        .await
        .unwrap();
    assert!(terminated.status().is_success());
    let _ = terminated.bytes().await.unwrap();
    assert!(sessions.sessions.read().await.is_empty());
    // GHSA-9pj6-vhgr-3mwh: each rejected first POST formerly left a permanent
    // LocalSessionManager entry. Inspect the production router's real manager
    // after each completed HTTP response, without a memory or timing threshold.
    for id in 0..128 {
        let response = client
            .post(&endpoint)
            .header("Accept", "application/json, text/event-stream")
            .json(&json!({"jsonrpc": "2.0", "id": id, "method": "ping"}))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), reqwest::StatusCode::UNPROCESSABLE_ENTITY);
        let rejection = response.text().await.unwrap();
        assert!(rejection.contains("initialize request"), "{rejection}");
        assert!(
            sessions.sessions.read().await.is_empty(),
            "rejected first POST {id} leaked a session"
        );
    }
    shutdown.cancel();
    server.await.unwrap();
}
