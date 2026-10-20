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
    // GHSA-9pj6-vhgr-3mwh: each rejected first POST formerly left a permanent
    // LocalSessionManager entry. Inspect the production router's real manager
    // after each completed HTTP response, without a memory or timing threshold.
    for id in 0..128 {
        let response = client
            .post(format!("http://{address}/mcp"))
            .header("Accept", "application/json, text/event-stream")
            .json(&json!({"jsonrpc": "2.0", "id": id, "method": "ping"}))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_client_error());
        let _ = response.bytes().await.unwrap();
        assert!(
            sessions.sessions.read().await.is_empty(),
            "rejected first POST {id} leaked a session"
        );
    }
    shutdown.cancel();
    server.await.unwrap();
}
