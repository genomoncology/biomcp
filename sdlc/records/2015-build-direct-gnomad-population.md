# Build the direct gnomAD population adoption

October 7, 2026. Global BioMCP ticket 2015, milestone 1.0, component backend. Build owner: assigned Codex consumer builder. Status: consumer BUILD complete; fresh CODE review pending. Root owns landing. The branch is `ticket/2015-adopt-direct-gnomad-population`; the base authorization commit is `e45d431e` over maintenance target `9ada31a9ef63322938bcde082efbcb53d13364ac`, which incorporates reviewed `37631c35`.

The exact BioData Git pin is `25f0c7fd3cee947230231aae0e3ed3aa960ecd46`, version `0.0.38`. Local producer HEAD and the ticket Accepted producer section agree. Producer CODE ACCEPT is `9169bfe1f5f7a1a8ac6c8fb620271f0856346c21`, unchanged runtime `ef4a5bbe4c36b3e273a28856685c0ec887af9729`. Root qualified this producer and owns its recorded Linux lint/test/spec results. No producer files changed during this build.

The existing bounded HTTP decoder now delegates the original variant member to `GnomadV4Population::deserialize_variant`. An optional Deserialize tuple wrapper owns no graph fields. `GnomadVariantPopulation` remains only a compatibility alias for the shared projection. The product envelope stores shared exome/genome records transferred with `into_parts`. Its optional Serde bridge delegates target encoding and decoding. Markdown borrows shared getters and passes the selected ancestry row through `as_population_target`. Source frequencies remain count-derived; supplied finite product-target frequencies remain retained. No Value carrier, serialization/reparse, model mirror or whole Variant migration was introduced.

Product owners retain HTTP limits, caching/retries, returned-ID correlation, coordinate admission, section outcomes, dataset/release/caveat, provenance, GraphQL policy, ancestry ranking/labels, filter explanations and compact/details display. Gene constraint remains unchanged. Whole Variant/VariantSearchResult ownership and other enrichment boundaries remain deferred.

## Outside-in evidence and retirement

The new caller leaf was written and registered before production implementation. Its native call requires `&biodata::GnomadSequencingPopulationProjection` from the returned product envelope. Red compilation failed with E0308 because the prior caller returned `&GnomadSequencingPopulation`. This proves shared storage adoption red; it does not claim runtime red for preserved output assertions. The first offline preparation attempt lacked the accepted commit in Cargo's cache. Fetching that exact commit from the local producer into Cargo's cache resolved preparation. No provider acquisition occurred.

`shared_direct_population_reaches_native_cli_and_typed_raw_mcp` owns unequal synthetic exome/genome records through the actual HTTP client, native JSON/Markdown, CLI and typed/raw MCP JSON/Markdown. Compact and details modes check independently authored JSON fields and selected ancestry/FAF Markdown literals. `direct_population_section_admission_and_private_failures` owns the local-fixture table for default no-call, details alias, all selection, one-sided data, zero denominators, null/absent data, not-found, wrong returned ID, malformed shared graph, errors alongside data, missing GraphQL messages and mixed messages. It checks section status, null sides, request correlation and private output.

Existing owners retain GRCh38/dbSNP recovery, section nulls/aliases/metadata, supplied target frequencies, row tie breaks, residual labels, raw IDs, filter explanations and finite/null/zero/partial-side selection. The defensive nonfinite claim moved to the renderer's numeric selection boundary because immutable shared records cannot admit nonfinite values. It adds no mutable test mirror.

Retired exactly four field-owning structs (`GnomadVariantPopulation`, `GnomadSequencingPopulation`, `GnomadFaf95`, `GnomadAncestryPopulation`) and the local `frequency` function/mutation loop. Retired only `variant_population_keeps_exome_genome_faf_flags_and_numeric_frequencies_separate`; the producer public graph owner covers its mapping semantics and the new actual channel owner covers consumer delegation. The recorded fixture and GraphQL error owner remain. Removed the obsolete mutable ancestry test constructor while retaining its nonfinite guard claim.

The retained fixture SHA256 is `582346a4a345cf4e30215c9e7d7b8e0963bf3c8ec149a3029b2c4d41450c58e0`. Its bytes and capture receipt remain unchanged. No files, worktrees, branches or provider data were deleted.

## Focused checks and observed costs

Host: `imaurer-m5`, Darwin arm64, Rust 1.93.1. Rust commands use locked offline Cargo, no default features, two jobs, debug info disabled and incremental compilation disabled. The shared development target reused available artifacts. Initial red compilation rebuilt dependencies for the new pin; subsequent builds reused that graph. Concurrent external load was not measured. The runner sampled free disk; its minimum stayed above 157 GiB. No full BioMCP gate, release gate, hosted CI or live provider test ran.

