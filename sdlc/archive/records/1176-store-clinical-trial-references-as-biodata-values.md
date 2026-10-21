---
flow: build
priority: 1
---
# Store clinical-trial references as BioData values

## Outcome

BioMCP stores clinical-trial references directly as `biodata::ClinicalTrialReference` values. The private `TrialReference` wrapper and its duplicate domain API no longer exist. JSON and Markdown preserve the current reference capability, validation, ordering, nullability, and absent-versus-empty behavior.

## Current facts

BioMCP main at `b5a8273b006d89ddf36db4549494f3f974900aff` pins BioData `0.0.7` at `65f6af05720fdc0fbf630578be98ea34d77122d6`. `Trial::references` stores `Option<Vec<TrialReference>>`. The private wrapper in `src/entities/trial/mod.rs` owns one `ClinicalTrialReference`, duplicates construction and getter behavior, and implements 142 lines of hand-written Serde and tests. The accepted clinical-trial plan and the completed arm record name this wrapper as the next removal before eligibility work.

The public JSON shape contains optional `pmid`, required `citation`, and optional `reference_type`. Missing or null `references` means the section is absent. An empty array means the caller requested the section and the provider returned no usable references. Markdown preserves source order and prints the same three values. BioData deliberately does not implement Serde because its values do not define a product wire contract.

## Scope and decisions

Change `Trial::references` to `Option<Vec<biodata::ClinicalTrialReference>>`. Remove the product-owned `TrialReference` type, constructor, shared-value getter, normalization helper, and type-level Serde implementation. Do not change BioData or its pinned revision.

Keep product wire policy at the `Trial::references` field through one private Serde adapter. The adapter serializes the current three-member object shape and deserializes through `ClinicalTrialReference::new` plus `ExtensibleCode::new`. It keeps `references` absent for a missing or null field, preserves an explicit empty array, preserves source order and Unicode, trims optional strings as the current wrapper does, treats blank optional values as absent, and rejects a missing, null, empty, or blank citation.

Deserialization assigns `clinicaltrials.gov` authority to a present `reference_type`, matching the only current reference-producing wire contract. Serialization must not silently discard shared information. Reject a reference with no usable citation, a source-type authority other than `clinicaltrials.gov`, or nonempty source-type display, vocabulary-version, or recognized-meaning metadata. A future provider or richer wire contract must widen this adapter deliberately before those values can be emitted.

Make the Markdown renderer build a small borrowed presentation view from the shared values. The view exposes only `pmid`, required `citation`, and the source-type code to the existing template. It must not own or duplicate reference domain values. Rendering an invalid shared value returns the existing safe BioMCP error and never prints rejected source content.

Build ClinicalTrials.gov product references directly from the BioData reference section. Preserve the current behavior that skips a shared reference without a usable citation and returns an error for `NotRequested` or `Unavailable`. NCI continues to return an explicit empty list when references are requested because no NCI reference projection exists yet.

Update direct test construction to use BioData constructors. Add focused red-green tests for direct shared storage, exact JSON round trips, missing/null/empty/populated section states, optional normalization, Unicode, source order, unsupported authority and metadata rejection, Markdown output, and safe invalid-data errors. Keep the existing real CLI and raw and typed MCP reference contracts green.

Do not change reference JSON, Markdown wording, provider requests, section selection, schemas, public documentation, BioData, eligibility, arms, or unrelated trial behavior. Do not add a generic wire framework. Delete more production Rust than this ticket adds. Keep the source package at or below 1,300 files.

## Acceptance

1. `Trial::references` stores `Option<Vec<biodata::ClinicalTrialReference>>` directly, and no product-owned `TrialReference` type remains.
2. Missing and null `references` deserialize to `None`; `[]` deserializes to `Some(Vec::new())`; populated arrays preserve source order and Unicode.
3. JSON preserves the current `pmid`, `citation`, and `reference_type` shape. Blank optional values normalize to absence. Missing, null, empty, and blank citations fail safely.
4. Serialization rejects absent citations, unsupported authorities, and source-type metadata that the current wire cannot carry. No shared information disappears silently.
5. Markdown uses a borrowed presentation view and preserves current output without reintroducing an owned domain wrapper.
6. ClinicalTrials.gov, NCI, real CLI, raw MCP, typed MCP, location flattening, and `Trial` serialization and deserialization retain their current reference behavior.
7. Net production Rust shrinks. The package remains at or below 1,300 files.
8. An independent design review and an independent code review accept the result. Focused red-green evidence and `make lint`, `make test`, and `make spec` pass.

## Dependencies

BioMCP record 1175 supplies the current direct BioData clinical-trial integration and names this cleanup as the next migration step. Both Factory channels remain paused. The manual subagent SDLC owns this work.

## Review

- Manual approval: Ian approved the clinical-trial delivery plan and directed implementation through the subagent SDLC without Factory.
- Design review: accepted. The independent reviewer confirmed that field-level Serde can preserve the current missing, null, empty, and populated states without giving BioData a product wire contract. Implementation must exercise those states through a complete `Trial`; cover whitespace-only shared values, every source-type metadata member, and non-ClinicalTrials.gov authority; validate before MiniJinja receives a view; remove or migrate the legacy reference extractor; and retain the real CLI, raw MCP, and typed MCP reference contracts.
- Code review: rejected once, then accepted. A fresh-binary review found that a whitespace-only optional ClinicalTrials.gov reference type caused the new serializer to reject the complete response. The remediation validates authority and unsupported metadata first, then normalizes a blank optional code to absence. Direct shared-value tests prove that a blank code cannot hide unsupported authority or metadata. The rebuilt real CLI passed all fifteen ClinicalTrials.gov reference contracts. Focused Rust reference, projection, and Markdown tests passed. Production Rust shrinks by sixty-two lines, the quality ratchet passes, and the package remains exactly 1,300 files.
- Final verification: `make lint`, `make test`, and `make spec` passed on the accepted revision. The test gate passed the grouped Rust suite, 910 Python contracts with three skips, strict documentation, and every offline specification group.
