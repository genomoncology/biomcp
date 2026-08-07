---
base: 94d4f0cc52b3509c6e7c376dabb79dcdd197d9da
head: 39247f0e2c382d6895f2092ba1f556eb9e7819b3
---
Root cause (reproduced in-tree): `rmcp` resolves to **1.7.0** in `Cargo.lock` (pinned as `rmcp = { version = "1.1.1", ... }` in `Cargo.toml`, but it resolves forward to 1.7.0). `StreamableHttpServerConfig::default()` ships a DNS-rebinding guard that sets `allowed_hosts = ["localhost", "127.0.0.1", "::1"]`. Our `run_http()` in `src/mcp/shell.rs` constructs the config with bare `::default()` and never touches `allowed_hosts`, so it inherits that restrictive allowlist and rejects every other Host header. This was filed as GitHub issue #240.

Imported from March ticket 429. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/429-serve-http-stop-rejecting-non-localhost-host-headers
