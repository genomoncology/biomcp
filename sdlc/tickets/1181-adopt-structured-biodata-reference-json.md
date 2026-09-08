---
flow: build
priority: 10
deps: ["1180"]
waits-on: ["botassembly/biodata/0104"]
---
# Emit complete BioData trial references

## Goal

BioMCP emits every valid shared clinical-trial reference without requiring a citation or reducing its source type to one provider's scalar code.

## Current evidence

BioMCP stores `ClinicalTrialReference` directly. Its duplicate wire adapter still requires a citation, trims source strings, fixes the source authority to ClinicalTrials.gov, and rejects source display, vocabulary version, recognized meaning, and other authorities. The adapter deserializes reference members into a local struct before it constructs the shared value. That parse can erase duplicate members before the shared codec validates them.

The legacy ClinicalTrials.gov model still declares `references_module`, `CtGovReference`, and `CtGovReferencesModule`. The detail path clears `references_module` explicitly before it runs the legacy transform. One capture-receipt mutation test protects the stale `CtGovReference.reference_type` rename. Record 1170 introduced this temporary second reference path and required its deletion after arms and eligibility. BioData ticket 0104 supplies the strict standalone value codec that makes the deletion possible.

## Required behavior

Update the exact BioData Git revision after ticket 0104 lands. Use `ClinicalTrialReference::to_json` and `ClinicalTrialReference::from_json_bytes` to encode and decode every populated reference value. Do not copy the three-member or nested five-member wire schema into BioMCP.

Keep the outer references section behavior. Missing or null input becomes `None` and stays omitted. An explicit empty array remains present and emits an empty array. Populated arrays preserve order and Unicode.

On input, deserialize each populated array entry as its original `Box<RawValue>`-style JSON value and pass those untouched bytes to BioData. Do not deserialize an entry into `serde_json::Value` or a BioMCP reference struct first. This boundary must preserve duplicate members for BioData's strict decoder. Tests must prove rejection when a reference object repeats a root member and when its nested `source_type` object repeats a member.

Replace the scalar `reference_type` member with the complete nullable `source_type` object from BioData. Every emitted reference has exactly the `pmid`, `citation`, and `source_type` keys, including null values. A populated `source_type` has exactly `authority`, `code`, `display`, `vocabulary_version`, and `recognized_meaning`, including null optional metadata. Accept PMID-only, citation-only, source-type-only, all-null, and fully populated shared values. Reject any entry with the retired `reference_type` key instead of maintaining two conversion policies.

Use the fixed error text `invalid clinical trial reference` for every reference adapter encode or decode failure. Do not include the rejected value, raw bytes, BioData error code, BioData path, or source metadata in that error.

BioMCP owns Markdown. Trim only for display. Show a citation and PMID when available. Show a PMID by itself when no citation exists. Choose a source-type label from display, recognized meaning, then code. Show a fixed useful fallback when no member has displayable text.

Delete the fixed authority, citation requirement, source normalization, scalar wire type, shared-metadata rejection, and citation filter. Delete `CtGovProtocolSection.references_module`, `CtGovReference`, and `CtGovReferencesModule`. Delete the explicit `references_module` clearing in the trial detail path. Delete the capture-receipt mutation case that protects the stale `CtGovReference.reference_type` rename. Keep the BioData response as the only ClinicalTrials.gov reference owner. Do not claim new provider support from synthetic shared values.

## Done, observably

- Exact JSON assertions prove that an all-null reference emits `{"pmid":null,"citation":null,"source_type":null}` and a complete reference emits only `pmid`, `citation`, and `source_type`. Its `source_type` emits all five named members. A second exact assertion covers a populated source type whose optional metadata members are null.
- JSON preserves complete source authority, code, display, vocabulary version, recognized meaning, PMID, citation, order, and Unicode without trimming stored values.
- Missing, null, empty, PMID-only, citation-only, source-type-only, all-null, and complete reference states behave as specified.
- Duplicate root members, duplicate nested `source_type` members, and retired scalar `reference_type` input fail with only the fixed data-free error.
- Markdown covers every fallback without blank entries.
- The real CLI JSON result, raw MCP result, and typed MCP result agree on the exact reference array for the same complete synthetic shared value. Their assertions cover all three reference keys and all five populated `source_type` keys. Existing recorded provider tests continue to prove provider content and order.
- No legacy `references_module`, `CtGovReference`, `CtGovReferencesModule`, explicit detail-path clearing, or stale capture mutation case remains.
- Recorded provider tests retain their existing evidence claims.
- CLI JSON, MCP, Markdown, exact dependency, lockfile, and full-feature gates pass.

## Boundaries

No provider adapter widening, new source claim, eligibility change, search change, release, or publication belongs here.
