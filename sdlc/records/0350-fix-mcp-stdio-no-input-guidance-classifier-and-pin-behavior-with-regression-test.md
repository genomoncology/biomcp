---
base: 2c35f8ee8bd26f52b41bce8ec6dd17e521579be3
head: adf38d84c5e62fbdef79ab2024b2c4e129c9c839
---
`biomcp mcp` and `biomcp serve` are intended to print a recovery hint pointing operators to `serve-http` when launched without an MCP client on stdin. `src/mcp/shell.rs` defines `mcp_stdio_guidance()` and an `is_handshake_startup_error()` classifier for exactly this case, but in the v0.8.22 candidate the binary now prints a bare `Error: connection closed: initialized request` and exits 1, because the underlying `rmcp` transport's no-handshake error string changed and the classifier no longer matches. This is a regression in the first-run operator experience captured by issue `348-mcp-stdio-eof-guidance-regression.md`. It is not a release blocker, but it is a low-effort fix that should land alongside an integration test pinning the no-input behavior so the next upstream wording change does not silently re-break the hint.

Imported from March ticket 350. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/350-fix-mcp-stdio-no-input-guidance-classifier-and-pin-behavior-with-regression-test
