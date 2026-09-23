---
flow: build
priority: 2
deps: [1234]
---

# 1235: Get one patient and their conditions from one FHIR server

## Outcome

An agent on stdio MCP or the CLI reads one patient and that patient's conditions from the FHIR server the operator configured. `serve-http` refuses. Decision record: `sdlc/planning/adr/0002-read-one-patient-fhir-record-as-the-patient-entity.md`. Search is ticket 1236.

```
BIOMCP_FHIR_BASE=https://fhir.example.org/fhir biomcp get patient <id>
biomcp get patient <id> conditions
```

## Current Facts

- No `patient` entity or FHIR source exists on the 1.0 line. `grep -c -i patient src/cli/commands.rs` prints 0.
- `SectionOutcomeState` offers `data`, `empty`, `degraded`, `unavailable`, `inapplicable`, and `not_requested` (`src/entities/section_outcome.rs:7`).
- `apply_no_store` exists (`src/sources/mod.rs:429`). Base URLs come from `BIOMCP_*_BASE` variables through `env_base` (`src/sources/mod.rs:433`).
- The shared client already retries through `RetryTransientMiddleware` and honors `Retry-After` (`src/sources/mod.rs:14`, `:634`). It sets no redirect policy, so reqwest follows up to 10 redirects to any host.
- The public error projection hides the wrapped `reqwest::Error` from `Display` (`src/error.rs:691`). That error still carries the request URL in `Debug` and in its source chain. `grep -rn without_url src` finds nothing.
- MCP reaches entities through four paths: the `biomcp` shell tool, the typed `search` and `get` tools (`src/mcp/catalog.rs:14`, `:19`, `:24`), and `batch`, which takes the entity as a free string (`src/cli/commands.rs:235`).
- Spec fixtures serve canned JSON from a supervised local server and export `BIOMCP_*_BASE` (`spec/fixtures/setup-section-outcomes-spec-fixture.sh:228`).
- An earlier read-only FHIR client prototype has tested code for bundle entry classification, the paged walk with repeated-link detection, and an error type with no URL or body. Port that code. Do not depend on the prototype.

## Scope

- A `fhir` source module with its own client, built through the shared setup that loads the operator CA bundle. The client sets its own redirect policy: a redirect off the origin and base path of `BIOMCP_FHIR_BASE` fails. The module has one request function. Every FHIR GET goes through it. It applies `apply_no_store`, strips URLs from errors with `without_url()`, and uses the shared client's retry layer. The prototype's own `Retry-After` loop is dropped so a request never retries twice over.
- `BIOMCP_FHIR_BASE` names the one unauthenticated server. When it is unset, patient commands name the variable and exit non-zero. `biomcp health` reports the source as configured or not configured. No command or MCP argument accepts a URL.
- A patient ID must match `[A-Za-z0-9\-.]{1,64}`. Anything else is refused before a request. Every query value is URL-encoded.
- Next links and redirects must stay on the origin and base path of `BIOMCP_FHIR_BASE`. Anything else stops the walk and marks the section `degraded`.
- `get patient <id>` reads `Patient/{id}`, shows id, gender, and birth date, and lists `conditions` as a section. A redirect of `Patient/{id}` to another host fails with an error that names no URL.
- Until ticket 1236 lands, `search patient` on the CLI and the typed `search` tool fails with a message that search is not yet available. On `serve-http`, the transport refusal checks the entity first, before any search runs.
- The `conditions` section reads `Condition?patient={id}&_count=100` and follows next links up to 20 pages. A repeated link or the page cap stops the walk and marks the section `degraded`. A condition without `clinicalStatus` marks it `degraded`. No matches is `empty`. A transport or status failure is `unavailable`.
- One transport check refuses patient calls on `serve-http`. It covers the shell tool, typed `search` and `get`, and `batch`. The message cites `sdlc/issues/2026-09-11-health-record-entity-needs-authenticated-http-transport.md`.
- `patient` becomes a valid entity for the existing typed `search` and `get` tools on stdio. No new MCP tool is added.
- Help, `biomcp list`, the MCP catalog text, `docs/user-guide/patient.md`, and `CHANGELOG.md` show `get patient`.

## Exclusions

No search, which is ticket 1236. No FHIRPath evaluator. No sign-in or token. No section beyond `conditions`. No BioData dependency. No new MCP tool. No live server in any gate.

## Acceptance

Synthetic bundles only, served by the existing spec fixture runner:

1. A unit test proves the request function sets no-store on every request.
2. A two-page Condition bundle returns both pages. A repeated next link, a next link to another origin, a redirect to another origin, and a 21st page each yield `degraded` with a warning that names no URL.
3. A condition missing `clinicalStatus` yields `degraded`. Zero entries yields `empty`. A 500 yields `unavailable`.
4. A `Patient/{id}` redirect to another host fails with an error naming no URL.
5. On stdio, typed `search patient` returns the not-yet-available message and sends no request.
6. An ID with a slash, a query character, or 65 characters is refused before any request.
7. With a patient ID planted in every fixture response and path, a run with `RUST_LOG=trace` leaves that ID out of stderr, the rendered error, and the cache directory.
8. Over `serve-http`, one test per path proves the refusal: shell tool, typed `search`, typed `get`, and `batch patient`. Over stdio MCP, typed `get` succeeds.
9. With `BIOMCP_FHIR_BASE` unset, the command names the variable and exits non-zero, and `biomcp health` says not configured.
10. `spec/entity/patient.md` covers get and conditions.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. One manual smoke run against the live HAPI server with the MIMIC-IV demo is noted in the record, with no patient data copied into the repository.

## Dependencies

1234. The merge brings the operator CA bundle and the MCP panic survival this entity relies on.

## Complexity

- Contract score: 1 (one get, one section, one refusal)
- State and timing score: 1 (bounded paged walk)
- Reach score: 2 (source, entity, CLI, MCP transport gate, health, docs, spec)
- Proof score: 2 (leak proofs across logs, errors, and cache, plus four refusal paths)
- Cost of error score: 2 (a leak exposes patient identifiers)
- Total: 8
- Minimum level floor: level 4 (a leak of patient identifiers is a credible security risk, and the floor is a fixed minimum)
- Final level: 4
- Reasons: security floor on patient identifiers, plus leak and refusal proofs across four MCP paths
- Selected model: claude-opus

## Review

- Design review: rejected and split (2026-09-23). Search moved to 1236. Added ID validation, same-origin paging and redirects, a page cap, one retry layer, one no-store request function, trace-level leak proof, and one serve-http check with a test per MCP path.
- Design re-review: accepted with required change (2026-09-23). Set level 4 by the security floor, defined `search patient` before 1236, restored the CA-bundle client with its own redirect policy, added the `Patient/{id}` redirect case.
- Code review: pending
