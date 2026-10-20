# 2309 — MCP transport security upgrade

Move both rmcp consumers to the supported patched 2.x release while preserving tool schemas and existing stdio and HTTP behavior. The developer verified GHSA-9pj6-vhgr-3mwh describes session allocation before rejecting a non-initialization POST on versions through 1.7.0, fixed in 2.0.0; GHSA-9g45-5xwm-f3wc requires the 2.1.0 client fix.

Before upgrading, the developer committed an outside-in regression using the production HTTP router. It completes 128 rejected first ping requests and inspects the same session manager, asserting no allocation for rejected requests without a memory or timing threshold. Planned source changes stay in both manifests, the lock, the HTTP shell and its security test. Gator job 2309/1792530286431-1290759 runs that control against the old lock; its result is pending here.

M5 dependency and API job 2309/1792530306199-82317 failed before compiling BioMCP: resolved rmcp 2.2.0 calls SseStream::from_bytes_stream, absent in selected sse-stream 0.2.1. This is a failed compatibility check. A narrow compatible lock resolution remains required, followed by unchanged MCP and contract-client checks, installed-wheel smoke, routine qualification and fresh review. No later result is claimed.

Both hosts retain commands, advisory references, source overlays and raw results under experiment 2102-biomcp-2258-final-live-replay/mcp-security/2309. GitHub alerts must close through the actual landed fix, never manual dismissal.
