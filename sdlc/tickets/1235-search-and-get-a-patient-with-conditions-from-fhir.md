---
flow: build
priority: 2
deps: [1234]
---

# 1235: Search and get a patient, with conditions, from one FHIR server

## Outcome

An agent on stdio MCP or the CLI finds patients by typed facts and reads one patient's conditions from the FHIR server the operator configured. Decision record: `sdlc/planning/adr/0002-read-one-patient-fhir-record-as-the-patient-entity.md`.

```
BIOMCP_FHIR_BASE=https://fhir.example.org/fhir biomcp search patient --gender female --born-after 1950-01-01 --condition http://snomed.info/sct|44054006 --count
biomcp get patient <id>
biomcp get patient <id> conditions
```

## Current Facts

- No `patient` entity or FHIR source exists on the 1.0 line: `grep -rn -i patient src/cli/commands.rs` finds nothing.
- `SectionOutcomeState` offers `data`, `empty`, `degraded`, `unavailable`, `inapplicable`, and `not_requested` (`src/entities/section_outcome.rs:7`).
- `apply_no_store` exists (`src/sources/mod.rs:429`). Base URLs come from `BIOMCP_*_BASE` variables through `env_base` (`src/sources/mod.rs:433`).
- The public error projection hides the wrapped `reqwest::Error` from `Display` (`src/error.rs:691`). That error still carries the request URL in `Debug` and in its source chain. `grep -rn without_url src` finds no call site.
- Spec fixtures serve canned JSON from a supervised local server and export `BIOMCP_*_BASE` (`spec/fixtures/setup-section-outcomes-spec-fixture.sh:228`).
- `src/mcp/mod.rs` has separate `run_stdio` and `run_http` entry points.
- An earlier read-only FHIR client prototype has tested code for the REST client with bounded `Retry-After` retries, bundle entry classification, the capability snapshot, the paged search with repeated-link detection, and an error type with no URL or body. Port that code. Do not depend on the prototype.

## Scope

- A `fhir` source module holding the ported client, bundle, capability, search, and error code. It uses the shared TLS client setup so the operator CA bundle applies. It sends every request through `apply_no_store`. It strips URLs from every transport error.
- `BIOMCP_FHIR_BASE` names the one unauthenticated server. When it is unset, patient commands fail with a message that names the variable, and `biomcp health` reports the source as not configured. No command or MCP argument accepts a URL.
- `search patient` takes `--gender`, `--born-after`, `--born-before`, `--condition <system|code>`, and `--count`. It rejects free text.
  - Before a condition search, read `metadata`. If `Patient` does not declare `_has`, refuse the filter with a clear message and send no search.
  - List query: `Patient?gender=..&birthdate=ge..&birthdate=le..&_has:Condition:patient:code=..&_elements=id,gender,birthDate`.
  - `--count` sends the same filters with `_summary=count` and labels the number as server-reported.
- `get patient <id>` reads `Patient/{id}`, shows id, gender, and birth date, and lists `conditions` as a section.
- The `conditions` section reads `Condition?patient={id}&_count=100` and follows `next` links. A repeated link stops the walk and marks the section `degraded`. A condition without `clinicalStatus` marks the section `degraded`. No matches is `empty`. A transport or status failure is `unavailable`.
- Patient IDs never appear in log lines, error messages, or the disk cache.
- `serve-http` refuses patient commands with a message citing `sdlc/issues/2026-09-11-health-record-entity-needs-authenticated-http-transport.md`.
- Help, `biomcp list`, the MCP catalog, `docs/user-guide/patient.md`, and `CHANGELOG.md` show the entity.

## Exclusions

No FHIRPath evaluator. No sign-in or token. No section beyond `conditions`. No BioData dependency. No typed MCP tool. No live server in any gate.

## Acceptance

Synthetic bundles only, served by the existing spec fixture runner:

1. Each filter maps to the exact query string above. A test pins the strings.
2. A capability statement without `_has` makes `--condition` fail with the refusal message, and the fixture's request log shows no Patient search.
3. `--count` prints the `Bundle.total` value labeled server-reported, and never a count of returned entries.
4. A two-page Condition bundle returns both pages. A repeated `next` link yields `degraded` with a warning that names no URL.
5. A condition missing `clinicalStatus` yields `degraded`. Zero entries yields `empty`. A 500 yields `unavailable`.
6. With a patient ID planted in every fixture response and path, a run leaves that ID out of stderr, the rendered error, and the cache directory. Every request carries the no-store mode.
7. Over `serve-http`, a patient call returns the refusal. Over stdio MCP, the same call succeeds.
8. With `BIOMCP_FHIR_BASE` unset, the command names the variable and exits non-zero.
9. A spec page under `spec/entity/patient.md` covers search, get, and conditions.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA. One manual smoke run against the live HAPI server with the MIMIC-IV demo is recorded in the ticket's record, with no patient data copied into the repository.

## Dependencies

1234. The merge brings the operator CA bundle and the MCP panic survival this entity relies on.

## Complexity

- Contract score: 2 (new entity, new search grammar, new section, refusal contract)
- State and timing score: 1 (paged walk with repeat detection, bounded retries)
- Reach score: 2 (source, entity, CLI, MCP transport gate, health, docs, spec)
- Proof score: 2 (leak proofs across logs, errors, and cache, plus transport refusal)
- Cost of error score: 2 (a leak exposes patient identifiers)
- Total: 9
- Minimum level floor: level 4 (patient identifier leak is a credible security risk)
- Final level: 4
- Reasons: security floor on patient data. The design reviewer may split the transport and leak guards from the entity surface.
- Selected model: claude-opus

## Review

- Design review: pending
- Code review: pending
