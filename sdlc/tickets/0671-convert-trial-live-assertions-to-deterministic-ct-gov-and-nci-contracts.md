---
flow: build
priority: 8
deps: ["0651", "0914", "0915", "0929"]
---
# Convert trial live assertions to deterministic CT.gov and NCI contracts

## Done when

The source-backed ClinicalTrials.gov and NCI CTS blocks in
spec/entity/trial.md run routinely against local provider-faithful responses.
They retain the existing cursor, contact, location, eligibility, alias,
source-routing, zero-result, and pagination claims without requiring a public
service.

## Proof required

CTGov and NCI already have substantial request and decoder coverage. Keep it
and add only the measured gaps:

- dated receipted responses for cursor/contact/detail and NCI routes, captured
  through production requests;
- proof that the entity dispatcher consumes those RequestPlans;
- local executor proof only for transport behavior not covered centrally;
- process-level CLI parsing and JSON/Markdown rendering.

Preserve the useful local trial fixtures. Do not change trial filter,
pagination, contact, or alias semantics as part of the conversion.

The design stage authors replacement assertions. trial.md becomes routine;
any live remainder moves verbatim to trial-live.md. Registry arrays, Makefile,
and architecture inventory agree.

## Authorized test changes

Design commits may restate source-backed blocks in trial.md, the CTGov/NCI
fixture routes and receipts, relevant source/entity tests, and registry
entries. Mechanical construction updates may land with implementation while
assertions remain unchanged.

The src line ceiling may rise by at most 160 lines.
