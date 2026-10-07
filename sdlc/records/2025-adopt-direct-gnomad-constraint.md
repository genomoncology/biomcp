# Adopt direct gene constraint records

October 7, 2026. BioMCP ticket 2025, milestone 1.0, component backend. Consumer BUILD complete. Root owns fresh CODE review and landing. Another agent owns cleanup. The branch is `ticket/2025-adopt-direct-gnomad-constraint`.

Root released the Gibbs-accepted [design](../planning/direct-gnomad-constraint-2025/design.md) at `b6e6908134bd18c2de190936f82917fb9295b7ec`, over dedicated base `9747e3f05d74881235a5a8d1a072938758856901`. That base includes population2015 and maintenance37631c35. Final tested code is `f20c7ec21bb1c0f0bcef0474ef6827948afe46bc`. This record and the [ticket](../tickets/2025-adopt-direct-gnomad-constraint.md) add administrative evidence after that code revision.

Cargo and the existing boundary constants use the exact BioData Git revision `543bf8c304c3965af6a1c92076c9e63d02bd2998`, version `0.0.39`. Root supplied producer CODE ACCEPT `46e8c9a33d9ebece7dd63c45fb23634144521b38` and its Linux qualification, then landed and pushed the producer. Cargo's HTTPS authentication failed during preparation. Fetching that exact producer commit from the local producer repository into Cargo's Git cache enabled the locked offline build. The declared dependency remains the exact Git dependency.

## Delivered behavior

The existing HTTP decoder delegates the original nonnull gene member to `GnomadGeneConstraint::deserialize_gene`. Its optional wrapper carries the shared projection. `GnomadConstraintData` is a compatibility alias. Product GraphQL classification and transport remain local. The parser moves the admitted record directly.

`GeneConstraint` owns the shared immutable record plus the existing source, version and reference-genome strings. Flattened Serde encoding borrows `as_constraint_target`; flattened decoding delegates `deserialize_constraint_target`. The existing MiniJinja renderer receives that same flat envelope. Serial retrieval, ParallelTop retrieval, section availability, outcomes and timing borrow shared getters. Source transcript normalization and literal target transcript restoration retain their separate rules.

The actual caller leaf checks independent transcript and four metric literals, all three metadata fields, section outcomes and JSON provenance through native retrieval and shipped CLI/raw/typed MCP in JSON and Markdown. Native requests constraint plus Ontology to activate the existing ParallelTop predicate. Reports identify both strategies and record constraint timing; request logs confirm the source call. One existing local fixture serves MyGene, Open Targets, Enrichr, OLS and gnomAD. All cases share its origin because the process-wide limiter retains its initial fixture origin. Native calls use the existing no-cache scope. The table covers committed data/null/not-found inputs, healthy all-empty data, source trimming, ignored private input, GraphQL failures beside data and without messages, malformed metrics, HTTP failure and timeout. Default native and public JSON requests omit constraint and its metadata and make no gnomAD call. Flattened Gene decoding checks present/null/omitted fields, literal blank transcripts, source-name exclusion and required product metadata.

Retired the local `GeneConstraintGene`, `ConstraintPayload`, five-field `GnomadConstraintData`, source trim/copy mapper and five authoritative product value fields. Retired only `gene_constraint_maps_metrics_and_transcript` and `gene_constraint_returns_some_with_transcript_when_constraint_is_null` after actual caller proof passed. Producer0712 owns their admission/mapping claims. The existing GraphQL, request construction, HTTP, classifier and section-only renderer owners remain.

Fixture bytes and capture receipts are unchanged. The four constraint receipts remain `pending_verification`. The template, generic transport and population runtime are unchanged. Whole Gene ownership and provider qualification remain deferred.

## Verification and measured cost

Host: M5, Darwin arm64, 18 logical CPUs, Rust 1.95.0. The initial worktree target was cold; Cargo dependency caches were available after exact producer preparation. Final compilation reused artifacts. Observed one-minute host load ranged from 2.23 to 11.54 during work; final test preparation observed 4.55–4.88. These are directional host measurements.

| Check | Result | Wall seconds |
| --- | --- | ---: |
| Initial locked offline CLI build | Pass | 58.82 |
| Initial test compilation, including artifact-lock wait | Pass | 92.24 |
| Final locked offline test compilation | Pass | 68.26 |
| Final locked offline CLI build | Pass | 3.50 |
| Existing gnomAD source tests | 9 pass | 1.531 |
| Actual caller and flattened Gene owners | 2 pass | 3.360 |
| Retained constraint outcome owner | 1 pass | 0.011 |
| Retained section-only renderer owner | 1 pass | 0.019 |
| Focused Python boundary/package/workflow files | 147 pass, one Darwin-incompatible test deselected | 48.10 |

