# Adopt the shared InterPro domain response

October 7, 2026. Consumer2027, milestone1.0 componentbackend, pairs with producer0715. DESIGN only. Base dedicated1.0 `c8e3c190`; producer `42c0a7eba67870185dbeca25d0209597ffdd3c20`.

## Selected complete result

Move the remaining selected InterPro response from `src/sources/interpro.rs` into the pure library and consume it through both real callers. `entities::protein::get` currently calls `InterProClient::domains(&protein.accession,20)` and maps accession/name/domain_type into `ProteinDomain`. `entities::variant::structure::structure` calls `domains(&accession,25)` only with a selected residue, then `overlapping_domains` chooses the first inclusive matching range per entry. Both callers must move and displaced source code must retire before BUILD completes.

The producer owns original-byte admission and context-free source shaping. The application owns the request, cap policy, residue overlap, final object fields, source labels, rendering and execution outcomes. This delivers a complete selected response used by two existing surfaces. It does not transfer a universal Variant or Protein entity.

## Evidence and queue

Filtered `pm next --milestone 1.0 --component backend` has no ready or waiting duplicate. Existing InterPro matches in tickets concern inherited0449/0452 and failure-state work0577/0587. None is an active paired producer transfer. Producer queue/current records have no InterPro replacement. Global `pm ticket new` allocated2027 without hand numbering. The allocator used temporary metadata over existing objects and current remote refs because the local maintenance main was stale. It did not update shared main or copy source files.

Reuse `architecture/experiments/variant-structure-annotation/explore.md` as the recorded experiment conclusion. Its cited direct_source_join result files are absent from the current checkout; this design does not claim inspection of raw experimental results or require their recovery. The promoted experiment established the useful residue/domain join and transcript-position warning. Preserve that warning and the existing selected residue. It proves historical reachability rather than current provider availability or clinical validity. Current `src/sources/interpro/tests/parsing.rs` protects the authored457..717 row; its source parsing claim moves to producer0715. Hotspots0714/2026 is complete and remains outside the pair. No fresh experiment is needed.

## Shared source and client boundary

Producer0715 supplies `InterProResponse::decode_json(&[u8])`, ordered source entries and borrowing source getters, plus `domains(first_rows)` yielding borrowing domain views. Source storage retains selected metadata and nested fragment values, including incomplete pairs. Views skip missing/blank accessions, trim optional labels and expose complete ranges in original order. `domains` takes the exact caller count before filtering. The application clamps `limit` to1..25 before requesting and viewing a page. Producer0715 defines the full admission table,8MiB/depth bounds and sanitized static errors.

Replace the InterPro-only generic `get_json<T>` with retrieval of bounded original bytes and direct producer decoding. Keep `domains_plan`, accession trimming/blank rejection, endpoint, BIOMCP_INTERPRO_BASE, shared client, cache mode, HTTP handling and SourceContext retry/narrow classifications. `InterProClient::domains` returns the admitted shared response; its request keeps the existing limit parameter. No serialize/reparse bridge or local mirrored source graph survives.

A malformed successful response maps to an application API error with static text and InterPro retry source context. Do not expose serde errors or source values. Existing non-success HTTP body_excerpt behavior remains transport-owned and unchanged; this pair does not claim to sanitize that unrelated established policy. A decoding failure must reach unavailable, never a normal empty result. The transport body limit and decoder limit both remain8MiB.

## Retained public outputs and policies

Protein `get` maps `response.domains(20)` into existing `ProteinDomain`. JSON retains accession plus optional `name` and `domain_type`, omitted when absent. Ranges remain absent from this output. Metadata-less and blank-accession rows consume their source position before being skipped. Request cap20 stays independent of accepted-row count. Healthy empty means empty with InterPro credited; transport/admission failure means unavailable and no successful source credit. Unrequested domains keep the existing unrequested behavior.

Structure maps `response.domains(25)` with its existing selected residue. For each entry, choose only the first range whose endpoints inclusively contain the residue. Keep entry order and repeated accessions; never sort ranges, combine overlaps, deduplicate entries or reject zero/reversed pairs during admission. Final `VariantStructureDomain` retains accession, optional name, optional `type`, start, end and required `source: InterPro`. Caller-owned output models remain local because their shapes differ and include application attribution.

No selected residue means no InterPro request and an inapplicable domains lookup. Selected residue plus no overlaps means empty; a matching range means data; provider/admission failure means unavailable. Preserve successful protein/structure fields from other sources, matched HGVSp, additional-position warnings, next commands, Markdown and JSON/MCP output. Default get variant remains unchanged. Existing `apply_domains_result` functions retain their separate policy ownership.

