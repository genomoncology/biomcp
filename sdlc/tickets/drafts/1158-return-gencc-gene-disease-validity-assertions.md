---
flow: build
priority: 7
---

# Return GenCC gene-disease validity assertions

## Goal

A gene request can retrieve public GenCC gene-disease validity assertions when ClinGen has no matching curation. On 2026-09-04, BioMCP returned an empty ClinGen section for ODC1. GenCC publishes a Strong ODC1–Bachmann-Bupp syndrome assertion with autosomal dominant inheritance, source publications, dates, and submitter identity. GenCC supplies a public bulk download with these fields and conditional-download metadata. The service evidence and source constraints appear in `sdlc/issues/feature-gencc-can-fill-public-gene-disease-validity-gaps.md`.

## Desired functionality

BioMCP exposes GenCC as a named source for gene-disease validity. It returns each submission as a separate assertion and preserves the gene, disease identifier and label, classification, inheritance, submitter, dates, source links, publications, and available criteria. Source status distinguishes unavailable acquisition from a successful lookup with no matching assertion.

## Success criteria

- An ODC1 request returns the public assertion for Bachmann-Bupp syndrome with its Strong classification and autosomal dominant inheritance.
- Human-readable, JSON, and MCP output identify GenCC, the assertion submitter, and the source record.
- Two submitters with different classifications remain two assertions.
- A successful lookup with no matching gene differs from an unavailable or stale source.
- Cached bulk data records its retrieval time and upstream version or validator.
- Fixed fixtures prove parsing and lookup without a live request.

## Boundaries

This ticket adds GenCC gene-disease validity evidence. It does not merge GenCC and ClinGen into a consensus, treat the strongest submission as truth, redistribute restricted OMIM content, or make a patient-level diagnosis.
