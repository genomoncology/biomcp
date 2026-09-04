---
flow: build
priority: 7
---

# Return GenCC gene-disease validity assertions

## Goal

A gene request can retrieve public GenCC gene-disease validity assertions when ClinGen has no matching curation. On 2026-09-04, BioMCP returned an empty ClinGen section for ODC1. GenCC publishes a Strong ODC1–Bachmann-Bupp syndrome assertion with autosomal dominant inheritance, source publications, dates, and submitter identity. GenCC supplies a public bulk download with these fields and conditional-download metadata. The service evidence and source constraints appear in `sdlc/issues/feature-gencc-can-fill-public-gene-disease-validity-gaps.md` at commit `84f2343f`.

## Desired functionality

`biomcp get gene <symbol> gencc` exposes GenCC as a named source for gene-disease validity. The existing `clingen` section remains source-specific. The GenCC result returns each submission as a separate assertion and preserves the gene, disease identifier and label, classification, inheritance, submitter, dates, source links, publications, and available criteria. Source status distinguishes unavailable acquisition from a successful lookup with no matching assertion.

BioMCP refreshes the weekly bulk file only when its ETag or Last-Modified value changes. A failed refresh can use the last validated cache and identifies it as stale. GenCC limits this file to 20 downloads per day, so ordinary gene requests never download it unconditionally.

## Success criteria

- An ODC1 request returns the public assertion for Bachmann-Bupp syndrome with its Strong classification and autosomal dominant inheritance.
- Human-readable, JSON, and MCP output identify GenCC, the assertion submitter, and the source record.
- Two submitters with different classifications remain two assertions.
- A successful lookup with no matching gene differs from an unavailable or stale source.
- Cached bulk data records its retrieval time, ETag or Last-Modified value, and upstream version when supplied.
- Repeated gene requests reuse one validated file until conditional refresh reports a change.
- A refresh failure can return the last validated file with explicit stale status.
- Fixed fixtures prove parsing and lookup without a live request.

## Boundaries

This ticket adds GenCC gene-disease validity evidence. It does not merge GenCC and ClinGen into a consensus, treat the strongest submission as truth, redistribute restricted OMIM content, or make a patient-level diagnosis.
