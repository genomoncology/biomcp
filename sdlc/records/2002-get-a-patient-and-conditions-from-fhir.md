---
base: d63eb544
head: bcfc3c6b
---

Added `get patient <id>` with a `conditions` section. It reads one Patient and its Condition list from the FHIR R4 server in `BIOMCP_FHIR_BASE`. The client ports the request rules of an earlier read-only FHIR client prototype. It shares no code with it.

## What changed

- `src/sources/fhir.rs` is the FHIR client. `PatientId` enforces the FHIR `id` rule (1-64 letters, digits, hyphens, or periods) before any request. It also refuses IDs made only of periods. The URL library drops `.` and `..` path segments, so those IDs would read `{base}/Patient`. `FhirBase` builds every URL with encoded path segments and query pairs. One private request function sets no-store twice: the cache extension and a `Cache-Control: no-store` header. The redirect policy and the page walk accept only the configured origin and base path. The walk stops at a repeated link or at 20 pages of 100. The client uses the shared retry layer, now one helper in `src/sources/mod.rs`. `FhirError` carries fixed text only. No error can name a URL or an ID.
- `src/main.rs` holds the `hyper`, `hyper_util`, `h2`, `reqwest`, and `rustls` log targets at `warn` whatever `RUST_LOG` says. At `debug`, hyper and the connection pool logged the server's host and port.
- `src/entities/patient.rs` settles `conditions` as `data`, `empty`, `degraded`, or `unavailable`. The ticket lists the degraded causes.
- The CLI, list page, help, renderer, template, health row, and source-state registry gained the patient entity. `search patient` fails with a not-yet-available message and sends no request. Ticket 2003 replaces it.
- `src/mcp/shell/patient_gate.rs` is the one serve-http check. `run_http` marks the process at startup. `execute_cli` refuses `get`, `search`, and `batch patient` when the mark is set. Every MCP tool reaches `execute_cli`, including the modern-protocol dispatcher. Typed search gained a filterless `patient` branch. Stdio callers get the same message as the CLI.
- Docs: `docs/user-guide/patient.md`, the CLI reference, `CHANGELOG.md`, the configuration reference, the licensing inventory (FHIR as an operator server that publishes no terms), and the agent index. `manifest.json` and the MCP catalog say patient records run on stdio only.
- Spec: `spec/entity/patient.md` runs against synthetic FHIR routes added to the provider contract fixture. The repository holds no data from any real or demo server.

## Red then green

- Source guards (`src/sources/fhir/tests.rs`): with each guard stubbed out, 12 of 15 failed. ID check off: failed. Base containment off: failed. No-store off: extension and header both absent. Redirects unrestricted: failed. Repeat detection off: stopped at the page cap. Page cap at 50: 50 requests. Two retry layers: 16 requests where 4 were expected. A naive URL gave `patient=a%20b&c=d/e?f`. Green: 15 of 15.
- Entity outcomes (`src/entities/patient/tests.rs`): 5 of 7 failed before the outcome rules existed. Green: 7 of 7.
- serve-http refusal (`tests/patient_fhir_contract.rs`): with the check disabled, all four tests failed (shell tool, typed search, typed get, batch). Green: 4 of 4, each with zero fixture requests.
- Trace leak: a `tracing::trace!` of the request URL made the test fail with "stderr names the ID". Green: at `RUST_LOG=trace`, a degraded walk, a conditions 500, a patient 500, a 404, and an off-origin redirect leave no ID or server in stderr, in the error output, or under the cache directory.
- Review fixes, red at `29ab82df` on yellow: the ID test failed on `.`, and the leak test failed with "degraded walk at debug: stderr names the server host and port". Green at `413799e5`: the ID test passed. The leak test then failed its guard that stderr is not empty on error. That guard had counted on hyper's log lines. `a7634fef` changed it to check that the error reaches stdout or stderr. Green at `a7634fef`: 9 of 9 contract tests. The leak test now runs at `debug` and at `trace` and searches stdout and stderr for the bare host and port.
- Re-review found that the `a7634fef` guard passed on any output, including retry log lines. `bcfc3c6b` makes each failing leak case check its exact error: not found, HTTP 500, and the off-base redirect. Plain runs check stderr for `Error: <text>.`. JSON runs check the error message on stdout.
- Typed get over stdio reads the patient in two requests. Bad IDs, an unset base, and `search patient` send no request.

## Checks

- Yellow ran from a clean detached checkout at `a7634fef` while otherwise idle. The checkout is removed. `make lint` passed. `make test` passed: 3,737 Rust tests and 1,260 Python tests, with 31 Rust and 4 Python skipped. `make spec` passed.
- `tools/check-biodata-1.0 --already-isolated` passed on yellow at `a7634fef` under BioMCP's `tools/run-offline`, after `cargo build --locked --offline --no-default-features --bin biomcp`: 119 Rust tests and 152 Python tests passed, with 1 skipped. Yellow has no BioData checkout. BioMCP's launcher sets the same `BIOMCP_OFFLINE_NETWORK=1` isolation.
- Two earlier yellow runs failed `make spec` on the typed-tools example. It counted eight search branches and gave `conditions` to diagnostic alone. Commits `5eca5042` and `e6276ac2` fixed both. The patient spec also stopped capturing output before checking it.
- At `bcfc3c6b`, a test-only change, yellow ran under `~/.yellow-gate.lock` from a clean checkout that is removed. `make lint` passed. The patient contract tests passed 9 of 9. The FHIR and patient unit tests passed 30 of 30.
- The yellow run at `413799e5` passed lint, spec, and check-biodata and failed `make test` on the leak guard above.
- Every commit since `5eca5042` skipped the local pre-commit hook. The hook runs clippy, and Rust builds no longer run on the development box. Yellow's `make lint` covered them.

## Smoke

One manual read-only run against the HAPI server on `blue` (MIMIC-IV demo) passed. Five patients read with exit 0. Each condition count matched the server's `_summary=count` total (76, 21, 8, 10, 73). A missing ID failed with the fixed not-found message. The cache directory held zero files. Nothing was copied into the repository.

## Open

- Every smoke patient came back `degraded`. None of the demo's Conditions carries `clinicalStatus`, and the ticket degrades on a missing status. FHIR R4 invariant con-3 requires `clinicalStatus` only for `problem-list-item` conditions. Encounter diagnoses may omit it. A follow-up could degrade only when con-3 is broken. That changes the ticket's rule. It needs a new ticket.
- The health row is covered by a unit test and by the runner wiring. A full `biomcp health` probes every public API. No offline test runs it end to end.
- After review accepts, the landing agent pushes to `biodata/biomcp-1.0` and dispatches BioData's verification workflow at that tip, per `AGENTS.md`.
