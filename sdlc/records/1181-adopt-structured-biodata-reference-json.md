---
flow: build
priority: 10
deps: ["1180"]
waits-on: ["botassembly/biodata/0104"]
---
# Adopt structured BioData reference JSON

## Outcome

BioMCP now delegates every populated clinical-trial reference entry to BioData's standalone codec. JSON emits the complete shared reference shape. The provider and detail paths no longer own a second ClinicalTrials.gov reference model.

## Implementation

BioMCP pins BioData `0.0.11` at `cfafc69d27c9a2fc74909f21692a418a8b17db83`. The field adapter keeps each input entry as `serde_json::value::RawValue` and passes its original bytes to `ClinicalTrialReference::from_json_bytes`. Serialization uses `ClinicalTrialReference::to_json`. Every adapter failure uses `invalid clinical trial reference` without source data.

Every reference emits exactly `pmid`, `citation`, and `source_type`. A present source type emits exactly `authority`, `code`, `display`, `vocabulary_version`, and `recognized_meaning`. Missing, null, empty, all-null, partial, and complete states preserve their specified behavior. Values preserve order, Unicode, and stored whitespace. Duplicate members and the retired `reference_type` member fail through the fixed safe error.

Markdown trims borrowed values only for display. It shows citation and PMID together, PMID alone, a source label chosen from display, recognized meaning, then code, or the fixed `Reference details unavailable.` fallback.

BioMCP deleted `CtGovProtocolSection.references_module`, `CtGovReference`, `CtGovReferencesModule`, explicit detail-path clearing, local authority and metadata policy, citation filtering, and the stale capture mutation. Mechanical boundary tests protect BioData codec ownership and the deleted provider model.

## Evidence

The initial focused adapter run produced four failures across five tests. The old adapter rejected valid shared states, returned its former error, and trimmed stored whitespace. The same five tests passed after the codec handoff. Focused Markdown, product section-state, CLI, raw MCP, typed MCP, provider-recording, ownership, and capture-receipt tests also passed.

Independent code review rejected the first implementation because a valid PMID-only reference also displayed its source-type label. A focused assertion reproduced that output. The remediation limits the source-type suffix to references with a citation. The focused Markdown test now proves that a reference with PMID and source type renders the PMID alone.

The complete implementation passed `make lint`, `make test`, `make spec`, and `make full-feature-check`. The Markdown-only review remediation then passed the focused reference and Markdown tests plus `make lint`, `make test`, and `make spec`. Routine tests passed 3,298 Rust tests with 30 skipped and 912 Python tests with 3 skipped. Strict documentation and every offline specification group passed. The source package remains at the 1,300-file ceiling.

## Boundary

This change does not widen provider extraction or source claims. It does not change eligibility, search, release, or publication behavior.
