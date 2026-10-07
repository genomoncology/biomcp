# Adopt shared Cancer Hotspots recurrence in both callers

Status: CODE candidate qualified. Root fresh review, dedicated 1.0 landing and paired closure pending.

The consumer merged dedicated target `582bbd492e64d0227517dcaef828425ab84b2e10` at `692bf15f`. Root reports that target incorporates maintenance `1c774def`. The consumer pins landed BioData0714 `edd138da0ba104039885fe213107003dfb626246`, version `0.0.41`, in Cargo and the existing boundary constants. HTTPS authentication failed. Fetching the exact landed producer Git object into Cargo's cache enabled locked offline preparation. No dependency path or floating revision was introduced.

Both detail and structure callers consume `CancerHotspotsResponse` from untouched HTTP bytes and retain `CancerHotspotRecurrenceProjection` directly. `CancerHotspotSection` holds only required source metadata and one private shared recurrence. Borrowed flattened serialization and shared target restoration preserve the existing JSON shape. Structure Markdown and both classifiers borrow shared getters. The opaque source position stays inside BioData and receives no application interpretation. Normalization, request routing, transport, caches, limits and timeout policies keep their existing implementations.

The new real-client privacy test failed on the old decoder because Debug exposed a malformed selected field value. Shared admission now maps to the static application message `Invalid Cancer Hotspots response.` without retaining a cause. Existing HTTP and content-type diagnostics retain their separate policy. Caller failure outputs retain their existing public messages and omit provider credit.

The first locked offline binary build passed in 29.31 wall seconds. The caller and retained source/renderer batch passed 17 tests in 12.66 seconds after 45.23 seconds of compilation; the full command took 59.42 seconds. Native detail and structure, CLI JSON/Markdown, typed/raw get MCP and raw structure MCP passed. The data/empty/failure tables preserve companion enrichment and structure/domain context, literal counts and transcript, section outcomes, source credit and exact route/count. Detail default and inapplicable requests make no Hotspots call. The held detail source exercises the existing eight-second timeout. Actual product restoration preserves required source, explicit nulls, omitted sections and literal whitespace transcript.

After those owners passed, the source parsing file retired seven displaced mapping tests. Three get helper tests and the structure helper matrix retired. The mixed-provider preflight test retains its other provider claims and removes its duplicated Hotspots case. The local row, recurrence fields, selection helpers and get test-only result helper retired. Existing HTTP, route and renderer owners remain.

## Qualification

The runtime CODE candidate is `542554c23c5d66077ddee6edabb5cadf77987c49`. Registration-only follow-up `e74a55d3cffed9c0000543b8a9fb537fa14b115c` adds the 14 affected Rust selections to the existing focused manifest and changes its expected count from 631 to 645. The final record changes no runtime or test behavior. All candidate commits were pushed to `ticket/2026-adopt-cancerhotspots-recurrence` with `[skip ci]`. Root owns fresh review and dedicated target landing.

Final test compilation used `cargo test --locked --offline --no-default-features --lib cancerhotspots --no-run`. It passed in 51.60 wall seconds. The resulting test executable ran the `cancerhotspots` filter and the retained structure renderer and three body-limit owners in one invocation with the current binary supplied by `BIOMCP_BIN`. All 14 tests passed in 13.29 test seconds and 14.80 command wall seconds. The final executable discovers 3972 library tests. Five new behavior tests replace eleven old tests. The remaining mixed-provider test removes only its Hotspots case.

Python used `CARGO_NET_OFFLINE=true`, the worktree routine test temporary directory and `uv run --offline --no-project --python 3.12 --with pytest python -m pytest tests/test_biodata_boundary.py tests/test_source_package_boundary.py tests/test_biodata_branch_workflow.py -k 'not forged_offline_marker_fails_in_the_normal_namespace'`. It passed 147 checks with one deliberate Darwin deselection in 31.06 test seconds and 31.39 command wall seconds. This includes the real Cargo source-package boundary. Extracted-package compilation retains the accepted development-version deferral.

The exact-pin boundary, changed-file rustfmt, Ruff 0.13.2 on the three changed pin/package Python files, tracked-text scan and whitespace checks pass. The existing source-size audit passes for 860 Rust files with no findings. The get baseline decreases from 1447 to 1434 lines. Its test baseline decreases from 1334 to 1273 lines. Existing authorization deltas decrease with those baselines; no ceiling increases.

The host was M5 `imaurer-m5`. Initial one-minute load was 1.81; final observed load was 2.94. Dependency compilation reused the shared compiler cache. The new worktree had no initial target artifacts. Later builds reused its dependency artifacts. These are ordinary build and test costs under shared host load. They establish no comparative performance result.

## Inherited limits