## Outside-in BUILD cases

1. Producer0715 owns source admission, raw retention, trimmed views, cap-before-skip and complete-fragment order. It uses authored synthetic bytes and one public edge table. The application does not replay that decoder table.
2. New `src/entities/protein/interpro_adoption_tests.rs` runs actual `protein::get` against the existing local provider fixture support. Feed literal UniProt and authored InterPro bytes. Assert the request path/page_size20 and full final domain-vector equality, including optional label omission and duplicate entry order. Include a blank first source entry and a distinguishable21st entry to detect moving the cap after filtering. This owns real client-to-protein propagation.
3. New `src/entities/variant/structure/interpro_adoption_tests.rs` runs actual `structure` with existing fixture support and a literal resolved variant/UniProt/InterPro response. At selected residue600, two overlapping fragments457..717 and590..610 must yield only457..717 for the entry. Include a separate600..600 entry to detect exclusive endpoints and a nonoverlapping entry to detect failure to filter. Assert exact ordered final domain objects, domains data state, source and retained residue warning. This owns caller overlap, not producer parsing.
4. In those caller owners, test one HTTP failure through protein get and one malformed successful response through structure. Assert domains unavailable while other fetched data survives. Exercise no-selected-residue through the existing callable structure fixture with no InterPro request and inapplicable outcome. These distinct cases guard transport, admission mapping and eligibility. Reuse existing `domain_result_matrix_classifies_contact_and_applicability`, protein `interpro_and_string_failure_state_matrix` and Markdown unavailable/retry assertions; do not copy their matrices into new files.

Use literal expectations and synthetic JSON. Existing dbNSFP/Hotspots adoption fixtures remain in place; do not copy their tests or source captures. Keep test infrastructure thin and use existing provider-base overrides. Register new test files and required module declarations according to the repository's existing ratchets. Any temporary inner test scaffolding is removed before landing. No benchmark suite or proof framework is required.

## Code and test retirement

Remove local `InterProResponse`, `InterProResult`, `InterProMetadata`, `InterProProtein`, `InterProLocation`, `InterProFragment`, `InterProDomain`, `InterProRange`, `decode_domains_response` and `interpro_ranges`. Replace the local generic JSON admission with the original-byte shared decoder. Update both callers to shared borrowing views and keep only their final output objects and caller-owned overlap helper. No compatibility alias or duplicate source serializer remains.

Retire `src/sources/interpro/tests/parsing.rs` and its module registration only after its source shaping claim passes in producer0715 and its propagation claim passes through both callers. Keep `src/sources/interpro/tests/construction.rs` because request policy stays local. Keep the existing protein/structure outcome and rendering tests because they own different behavior. Later BUILD updates only necessary file-cap registrations and records the removed claims once. DESIGN deletes nothing.

## Source, policy and privacy

The current source registry and September27 licensing review qualify InterPro reuse under EMBL-EBI terms and embedded member-database obligations. This pair reuses that recorded qualification and imports no external fixture. MIT library code does not license upstream data. Future capture or redistribution needs separate admission. No source-data, production-data, credential, licensing, health catalog or source registry change is claimed.

Concrete risks are cap-after-filter drift, reordered first overlap, false absence after failure and source-value diagnostics. The proposed ownership and distinct outside-in cases address them. Residue selection and isoform uncertainty retain current behavior; this transfer makes no new mapping claim. No policy exception or new parser activation work is needed.

## Scope, checks and release

DESIGN claims only2027 and this note. Push its WIP with `[skip ci]` for Root's fresh paired review. Root releases BUILD after producer0715 is reviewed and provides an accepted exact Git pin. Consumer BUILD claims InterPro source/client, the two caller sections, the named new tests and necessary module/cap registrations, Cargo pin/lock and one owning record. It does not claim shared programme/lane documents, activation files, source fixtures, other enrichments or maintenance main.

Root reconciles current maintenance on the dedicated1.0 line before implementation/review. M5 builds and focused affected checks are authorized for later BUILD. Record the exact producer/consumer pair and ordinary focused costs once. Broad release checks belong to Root's candidate rehearsal. DESIGN runs no product code, tests, gates, live/hosted jobs, production reads, Pi jobs, nested delegates, source copies or deletion. Another agent owns worktree and temporary allocation cleanup.
