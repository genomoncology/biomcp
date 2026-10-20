# 2309 — MCP transport security upgrade

Move both rmcp consumers to the supported patched 2.x release while preserving tool schemas and existing stdio and HTTP behavior. The developer verified GHSA-9pj6-vhgr-3mwh describes session allocation before rejecting a non-initialization POST on versions through 1.7.0, fixed in 2.0.0; GHSA-9g45-5xwm-f3wc requires the 2.1.0 client fix.

Before upgrading, the developer committed an outside-in regression using the production HTTP router. It completes 128 rejected first ping requests and inspects the same session manager, asserting no allocation for rejected requests without a memory or timing threshold. Planned source changes stay in both manifests, the lock, the HTTP shell and its security test. Gator job 2309/1792530286431-1290759 runs that control against the old lock; its result is pending here.

M5 dependency and API job 2309/1792530306199-82317 failed before compiling BioMCP: resolved rmcp 2.2.0 calls SseStream::from_bytes_stream, absent in selected sse-stream 0.2.1. This is a failed compatibility check. A narrow compatible lock resolution remains required, followed by unchanged MCP and contract-client checks, installed-wheel smoke, routine qualification and fresh review. No later result is claimed.

Both hosts retain commands, advisory references, source overlays and raw results under experiment 2102-biomcp-2258-final-live-replay/mcp-security/2309. GitHub alerts must close through the actual landed fix, never manual dismissal.

## Reproduced session leak and SDK migration

Gator job 2309/1792530452524-1304021 reproduced the old-lock leak: the first rejected non-initializing POST retained a session, failing the security regression. The earlier --bin selection ran zero tests and is not reproduction evidence. Corrected raw red-lib.log and the exact red source 3b8516e81 remain retained. The regression also performs valid initialization and DELETE as its positive control before testing rejected requests.

Candidate c95e30e0b resolves rmcp and rmcp-macros 2.2.0 with compatible sse-stream 0.2.6. The initial suggestion to downgrade sse-stream was corrected before a second compiler run: the required from_bytes_stream alias appears in later releases. SDK content and resource adaptations stay in the MCP shell, modern shell, contract client and existing HTTP example; schemas and protocol assertions remain unchanged. Focused API, stdio, HTTP and contract-client qualification is queued on this exact source. The existing installed Linux wheel smoke and routine suite remain required after compatibility settles.

## Focused patched-source qualification

M5 job 2309/1792531121552-49033 passed 76 Rust/MCP tests and 29 Python stdio, HTTP and protocol cases on d9a95030a. This includes the local session-retention regression with the SDK-defined 422 response and expected error text; the first attempt guessed 400 and failed that new assertion, then the correction preserved the zero-retained-session claim. Existing protocol assertions remain unchanged.

Merged candidate 287d81d6273cac54bb2c52077e8b7fad2928dc03 incorporates actual main a5314c569 with both changelog entries preserved and no source conflict. Its required M5 routine and actual installed Linux wheel smoke are submitted on the exact retained bundle. Those results remain pending here. The security regression stays on localhost; the standard installed-artifact smoke performs only its already authorized read-only public provider queries. Fresh review and final advisory closure remain required.
