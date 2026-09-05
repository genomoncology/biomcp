---
flow: build
priority: 9
---

# Return current direct ClinVar assertions

## Goal

An explicit ClinVar variant request returns the current NCBI record and shows each submitted assertion separately. On 2026-09-04, `biomcp --json get variant rs1753477498 clinvar` reported one submitter and a one-star review status from MyVariant. NCBI EFetch reported two submitters and a multiple-submitter aggregate for ClinVar Variation ID 974782. The reproduction and source analysis appear in `sdlc/issues/clinvar-section-can-report-stale-review-strength-without-a-date-or-warning.md` at commit `84f2343f`.

## Desired functionality

`biomcp get variant <id> clinvar` retrieves the already-resolved numeric Variation ID from NCBI ClinVar EFetch. The same direct retrieval runs when `all` is requested, but never for the default variant card. The returned `VariationArchive@VariationID` must match that requested ID exactly before any direct fact is accepted.

The direct payload has two typed layers rather than one singular "classification": condition-specific RCV aggregate rows and individual SCV clinical-assertion rows. Both preserve their accession and version, classification domain (germline, somatic clinical impact, oncogenicity, or another provider value), review status, evaluation date, and condition associations. The direct record preserves `VariationArchive` record status once at the VCV level. SCV rows preserve their own `ClinicalAssertion` record status and additionally preserve submitter, the `ContributesToAggregateClassification` state, criteria, citations, and public comment when supplied. An RCV row carries record status only when the RCV element/schema explicitly supplies it; the parent VCV status is never copied into an RCV field as if it were RCV evidence. Trait mappings participate in condition association. Every current SCV in an accepted record is returned, including current non-contributing assertions; non-current assertions are not silently mixed into the current set. Classification domains are never collapsed into one consensus.

Direct XML is an untrusted bounded input. The request uses the repository's bounded-response transport, validates successful XML content, rejects entity declarations through `parse_external_xml`, and accepts at most 8 MiB, 1,000,000 XML nodes, 512 RCV aggregates, 2,048 SCVs, 256 condition associations per row, 128 citations per SCV, and 32 KiB each for criteria and public-comment text. If any record-wide or per-row cap is exceeded, the direct record is rejected as unavailable rather than silently truncated, so "every current SCV" means every current assertion inside a fully accepted bounded record.

If direct ClinVar retrieval is unavailable, malformed, oversized, non-XML, or identity-mismatched, BioMCP may retain a usable indirect MyVariant summary. Its typed indirect rows preserve `clinvar.rcv.accession`, version when supplied, `clinvar.rcv.last_evaluated`, and `clinvar.rcv.number_submitters`. The result identifies MyVariant.info as the carrier and does not present several submissions as one unqualified assertion.

`clinvar` becomes a canonical section outcome shared by JSON, Markdown, raw MCP, and typed MCP. Direct success is `data` credited to `NCBI ClinVar`; a direct failure with usable MyVariant data is `degraded` credited to `MyVariant.info` with a bounded public message; a valid direct response with no usable record is `empty`; and no usable direct or indirect result is `unavailable`. If resolution yields no numeric ClinVar Variation ID, BioMCP makes no direct request and completes `clinvar` as `inapplicable`, with no credited source and a bounded message explaining that direct retrieval requires a resolved Variation ID. `_meta.section_sources`, rendered provenance, payload typing, and the section outcome must agree. Provider/parser internals are not exposed in public errors.

## Success criteria

