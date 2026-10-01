//! Terminal rejection crosses real public channels without rescue or cache writes.
use super::*;
use rmcp::ServiceExt as _;
use rmcp::transport::TokioChildProcess;
use tracing::instrument::WithSubscriber as _;

fn private_channels(value: impl std::fmt::Debug) {
    let text = format!("{value:?}");
    for sentinel in ["QUERY_SENTINEL_0224", "SOURCE_SENTINEL_0224"] {
        assert!(!text.contains(sentinel), "unsafe channel: {text}");
    }
}

#[tokio::test]
#[serial_test::serial(source_env)]
async fn rejected_trial_alias_lookup_never_contacts_ctgov_or_populates_cache() {
    let _cache_mode = crate::sources::test_cache_mode::off();
    for case in table(4).into_iter().filter(|case| {
        matches!(
            case["input"]["operation"].as_str(),
            Some("trial_search" | "two_sequential_trial_search_calls")
        )
    }) {
        let fixture = CaseHttp::new(&case).await;
        let cache = tempfile::tempdir().unwrap();
        let mut env = TestEnv::new();
        fixture.environment(&mut env, cache.path());
        trial_alias_cache().lock().unwrap().clear();
        let calls = if let Some(calls) = case["input"]["calls"].as_array() {
            calls.clone()
        } else {
            vec![case["input"].clone()]
        };
        for (index, call) in calls.iter().enumerate() {
            let query = call["intervention"][0].as_str().unwrap();
            let filters = crate::entities::trial::TrialSearchFilters {
                intervention: Some(query.into()),
                no_alias_expand: false,
                source: crate::entities::trial::TrialSource::ClinicalTrialsGov,
                ..Default::default()
            };
            let trace = tempfile::NamedTempFile::new().unwrap();
            let writer = trace.as_file().try_clone().unwrap();
            let subscriber = tracing_subscriber::fmt()
                .with_ansi(false)
                .without_time()
                .with_env_filter("biomcp_cli=trace")
                .with_writer(move || writer.try_clone().unwrap())
                .finish();
            let result = crate::entities::trial::search_page(
                &filters,
                call["limit"].as_u64().unwrap() as usize,
                call["offset"].as_u64().unwrap() as usize,
                None,
            )
            .with_subscriber(subscriber)
            .await;
            let wanted = if calls.len() == 1 {
                &case["expected"]["failure"]
            } else {
                &case["expected"]["calls"][index]["failure"]
            };
            failure(
                result.err().expect("terminal identity rejection"),
                wanted,
                &case["id"],
            );
            assert!(trial_alias_cache().lock().unwrap().is_empty());
            assert_eq!(fixture.requests.lock().unwrap().len(), index + 1);
            assert!(
                std::fs::read(trace.path()).unwrap().is_empty(),
                "source rejection emits no product tracing"
            );
        }
        fixture.assert_requests(&case["id"]);
        trial_alias_cache().lock().unwrap().clear();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[serial_test::parallel(source_env)]
async fn rejected_identity_cli_and_mcp_channels_match_complete_private_error_objects() {
    let case = table(4)
        .into_iter()
        .find(|case| case["input"]["operation"] == "nested source rejection privacy")
        .unwrap();
    let binary = std::env::var_os("BIOMCP_BIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/biomcp")
        });
    let input = &case["input"];
    let expected = &case["expected"];
    let fixture = CaseHttp::new(&case).await;
    let cache = tempfile::tempdir().unwrap();
    let mut env = environment(&fixture, cache.path());
    env.push(("RUST_LOG", "biomcp_cli=trace,reqwest_retry=error".into()));
    let args = input["cli"]
        .as_array()
        .unwrap()
        .iter()
        .skip(1)
        .map(|arg| arg.as_str().unwrap())
        .collect::<Vec<_>>();
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        tokio::process::Command::new(&binary)
            .args(args)
            .envs(env)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        output.status.code(),
        Some(expected["cli"]["exit"].as_i64().unwrap() as i32)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&output.stdout).unwrap(),
        expected["cli"]["stdout"]
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        expected["cli"]["stderr"].as_str().unwrap()
    );
    private_channels((
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    ));
    fixture.assert_requests(&case["id"]);

    for (key, tool) in [("raw_mcp", "biomcp"), ("typed_mcp", "get")] {
        let fixture = CaseHttp::new(&case).await;
        let cache = tempfile::tempdir().unwrap();
        let mut env = environment(&fixture, cache.path());
        env.push(("RUST_LOG", "biomcp_cli=trace,reqwest_retry=error".into()));
        let trace = tempfile::NamedTempFile::new().unwrap();
        let mut command = tokio::process::Command::new(&binary);
        command
            .arg("serve")
            .envs(env)
            .env_remove("BIOMCP_TEST_PANIC_TOOL")
            .stderr(trace.as_file().try_clone().unwrap())
            .kill_on_drop(true);
        let client = ().serve(TokioChildProcess::new(command).unwrap()).await.unwrap();
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(30),
            client.peer().call_tool(
                CallToolRequestParams::new(tool)
                    .with_arguments(input[key].as_object().unwrap().clone()),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        client.cancel().await.unwrap();
        let wanted = &expected[key];
        assert_eq!(result.is_error, wanted["isError"].as_bool());
        assert_eq!(first_text(&result.content), wanted["content"][0]["text"]);
        assert!(result.structured_content.is_none());
        let wire = serde_json::to_value(&result).unwrap();
        assert!(wire.get("_meta").is_none());
        private_channels(&wire);
        let trace = std::fs::read_to_string(trace.path()).unwrap();
        private_channels(&trace);
        assert!(
            trace.is_empty(),
            "terminal guard emits no product trace: {trace}"
        );
        fixture.assert_requests(&case["id"]);
    }
}