Commands: `cargo build --locked --offline --no-default-features --bin biomcp`; `cargo test --locked --offline --no-default-features --lib --no-run`. The resulting library test executable ran `sources::gnomad::`, `entities::gene::constraint_surface_tests::` and the fully qualified classifier and renderer names, each with `--test-threads=1`; the last two also used `--exact`. The actual caller run set `BIOMCP_BIN` to the rebuilt worktree binary. The affected Rust runs total 13 passes and 4.921 wall seconds.

Python ran through cached pytest with `uv run --offline --no-project --with pytest python -m pytest tests/test_biodata_boundary.py tests/test_source_package_boundary.py tests/test_biodata_branch_workflow.py -k 'not forged_offline_marker_fails_in_the_normal_namespace'`. `TMPDIR` points to the worktree's routine test output directory. The initial unrestricted selection reproduced the existing Darwin failure and exposed an incorrect temporary-directory invocation; the corrected temporary-path check passed before the final 147-pass run.

The exact-pin checker, capture-receipt checker, tracked-text scan, whitespace and Rust source-size audit pass. Source-size checks inspect 858 Rust files with zero findings. Gene shrank from 3871 to 3870 lines; its baseline and existing delta decreased by one. The focused selection contains 631 Rust owners and 15 Python selections. Four wholly affected Rust files pass rustfmt. The renderer leaf's changed setup was formatted; its unrelated inherited formatting remains preserved. Ruff 0.13.2 passes the pin checker and two boundary test files. The workflow file has the unchanged finding below.

## Inherited findings and host limits

Strict library Clippy with `-D warnings` fails with 12 diagnostics in unchanged code: dead-code findings in `src/entities/mod.rs`, `src/entities/drug/get/resolver.rs`, `src/entities/drug/get/trial_alias.rs`, `src/entities/drug/search/ranking.rs` and `src/entities/trial/mod.rs`; the fold in `src/mcp/shell.rs:1092`; the collapsible condition at `src/entities/variant/resolution/genomic_assertion.rs:148`; two prefix suggestions at that file's lines 297–298; a needless borrow in `src/entities/variant/search/exact_scan.rs:50`; and a while-let suggestion in `src/sources/opencitations.rs:54`. That attempt measured 46.73 wall seconds. Diff inspection confirms these diagnostic owners did not change.

Ruff reports inherited E731 at `tests/test_biodata_branch_workflow.py:258`; this ticket changes only that file's selection count. Repository-wide rustfmt reports inherited differences in `src/cache/migration.rs`, `src/entities/article/test_support.rs`, `src/entities/variant/resolution/interval_comparison.rs`, `src/entities/variant/resolution/interval_tests.rs` and unrelated blocks in `src/render/markdown/gene/tests/extended.rs`. The minimal cached pytest environment warns about its unavailable `asyncio_mode` plugin.

`test_forged_offline_marker_fails_in_the_normal_namespace` fails on Darwin because the Linux verifier reads `/proc/self/status`. This failure was reproduced before the explicit final deselection. The repository's network-isolation wrapper requires Linux. The M5 checks used locked offline dependencies and local fixtures; they do not claim Linux namespace qualification. `cargo-nextest` is unavailable on this host, so final affected execution used the compiled Rust test executable. The uncached complete docs dependency set was unnecessary for the focused Python files.

PM initially reported 207 existing record findings. Root owns their reconciliation. Fresh consumer CODE acceptance and landing remain pending. Local scratch logs use the `biomcp-2025-` prefix; artifacts and protected worktrees remain for their cleanup owner.

## Exact changed files

- `Cargo.toml`
- `Cargo.lock`
- `src/sources/gnomad.rs`
- `src/sources/gnomad/tests/parsing.rs`
- `src/entities/gene.rs`
- `src/entities/gene/constraint_surface_tests.rs`
- `src/render/markdown/gene/tests/extended.rs`
- `tools/check-biodata-boundary.py`
- `tests/test_biodata_boundary.py`
- `tests/test_source_package_boundary.py`
- `tests/test_biodata_branch_workflow.py`
- `tools/biodata-1.0-focused.toml`
- `tools/rust-source-size-inventory.json`
- `sdlc/tickets/2025-adopt-direct-gnomad-constraint.md`
- `sdlc/records/2025-adopt-direct-gnomad-constraint.md`

Fresh read-only CODE review ACCEPTED1b94ae4148f760a5ecefde5ea011046551d76acf with no product findings. Root inspected unchanged tested runtime and scoped gates before landing.

Landed: ccec533aaf33f49c45798e3ea211840851ffbb34
