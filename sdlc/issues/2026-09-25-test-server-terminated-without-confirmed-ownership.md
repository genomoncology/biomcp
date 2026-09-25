# Test server terminated without confirmed ownership

Status: open; impact unverified. Reported September 25, 2026 during a shared build-host gate attempt from another repository. This is an incident record, not a runtime change or release handoff.

## Reported action and evidence

The worker saw PID 2943903 with PPID 1, running `biomcp serve-http --host 127.0.0.1 --port 45225` from a nextest archive. The host reported a start time of Friday September 25 at 15:11:09. Its working directory was the BioMCP checkout. The other gate runner refused to start because its process check matched the archive path.

The worker inferred that the server was abandoned and sent one TERM signal without checking its listener, controlling test, or owner. It reported that the process exited within one second. The coordinator's instruction to preserve the process arrived after the signal. These observations come from the worker's report; the coordinator has not established the affected test or service state.

PPID 1 and an archive path did not establish cleanup authority. Further process and service actions from that worker were stopped. Its already-running repository test was allowed to finish normally; no replacement server was started. The coordinator informed Ian.

## Remaining work

The release/test owner should identify the server's controlling test or service, determine whether termination affected an active run, and decide whether any test needs to be repeated. Do not infer release success or failure from the termination alone. A later ticket can make runner refusals identify ownership more clearly; a refusal must not authorize terminating another process.

## Resolution 2026-09-25 (test owner)

The server was the leaked child of
`rmcp_client_contract::raw_article_batch_contract_is_safe_over_http`
during the first ticket-1251 gate run on this host (checkout at
bbb86486, `make test` window 15:02-15:11). That run's nextest output
marks the test `FAIL + LEAK`: the client panicked on a stale schema
assertion while its `serve-http` child stayed behind — PID 2943903,
started 15:11:09, exactly that window. The leak was a symptom of the
already-diagnosed stale `oneOf` consumer, fixed at e2fd276e
(crates/biomcp-mcp-contract-client). No active run was affected: the
gate had already failed and completed before the signal, and the
fixed suite ran green at 74f600db with no leak flag. Nothing needs
repeating. The follow-up idea stands: a runner refusal must identify
ownership, not invite termination; filed for 1254's batch.

The boundary constant: the true packaged count after the 1251 merge
is 1,361 — `sdlc/planning/adr/` is excluded from the crate, so the
ADR adds nothing; the +1 reasoned value was corrected after a real
`cargo package` measurement and a green count test.
