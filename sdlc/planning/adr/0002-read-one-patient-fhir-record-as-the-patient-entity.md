# 0002 — Read one patient's FHIR record as the `patient` entity

- Status: accepted
- Date: 2026-09-23
- Decided by: Ian Maurer. He can overturn any numbered decision below.
- Line: 1.0 only (`biodata/biomcp-1.0`). Main (0.9) does not get this entity.

## Context

The ideal state names one patient's own FHIR record as a second destination for BioMCP. An agent should reach it through the same `search` and `get` grammar it already uses. An earlier read-only FHIR client prototype proved the transport: capability read, paged search, bundle entry classification, and errors that never carry a URL or a body.

An outside design review on 2026-09-11 recommended one new entity and set a gate. Record calls may run over local stdio only until the HTTP transport has authenticated per-user sessions. The gate lives in `sdlc/issues/2026-09-11-health-record-entity-needs-authenticated-http-transport.md`.

## Decision

1. The capability is a new entity named `patient`, built on the 1.0 line.
2. `search patient` finds people by typed facts only: gender, a birth date range, and a coded condition. It takes no free text. With `--count` it returns the count the server reports, labeled as server-reported.
3. `get patient <id>` returns demographics and lists the available sections. Each section reads one patient's chart.
4. No separate cohort entity exists. A cohort is a `search patient` result.
5. The entity is visible in 1.0 help, docs, and tools while 1.0 is unreleased.
6. A live HAPI FHIR server loaded with the MIMIC-IV demo data serves manual smoke runs only. No gate calls it. Committed tests use synthetic bundles through the existing spec fixture runner.
7. The first slice has no sign-in. The operator sets one unauthenticated FHIR base URL in an environment variable. The agent never passes a URL.
8. Patient IDs never reach logs, error messages, or the disk cache. Every FHIR request is sent with no-store.
9. Patient commands work over stdio MCP and the CLI. `serve-http` refuses them until the gate in the 2026-09-11 issue is met.

## Options weighed

**Name it `record` with clinical kinds, as the review proposed.** A record is what a patient has. Agents and clinicians ask for patients first. `patient` also gives search a natural subject: people who match typed facts.

**Add a cohort entity beside `patient`.** Costs a second grammar for the same query. A count on `search patient` answers the cohort question.

**Let the agent pass a server URL.** Costs a server-side request forgery surface and lets a prompt steer patient data to an arbitrary host. An operator setting buys a fixed trust boundary.

**Gate tests on the live HAPI server.** Costs determinism and ties gates to a third party's uptime and data load. Synthetic bundles cost fixture upkeep and buy repeatable proofs.

## Cost accepted

- The entity cannot serve a remote MCP client until authenticated HTTP sessions exist.
- An operator must run or pick a FHIR server and set its URL.
- Search depends on the server declaring `_has` for the condition filter. A server without it refuses that filter.
- No response is cached, so repeated reads cost repeated requests.

## First slice

Ticket 1235: `get patient <id>` and the `conditions` section. Ticket 1236: `search patient`.

## Follow-ups (not yet ticketed)

- Labs and vitals, with a guard that refuses an unknown code system.
- Medications and encounters.
- Typed MCP tools for patient calls on stdio.
- SMART on FHIR sign-in with PKCE.
- Retiring the earlier prototype once the entity covers its reads.
