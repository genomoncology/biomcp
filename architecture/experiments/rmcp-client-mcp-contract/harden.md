# Harden — rmcp client contract library

## Decomposition

Extracted the reusable rmcp MCP client contract code out of the integration-test monolith into a small in-tree Rust library crate:

- Library: `crates/biomcp-mcp-contract-client/`
- Public API: `crates/biomcp-mcp-contract-client/src/lib.rs`
- Thin wrapper tests: `tests/rmcp_client_contract.rs`
- Build wiring: root `Cargo.toml` now uses `biomcp-mcp-contract-client` as a path dev-dependency.

The extracted library owns the reusable pieces downstream migration spikes need: black-box server spawning, rmcp stdio and streamable HTTP client setup, MCP content helpers, resource/study fixtures, deterministic OLS4 stubbing, optimized child teardown, and composable assertion helpers.

The integration test is now only a caller/wrapper: it selects stdio vs HTTP, passes environment overrides, and composes the shared assertions. It is 118 lines, below the 200-line thin-wrapper target.

## Public API

Import crate:

```rust
use biomcp_mcp_contract_client::{
    ContractHarness,
    assert_explore_core_contract,
    assert_initialize_and_tools,
    assert_resource_inventory_and_reads,
};
```

Main types and functions:

- `ContractHarness`
  - `ContractHarness::new(biomcp_bin, repo_root)`
  - `ContractHarness::from_repo_root(repo_root)`
  - `spawn_stdio_client(extra_env)`
  - `spawn_stdio_client_with_pid(extra_env)`
  - `spawn_http_server(extra_env)`
  - `http_client(mcp_url)`
- `EnvVar = (&'static str, String)`
- `RunningClient<T>`
- Content helpers:
  - `text_chunks(content)`
  - `image_chunks(content)`
  - `first_text(content)`
  - `tool_arguments(command)`
  - `call_biomcp(client, command)`
- Assertion helpers:
  - `assert_explore_core_contract(client)`
  - `assert_initialize_and_tools(client, repo_root)`
  - `assert_version_call(client)`
  - `assert_resource_inventory_and_reads(client, repo_root)`
  - `assert_read_only_and_policy_calls(client)`
  - `assert_invalid_resource_error(client)`
  - `assert_chart_calls(client)`
- Fixture/helpers:
  - `start_ols4_stub()`
  - `provision_study_fixture(repo_root)`
  - `study_dir_from_fixture(root)`
  - `terminate_process(pid)`

Stdio usage example:

```rust
use biomcp_mcp_contract_client::{ContractHarness, assert_initialize_and_tools};

#[tokio::test(flavor = "multi_thread")]
async fn stdio_contract() -> anyhow::Result<()> {
    let harness = ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let client = harness.spawn_stdio_client(&[]).await?;

    assert_initialize_and_tools(&client, &harness.repo_root).await?;

    client.cancel().await?;
    Ok(())
}
```

Streamable HTTP usage example:

```rust
use biomcp_mcp_contract_client::{ContractHarness, assert_explore_core_contract};

#[tokio::test(flavor = "multi_thread")]
async fn http_contract() -> anyhow::Result<()> {
    let harness = ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let (mut server, base_url) = harness.spawn_http_server(&[]).await?;

    let result = async {
        let client = harness.http_client(format!("{base_url}/mcp")).await?;
        assert_explore_core_contract(&client).await?;
        client.cancel().await?;
        Ok::<(), anyhow::Error>(())
    }
    .await;

    server.kill().await.ok();
    result
}
```

Chart fixture example:

```rust
use biomcp_mcp_contract_client::{
    ContractHarness, assert_chart_calls, provision_study_fixture, study_dir_from_fixture,
};

#[tokio::test(flavor = "multi_thread")]
async fn chart_contract() -> anyhow::Result<()> {
    let harness = ContractHarness::from_repo_root(env!("CARGO_MANIFEST_DIR"));
    let fixture_root = provision_study_fixture(&harness.repo_root)?;
    let study_dir = study_dir_from_fixture(fixture_root.path())?;
    let client = harness
        .spawn_stdio_client(&[("BIOMCP_STUDY_DIR", study_dir)])
        .await?;

    assert_chart_calls(&client).await?;

    client.cancel().await?;
    Ok(())
}
```

## Build System

Root `Cargo.toml` now declares the reusable helper as a path dev-dependency:

```toml
[dev-dependencies]
biomcp-mcp-contract-client = { path = "crates/biomcp-mcp-contract-client" }
```

The helper crate owns the rmcp client transport features:

```toml
rmcp = { version = "1.1.1", features = [
  "client",
  "transport-child-process",
  "transport-streamable-http-client-reqwest",
] }
```

This keeps the client contract dependency surface in the helper crate instead of in the integration-test file. Downstream spikes in this repo can import the helper directly with:

```toml
[dev-dependencies]
biomcp-mcp-contract-client = { path = "crates/biomcp-mcp-contract-client" }
```

No separate binary or shell wrapper is required. The existing `biomcp` server binary is still spawned as the black-box system under test because the MCP contract is a process/transport contract.

## Regression Check

Focused contract benchmark after extraction:

- `cargo test --no-run --test rmcp_client_contract`: passed.
- `cargo nextest run --test rmcp_client_contract`: passed, 6/6 tests.
- Best repeated focused run after extraction: 0.924 s.
- Optimize final baseline: 0.933 s.
- Result: no benchmark regression; the refactor is slightly faster within normal run noise.

Full validation after extraction:

- `make lint`: passed.
- `make test`: passed.
  - Rust nextest: 2358 passed, 28 skipped.
  - Python pytest: 297 passed.
  - MkDocs strict build: passed.
- `make spec`: passed, 71 passed, 4 skipped.

Result: zero correctness regression from refactoring.

## Reusable Assets

Downstream spikes inherit these concrete assets:

- A reusable Rust path crate for MCP client contract checks.
- rmcp stdio client spawning for `biomcp serve`.
- rmcp streamable HTTP client connection for `serve-http` at `/mcp`.
- A `ContractHarness` that centralizes binary path, repo root, environment overrides, and server startup.
- MCP content helpers for text and image chunks.
- A single `call_biomcp` helper for tool calls.
- Assertion helpers covering initialize, tools, resources, read-only policy, error shape, and chart image responses.
- Deterministic OLS4 stub support for `discover BRCA1` contract checks.
- Study fixture setup for chart contract checks.
- Optimized child-process teardown that preserves the previous benchmark improvement.
- A build pattern for downstream test/spec migration code to import the library instead of shelling out or copy-pasting helpers.
