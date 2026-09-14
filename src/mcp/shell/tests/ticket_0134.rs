use super::*;

#[tokio::test]
async fn raw_mcp_trial_document_rejection_is_structured_json() {
    let result = BioMcpServer::new()
        .biomcp(rmcp::handler::server::wrapper::Parameters(ShellCommand {
            command: "biomcp get trial NCT1 document fixture.pdf".into(),
            json: true,
        }))
        .await
        .expect("raw MCP trial document rejection");
    let value = serde_json::to_value(result).expect("serialize raw MCP rejection");
    assert_eq!(value["isError"], true);
    let text = value["content"][0]["text"]
        .as_str()
        .expect("raw MCP rejection text");
    let error: serde_json::Value =
        serde_json::from_str(text).expect("raw MCP rejection is structured JSON");
    assert_eq!(error["error"]["code"], "invalid_argument");
    assert!(error["error"]["message"].as_str().is_some_and(|message| {
        message.contains("trial document") && message.contains("CLI-only")
    }));
}