- The fixed HSD17B4 example reports the current two-submitter RCV aggregate and both current SCV submissions without treating the stale indirect one-submitter summary as direct NCBI evidence.
- Every returned aggregate and submission identifies its source, accession, version when supplied, classification domain, condition associations, and available evaluation date. The VCV-level record status and each SCV's own record status remain distinct; an RCV status is present only when explicitly supplied. SCVs also preserve submitter and contribution state.
- A fixture containing disagreeing germline SCVs, a non-contributing current SCV, a distinct somatic classification, and trait mappings proves all current rows and their conditions remain separate and no classification domains collapse.
- Explicit `clinvar` and `all` each make one direct request after variant resolution selects one numeric Variation ID. The default card makes none. A mismatched returned Variation ID is never accepted.
- A resolved variant without a numeric ClinVar Variation ID makes zero direct requests and renders the source-free canonical `inapplicable` outcome consistently through Markdown, JSON, raw MCP, and typed MCP.
- Deterministic transport/parser fixtures cover a non-success response, HTML content, malformed XML, an entity declaration, mismatched identity, the exact body/node/list/text boundaries, and limit-plus-one. Each failure degrades to usable indirect data when present without leaking parser or transport details; no partial direct record escapes a cap.
- The indirect fixture proves exact projection and round-trip of `clinvar.rcv.accession`, version when supplied, `clinvar.rcv.last_evaluated`, and `clinvar.rcv.number_submitters`.
- Direct `data`, direct `empty`, indirect `degraded`, and wholly `unavailable` fixtures make payload source, canonical `clinvar` outcome, `_meta.section_sources`, Markdown, JSON, raw MCP, and typed MCP agree.
- `docs/sources/clinvar.md`, `docs/reference/source-licensing.md`, and their executable contracts stop claiming ClinVar is indirect-only and describe the direct/fallback boundary truthfully.

## Boundaries

This ticket changes the explicit ClinVar section and `all`, plus the typed status/provenance and documentation needed to describe them. It does not require direct ClinVar retrieval on the default variant card, return a partially truncated direct record, calculate a consensus classification, merge classification domains, apply ACMG criteria, or make a clinical conclusion. It does not duplicate assertion rows beyond the bounded accepted record or expose provider-controlled unbounded prose.

## Result

The explicit `clinvar` section and `all` now retrieve one current Variation Archive by the resolved numeric Variation ID through NCBI EFetch, while the default card performs no direct ClinVar request. Typed VCV, condition-specific RCV, and current SCV values preserve their separate identity, status, domain, contribution, condition, freshness, citation, criteria, and public-comment semantics. Invalid, hostile, mismatched, or over-budget XML fails closed without returning a partial direct record. MyVariant.info remains a typed degraded fallback with accession, version, evaluation date, and submitter count.

The canonical `clinvar` outcome now drives JSON, Markdown, and MCP provenance for direct data, checked empty, degraded fallback, unavailable, and source-free inapplicable states. The ClinVar source page, licensing inventory, source registry, configuration override classification, and executable documentation contracts describe the direct/fallback boundary. Cargo files remain unchanged, BioData remains pinned to `4f912d35a0f3fbff6994f1769d7601d7d0367aa1`, and `cargo package --list --allow-dirty --locked --offline` reports exactly 1,300 files.

Focused parser, fallback, rendering, outcome/provenance, source-documentation, licensing, configuration, package-boundary, and quality-ratchet tests passed. The final routine rerun passed `make lint`; `make test` passed 3,166 Rust tests with 30 skipped, 901 Python tests with three skipped, and the strict documentation build; `make spec` passed every routine and static group. The first two routine attempts exposed and led to removal of an orphan health probe and documentation of the new fixture override seam before the clean final rerun.

The first independent implementation review rejected the change because duplicate assertion IDs could amplify trait-mapping allocation, supplied malformed numeric and contribution attributes did not all fail closed, the optional NCBI key was inconsistent in the source inventory, and deterministic override coverage did not yet span the public request modes and MCP surfaces. Remediation rejects duplicate supplied assertion IDs before building the mapping index, strictly parses supplied VCV/RCV/SCV numeric and contribution attributes, extends exact/over criteria-boundary coverage, and exercises the ClinVar override through default, explicit, `all`, fallback, inapplicable, JSON, raw MCP, and typed MCP paths. The source inventory and user-facing key references now consistently identify `NCBI_API_KEY` as optional ClinVar EFetch quota uplift.

Post-review focused evidence passed 27 ClinVar-filtered Rust tests, Clippy for the no-default-feature library and test graph with warnings denied, 51 source/licensing/configuration/documentation contracts, and 12 package-boundary and quality-ratchet contracts. The package list remains exactly 1,300 files and the BioData pin remains unchanged. Routine gates were not rerun after remediation because the coordinating agent reserved that gate lane.

## Review

- Design review: accepted in the materially tightened ticket before implementation.
- First implementation review: rejected with bounded-allocation, fail-closed parsing, documentation-inventory, and end-to-end override-coverage findings; all findings were remediated in the implementation and focused tests.
- Remediation review: pending independent code review; this record does not claim code-review acceptance.
