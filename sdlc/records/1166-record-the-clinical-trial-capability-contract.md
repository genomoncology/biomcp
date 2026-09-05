---
flow: build
priority: 1
status: complete
---
# Record the clinical-trial capability contract

## Outcome

BioMCP carries one bounded inventory of the clinical-trial capabilities that BioData adoption must preserve or improve. The inventory describes user operations and observable behavior. It does not list provider fields or require historical output parity.

## Current facts

At BioMCP `4066933749a6e9d5b2e7088d6bd080e571e7d6d6`, trial behavior is spread across CLI help, executable specifications, Rust and Python tests, public documentation, and provider modules. No short artifact tells the next model-design ticket which product capabilities exist.

The inspected surface includes filtered search, provider-specific search behavior, result paging and counts, trial overview, named detail sections, eligibility, locations and contacts, arms and interventions, references, posted documents, batch detail, Markdown and JSON, raw and typed MCP, cross-entity trial pivots, and shared transport behavior.

CTGov and NCI do not support identical operations. BioMCP never changes providers after a direct operation fails. NCI condition search and CTGov intervention expansion each have narrower internal fallback behavior. Direct NCI detail can currently return `None` for a requested unsupported section. The product cannot distinguish unsupported, absent, and failed section states yet.

## Scope and decisions

Add `sdlc/planning/clinical-trial-capabilities.md`. Keep the main inventory to at most fifteen capability rows. Each row must have a stable identifier, user operation, supported source, observable behavior, code owner, executable contract, and public document. Add one compact source-support table for search groups and detail sections.

Include separate machine-readable declarations for CLI search flags, the typed MCP search subset, CLI detail section names, typed MCP detail section names, and source names. Extend the existing trial-help contract tests to compare each declaration only with its applicable shipped surface. Test the complete CLI search declaration against Clap. Test the typed-search subset against the typed schema. Test shared source names against both surfaces. Preserve explicit typed MCP exclusions, including CLI-only document forms. Do not require set equality between CLI and typed MCP.

Require each inventory row to point at an existing code path, a specific heading in an executable trial specification, and a public document. The focused check must verify that each path and specification heading exists. Keep this structural. The inventory must not duplicate expected output or parse behavior from the specification body.

Do not create JSON, enumerate provider response fields, generate public documentation, duplicate executable specifications, or add a completeness percentage. Do not treat unsupported NCI sections as supported merely because the CLI accepts their names. Record that gap for the later partial-response ticket.

The inventory describes the current product. It does not freeze every behavior. Later area tickets may improve an answer when they explain the change and keep equal or better information and capability coverage.

## Acceptance

1. One planning document contains no more than fifteen capability rows and one compact source-support table.
2. Every row names an existing user operation, code owner, public document, and exact executable trial specification heading. A focused structural check proves that each named path and heading exists.
3. A focused test fails when a surface-specific declaration drifts from the matching shipped CLI or typed MCP surface. It also proves the documented exclusions between those surfaces.
4. The inventory states the two internal fallback behaviors, the no-provider-failover rule, and the current partial-response gap.
5. Inspection proves that the work adds no field ledger, generated documentation, output hash, or provider capture.
6. Independent design and code reviews accept the result. Focused red-green evidence and `make lint`, `make test`, and `make spec` pass.

## Dependencies

Ticket 1160 established the first BioData-backed value on the current BioMCP main branch. This ticket changes no clinical model or runtime behavior. Both factory channels remain paused. The manual subagent SDLC owns this work.

## Review

- Design review: accepted before implementation.
- Implementation: complete and awaiting independent code review.
- Red: `uv run --no-sync pytest -q tests/surface/test_trial_help_contract.py` collected four tests; the two new tests failed because `sdlc/planning/clinical-trial-capabilities.md` did not exist.
- Green: the same focused command passed all four tests after the bounded document and structural checks were added.
- Code-review red: the focused command failed one test after the validator required CT-PIVOTS to cite the gene, variant, drug, and disease command owners. The row cited only the drug owner. Review also found that CT-BATCH cited the unrelated `Canonical Trial Age Bounds` heading.
- Code-review green: the same focused command passed all four tests after CT-PIVOTS cited all four command owners and a specification heading that executes all four command forms. CT-BATCH now cites a dedicated heading around the existing direct-versus-batch executable contract. The validator requires these exact references.
- Gates after remediation: `make lint`, `make test`, and `make spec` passed. `make test` ran 3,152 Rust tests with 3,152 passed and 30 skipped, then 902 Python tests with 899 passed and 3 skipped. Strict documentation passed. `make spec` passed its routine and static lanes.
- Code review: accepted after remediation. The reviewer verified all four pivot owners and executable command forms, the dedicated batch heading, the fifteen-row bound, the surface-derived declarations, and the absence of runtime changes.