Record2025 already records inherited strict Clippy findings, whole-repository formatting differences, the workflow Ruff finding and Darwin network-isolation limitations. These whole gates were not re-audited. Compilation retains the existing unused-code warnings. Cached pytest retains its unknown `asyncio_mode` option warning. The known Linux `/proc` verifier test remains deselected on Darwin. Offline Cargo and local fixtures supplied this qualification; it does not claim Linux namespace isolation, live-provider acceptance or release qualification.

The old decoder's privacy regression failed before the implementation after 58.74 seconds of compilation and 0.08 test seconds. Two intermediate caller assertions failed because the authored fixture initially supplied no companion frequency data and no coding field for the alternate transcript. Correcting those inputs made the preserved behavior observable. The first full affected command took 56.61 wall seconds and passed 15 of 17 tests. The corrected command passed all 17 before the seven source-mapping owners retired. Those attempts supply no additional final test credit. An initial Python invocation used the system interpreter without `tomllib`; the selected Python 3.12 and recorded Ruff version passed. HTTPS Git fetch/push lacked authentication. Exact local Git-object preparation and authenticated SSH pushes succeeded without changing dependency declarations.

## Retired code and claims

The application retires `CancerHotspotRow`, `CancerHotspotRecurrence`, `checked_absent`, the local `recurrence_for_change`, `same_aa_count`, `normalize_residue`, `residue_and_alt` and the get-only test helper `apply_cancerhotspots_result`. Structure retains its outcome classifier because that classifier owns local contact/applicability behavior. No application access decodes the producer's opaque source position.

The seven retired source parsing tests are `parses_by_gene_fixture`, `receipted_recurrence_preserves_braf_and_myd88_landmarks`, `receipted_empty_response_is_checked_absence`, `recurrence_maps_counts_and_transcript_for_exact_alt`, `recurrence_serializes_checked_absence_with_nulls`, `recurrence_treats_missing_exact_alt_as_checked_absence` and `recurrence_checks_later_matching_residue_rows_for_exact_alt`. BioData0714's public API owns those pure semantics. The real consumer owners prove delegation and public output with the existing receipted BRAF bytes.

The retired get tests are `cancerhotspots_enrichment_uses_requested_change_not_resolved_hgvsp`, `cancerhotspots_checked_absence_is_empty_not_data` and `cancerhotspots_upstream_failure_omits_recurrence_and_preserves_cbioportal`. The retired structure test is `cancerhotspots_result_matrix_classifies_contact_and_applicability`. Real get/structure calls replace each claim. The get fixture carries requested V600E alongside resolved p.Val601Glu and preserves actual companion frequency data. The structure fixture retains an overlapping domain and PDB entry under failure. Existing independent Markdown assertions remain in their original renderer tests.

Private structural/resource decoder failures now contain only a static application message. Display, Debug and error JSON do not contain the private selected-field marker. Native, CLI stdout/stderr and typed/raw MCP text preserve safe caller failure behavior. HTTP failures and content-type rejection retain the existing transport diagnostic policy; this change does not broaden that policy.

## Exact file claims

- `Cargo.lock`
- `Cargo.toml`
- `sdlc/records/2026-adopt-cancerhotspots-recurrence.md`
- `sdlc/tickets/2026-adopt-cancerhotspots-recurrence.md`
- `src/entities/variant/get.rs`
- `src/entities/variant/get/cancerhotspots_transport_tests.rs`
- `src/entities/variant/get/tests.rs`
- `src/entities/variant/mod.rs`
- `src/entities/variant/structure.rs`
- `src/entities/variant/structure/cancerhotspots_transport_tests.rs`
- `src/render/markdown/variant.rs`
- `src/sources/cancerhotspots.rs`
- `src/sources/cancerhotspots/tests/parsing.rs`
- `tests/test_biodata_boundary.py`
- `tests/test_biodata_branch_workflow.py`
- `tests/test_source_package_boundary.py`
- `tools/biodata-1.0-focused.toml`
- `tools/check-biodata-boundary.py`
- `tools/rust-source-size-inventory.json`

The design, fixture bytes, capture receipts, templates, shared transport, activation parsers and maintenance 0.9 source remain outside the implementation diff. Root owns review, landing and paired closure. Cleanup belongs to the separate owner. Code, build output and logs with prefix `biomcp-2026-` remain for that owner. No scratch cleanup ran.

## Landing

Fresh read-only CODE review accepted d97fd21b with no concrete findings. Merge e87bb5981665c3d077e576058b9cf079aa7e3af0 lands both callers on the dedicated 1.0 line. The merge contains no runtime changes beyond the reviewed candidate. Fourteen affected native cases and 147 Python cases passed; inherited whole-gate findings remain as recorded. BioData producer edd138da0ba104039885fe213107003dfb626246 is the paired dependency. Both actual callers now store and render the shared recurrence; eleven displaced tests and local mapping code are retired.
