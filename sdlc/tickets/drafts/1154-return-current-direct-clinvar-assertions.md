---
flow: build
priority: 9
---

# Return current direct ClinVar assertions

## Goal

An explicit ClinVar variant request returns the current NCBI record and shows each submitted assertion separately. On 2026-09-04, `biomcp --json get variant rs1753477498 clinvar` reported one submitter and a one-star review status from MyVariant. NCBI EFetch reported two submitters and a multiple-submitter aggregate for ClinVar Variation ID 974782. The reproduction and source analysis appear in `sdlc/issues/clinvar-section-can-report-stale-review-strength-without-a-date-or-warning.md`.

## Desired functionality

`biomcp get variant <id> clinvar` retrieves the resolved Variation ID from NCBI ClinVar. The result keeps the variant aggregate separate from each SCV submission. It identifies the direct NCBI source and preserves available accessions, versions, conditions, classifications, review status, submitters, evaluation dates, criteria, citations, and public comments.

If direct ClinVar retrieval is unavailable, BioMCP may return an indirect MyVariant summary. The response identifies MyVariant as the carrier, shows available freshness information, and marks coverage as partial. BioMCP never presents several submissions as one unqualified assertion.

## Success criteria

- The fixed HSD17B4 example reports the current two-submitter aggregate and both SCV submissions.
- Every returned aggregate and submission identifies its source, accession, version when supplied, and available evaluation date.
- Human-readable, JSON, and MCP output distinguish direct ClinVar facts from an indirect MyVariant summary.
- A fixture with disagreeing SCVs preserves both classifications and their conditions.
- A direct-provider failure can retain a dated indirect result with an explicit partial status.
- Existing variant resolution still selects one Variation ID before the direct request.

## Boundaries

This ticket changes the explicit ClinVar section. It does not require direct ClinVar retrieval on the default variant card, calculate a consensus classification, apply ACMG criteria, or make a clinical conclusion.
