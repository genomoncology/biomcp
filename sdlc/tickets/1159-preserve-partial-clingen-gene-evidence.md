---
flow: build
priority: 8
---

# Preserve partial ClinGen gene evidence

## Goal

A slow or failed ClinGen request does not erase evidence returned by another ClinGen dataset. On 2026-09-04, two no-cache TP53 requests returned no ClinGen facts after the combined section timed out. Direct ClinGen requests responded separately, and the implementation can discard completed validity or dosage data when its sibling request fails. The reproduction and code path appear in `sdlc/issues/2026-09-04-one-clingen-failure-erases-other-results.md` at commit `d9d29dd1`.

## Desired functionality

BioMCP retains completed ClinGen gene-validity and dosage-sensitivity results independently. Each result family reports whether it returned data, found no matching record, failed, or timed out. A cold acquisition does not convert partial success into an empty combined object.

## Success criteria

- A fixed TP53 fixture with a delayed dosage response still returns completed gene-validity records.
- A fixed TP53 fixture with a failed validity response still returns completed dosage records.
- Human-readable, JSON, and MCP output report source status separately for validity and dosage.
- An unavailable result cannot appear as a confirmed empty result.
- The output identifies which ClinGen operation failed or timed out.
- Tests prove partial success without live requests.

## Boundaries

This ticket changes failure isolation and result status for existing ClinGen gene evidence. It does not add GenCC, add clinical actionability, change ClinGen classifications, or create a consensus across sources.
