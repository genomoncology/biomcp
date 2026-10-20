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

## Security proof scope

The ticket proof now names the actual advisory and regression: repeated rejected non-initializing first POSTs allocate no retained sessions; one valid initialization and DELETE establish the positive control. It does not claim a general bound on abandoned valid sessions. This corrects proof wording without changing implementation, tool schemas or protocol assertions.

The separate 2310 recipe repair owns source prevention of root-owned temporary wheel output. Any 2309 aggregate cleanup failure must remain failed; verified successful build and installed-smoke stages and a later owned-cleanup receipt may be cited separately. Final admission remains held for the repaired recipe and full rehearsal.

## Review correction and exact wire comparison

The reviewer found that the SDK upgrade silently removed `inputSchema.title` from six tools. Outside-in M5 job 2309/1792531430163-78237 compared the retained old CLI with the routine candidate through real stdio processes and failed on that difference. The original replies and failed job remain under `mcp-security/2309/review-wire-red`. Resource list/read replies, invalid cursor errors and invalid tool argument results matched.

Commit a993dd25f64231a9a847e167eea0c0f2eeaf4910 requests the existing SDK `schema_for_type<T>` for each of the six named input types. It preserves their generated schemas without a second list of schema titles. The shared contract client now rejects missing named input titles on its actual stdio and HTTP tool-list checks. The custom ERepo schema remains unchanged.

M5 job 2309/1792532093948-5347 imported that exact source and completed `make landing-gate`, including lint, the full routine suites, documentation and specifications. Its final old/new stdio comparison also passed exact equality for the tools, resources and checked errors. The aggregate job nevertheless failed because the review script then called `hashlib.file_digest`, which the system Python lacks. That failed wrapper remains retained under `review-hash-instrumentation-failure`; it is not a passing job. A portable hash-print correction requires only the small comparison, with no build or routine suite repeated.

Gator job 2309/1792531290483-1349952 built, installed and completed the existing pinned manylinux wheel smoke on pre-review source 287d81d6273cac54bb2c52077e8b7fad2928dc03, then failed while removing Docker-owned temporary output. Its aggregate result remains failed. Cleanup continuation 2309/1792532414761-1497381 passed and removed the exact scratch directory. The retained native binary SHA-256 is `9aa8de14ee558d0ddcd5665ce6b790b1e53fc83102719d602ed42b9a4671f4f0`. These receipts prove those component stages and cleanup, not an installed-wheel execution of the schema repair. The repaired source still requires the final release rehearsal and post-landing security-alert closure.

The portable comparison-only follow-up 2309/1792532698550-19044 passed without rebuilding or repeating the routine suite. It retained exact old/new tool schema, resource list/read and checked error replies. The old CLI SHA-256 was `338e94737bb9bdd5f4d72bd0d95e7adec50b34d3271feae6b920c71dab83b2ba`; the repaired CLI SHA-256 was `6a3a40fed5905d33c43655cd4cddd73fa75f60ee57d95af62e4fa90412156f1a`. The full routine component proof remains bound to a993dd25f64231a9a847e167eea0c0f2eeaf4910, followed only by this review evidence text.