| Check | Actual result | Observed seconds |
| --- | --- | ---: |
| Shared-type red library-test compile | E0308, intended failure | 46.701 |
| First implementation library-test compile | Pass | 38.876 |
| Compile with section-table owner | Pass | 42.828 |
| CLI build | Pass | 31.048 |
| Intermediate final library-test compile | Pass | 40.962 |
| Final source library-test compile after formatting correction | Pass | 38.845 |
| Native population selection | 19 pass; test body 5.43 seconds | 8.192 |
| Full gnomAD source owner selection | 11 pass; test body 0.01 seconds | 1.389 |
| Native/CLI/typed/raw MCP positive channel owner | 1 pass; test body 1.78 seconds | 2.003 |
| Updated boundary contract file | 107 pass, inherited config warning | 20.62 |
| Exact package pin plus focused-runner contracts | 19 pass, one inherited Darwin failure | 0.65 |

The Rust selections contain 27 distinct passing owners. Four direct-source tests occur in both native population and source selections. Repetition is execution overlap, not duplicated owning tests. The second final compilation follows the equivalent ownership-transfer closure needed to meet formatting and the existing 1447-line getter ceiling. The CLI channel binary predates that equivalent closure spelling only; the complete shared source/storage/rendering implementation is identical.

Commands: `cargo test --offline --locked --no-default-features --lib --no-run -j 2`; `cargo build --offline --locked --no-default-features --bin biomcp -j 2`; the built library-test executable with `population --skip shared_direct_population_reaches_native_cli_and_typed_raw_mcp --test-threads=1`, with `sources::gnomad:: --test-threads=1`, and with the fully qualified positive channel name plus `--exact --test-threads=1`. The channel invocation sets `BIOMCP_BIN` to the built CLI. Python uses the existing cached environment through `uv run --offline --no-project --python 3.14 --with pytest python -m pytest`, selecting `tests/test_biodata_boundary.py`, then the exact manifest pin test and `tests/test_biodata_branch_workflow.py`.

The exact-pin boundary checker passes. Changed Rust files pass rustfmt with `skip_children=true`. Whitespace passes `git diff --check`. The source-size check passes across 856 Rust files with zero findings. Getter tests measure 1334 lines; renderer tests measure 1078 lines. Their baselines were lowered to those exact totals. The getter remains 1447 lines; no ceiling increases. The focused manifest registers two new caller owners and two retained source owners, advancing 620 to 624.

Inherited findings remain visible: `test_forged_offline_marker_fails_in_the_normal_namespace` fails on Darwin as recorded by 2014; this attempt reproduced that same diagnostic assertion failure. Strict Clippy remains unpassed from the inherited 2014 results (10 library and 18 library-test errors on unchanged diagnostic owners). Clippy was not rerun for this focused build. Cargo still reports unchanged dead-code warnings, including SearchPage fields/cursor methods. The minimal cached pytest environment retains its unknown `asyncio_mode` warning. No warning suppression, gate acceptance or Linux isolation qualification is claimed.

PM status reports a clean worktree and 206 existing record findings. Its unchanged ticket 2015 Risk line is one finding. No finding names the new build record; this build does not repair repository-wide record debt.

Preparation/invocation failures supplied no acceptance evidence: the first offline dependency-cache lookup, Python executables without tomllib or pytest, and a short unqualified exact test selector that discovered zero tests. Each was corrected before the stated passes. Local logs use the `biomcp2015-` prefix under `/private/tmp`; the red, implementation/owner/final compile, CLI, native, source, channel, boundary and pin-selection logs preserve output and measured costs. They are scratch evidence; this repository record owns the conclusions.

## Exact changed files

- `Cargo.toml`
- `Cargo.lock`
- `src/sources/gnomad.rs`
- `src/sources/gnomad/tests/parsing.rs`
- `src/entities/variant/mod.rs`
- `src/entities/variant/get.rs`
- `src/entities/variant/get/tests.rs`
- `src/entities/variant/get/direct_gnomad_population_transport_tests.rs`
- `src/render/markdown/variant.rs`
- `src/render/markdown/variant/tests.rs`
- `tools/biodata-1.0-focused.toml`
- `tools/rust-source-size-inventory.json`
- `tools/check-biodata-boundary.py`
- `tests/test_biodata_boundary.py`
- `tests/test_source_package_boundary.py`
- `tests/test_biodata_branch_workflow.py`
- `sdlc/tickets/2015-adopt-direct-gnomad-population.md`
- `sdlc/planning/direct-gnomad-population-2015/design.md`
- `sdlc/records/2015-build-direct-gnomad-population.md`

The four extra contract files were named to Root before editing their exact revision/version/count constants. Fresh consumer CODE review must inspect the whole diff from `e45d431e` to this branch tip. Root owns any review fixes assignment, final focused verification, landing and ticket closure. This record makes no CODE acceptance claim.
