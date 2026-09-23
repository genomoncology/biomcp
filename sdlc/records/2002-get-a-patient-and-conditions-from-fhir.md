---
base: d63eb544
head: e6276ac2
---

Added `get patient <id>` with a `conditions` section. It reads one Patient and its Condition list from the FHIR R4 server in `BIOMCP_FHIR_BASE`. The client ports the request rules of an earlier read-only FHIR client prototype. It shares no code with it.

## What changed

- `src/sources/fhir.rs` is the FHIR client. `PatientId` enforces the FHIR `id` rule (1-64 letters, digits, hyphens, or periods) before any request. `FhirBase` builds every URL with encoded path segments and query pairs. One private request function sets no-store twice: the cache extension and a `Cache-Control: no-store` header. The redirect policy and the page walk accept only the configured origin and base path. The walk stops at a repeated link or at 20 pages of 100. The client uses the shared retry layer, now one helper in `src/sources/mod.rs`. `FhirError` carries fixed text only. No error can name a URL or an ID.
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
- Typed get over stdio reads the patient in two requests. Bad IDs, an unset base, and `search patient` send no request.

## Checks

- Yellow ran from a clean detached checkout at `e6276ac2` while otherwise idle. The checkout is removed. `make lint` passed. `make test` passed: 3,737 Rust tests and 1,260 Python tests, with 4 skipped. `make spec` passed.
- `tools/check-biodata-1.0 --already-isolated` passed on yellow at `e6276ac2` under BioMCP's `tools/run-offline`, after `cargo build --locked --offline --no-default-features --bin biomcp`: 119 Rust tests and 152 Python tests passed, with 1 skipped. Yellow has no BioData checkout. BioMCP's launcher sets the same `BIOMCP_OFFLINE_NETWORK=1` isolation.
- Two earlier yellow runs failed `make spec` on the typed-tools example. It counted eight search branches and gave `conditions` to diagnostic alone. Commits `5eca5042` and `e6276ac2` fixed both. The patient spec also stopped capturing output before checking it.
- The last two commits skipped the local pre-commit hook. The hook runs clippy, and Rust builds no longer run on the development box. Yellow's `make lint` covered them.

## Smoke

One manual read-only run against the HAPI server on `blue` (MIMIC-IV demo) passed. Five patients read with exit 0. Each condition count matched the server's `_summary=count` total (76, 21, 8, 10, 73). A missing ID failed with the fixed not-found message. The cache directory held zero files. Nothing was copied into the repository.

## Open

- Every smoke patient came back `degraded`. None of the demo's Conditions carries `clinicalStatus`, and the ticket degrades on a missing status. FHIR R4 invariant con-3 requires `clinicalStatus` only for `problem-list-item` conditions. Encounter diagnoses may omit it. A follow-up could degrade only when con-3 is broken. That changes the ticket's rule. It needs a new ticket.
- The health row is covered by a unit test and by the runner wiring. A full `biomcp health` probes every public API. No offline test runs it end to end.
- After review accepts, the landing agent pushes to `biodata/biomcp-1.0` and dispatches BioData's verification workflow at that tip, per `AGENTS.md`.
