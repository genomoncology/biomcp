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

BioMCP stores `ClinicalTrialReference` directly. Its duplicate wire adapter still requires a citation, trims source strings, fixes the source authority to ClinicalTrials.gov, and rejects source display, vocabulary version, recognized meaning, and other authorities. BioData ticket 0104 supplies the missing strict value codec.

## Required behavior

Update the exact BioData Git revision after ticket 0104 lands. Use BioData to encode and decode each populated reference value.

Keep the outer references section behavior. Missing or null input becomes `None` and stays omitted. An explicit empty array remains present and emits an empty array. Populated arrays preserve order and Unicode.

Replace the scalar `reference_type` member with the complete nullable `source_type` object from BioData. Accept PMID-only, citation-only, source-type-only, all-null, and fully populated shared values. Reject the retired scalar input instead of maintaining two conversion policies.

BioMCP owns Markdown. Trim only for display. Show a citation and PMID when available. Show a PMID by itself when no citation exists. Choose a source-type label from display, recognized meaning, then code. Show a fixed useful fallback when no member has displayable text.

Delete the fixed authority, citation requirement, source normalization, scalar wire type, shared-metadata rejection, and citation filter. Do not claim new provider support from synthetic shared values.

## Done, observably

- JSON preserves complete source authority, code, display, vocabulary version, recognized meaning, PMID, citation, order, and Unicode.
- Missing, null, empty, PMID-only, citation-only, source-type-only, all-null, and complete reference states behave as specified.
- Retired scalar `reference_type` input fails safely.
- Markdown covers every fallback without blank entries.
- Recorded provider tests retain their existing evidence claims.
- CLI JSON, MCP, Markdown, exact dependency, lockfile, and full-feature gates pass.

## Boundaries

No provider adapter widening, new source claim, eligibility change, search change, release, or publication belongs here.
