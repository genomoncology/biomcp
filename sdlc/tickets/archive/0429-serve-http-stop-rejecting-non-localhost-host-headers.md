---
flow: quickfix
priority: 5
---
# serve-http: stop rejecting non-localhost Host headers

Root cause (reproduced in-tree): `rmcp` resolves to **1.7.0** in `Cargo.lock` (pinned as `rmcp = { version = "1.1.1", ... }` in `Cargo.toml`, but it resolves forward to 1.7.0). `StreamableHttpServerConfig::default()` ships a DNS-rebinding guard that sets `allowed_hosts = ["localhost", "127.0.0.1", "::1"]`. Our `run_http()` in `src/mcp/shell.rs` constructs the config with bare `::default()` and never touches `allowed_hosts`, so it inherits that restrictive allowlist and rejects every other Host header. This was filed as GitHub issue #240.

Completed under March on 2026-06-22, as March ticket 429. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/429-serve-http-stop-rejecting-non-localhost-host-headers
