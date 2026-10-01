# Adopt source-scoped MyGene identity through BioData

Prepared October 1, 2026 under ticket 2007. Fresh design review is next. Product authority remains BioData 0182. This preparation executes no product code, compiler, provider call, fixture server, private-data read or hosted workflow. Root owns maintenance verification and later implementation assignment.

## Exact bases and accepted records

BioMCP shared 1.0 and this metadata parent are `11904b5658d8208d4d6b8bb79a3bbf49a6954344`. The remote dedicated branch independently matched that revision during preparation. Its BioData pin remains `c9938b99bd091ab4bf6da8b909ed239826e5ab6d`, version 0.0.28. The initial inspected BioData main was `18eb6e9c5dbb1119a466b2e7300c727029c0294c`, version 0.0.36. Root has since landed and pushed main `59fde6246c1a07ac115d6fc9f16471354a725bff`. Its library source is unchanged from 18eb6e9; exact Git diff shows only three 0203 completion records changed. The proposed adoption pin is accepted main `59fde6246c1a07ac115d6fc9f16471354a725bff`, subject to new exact-pair proof. HGNC publisher availability at that pin does not qualify MyGene claims.

Read the workspace and repository instructions, programme queue and consumer plan, existing current-readiness reconciliation, and these owning public records at the BioData base:

- [0182 ticket](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/tickets/0182-adopt-mygene-primary-gene-identity.md), [library acceptance](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/records/0182-gene-identity-library-phase.md), [source design](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/planning/mygene-gene-identity-design.md), and [compatibility map](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/planning/biomcp-compatibility/gene-identity.md).
- [0193 adoption policy](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/records/0193-record-gene-test-adoption-policy.md) and [gene preparation](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/planning/gene-identity-preparation.md).
- [0213 accepted correction and landing](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/records/0213-gene-ensembl-disposition-preparation.md) and its [finite Ensembl decision](https://github.com/genomoncology/biodata/blob/18eb6e9c5dbb1119a466b2e7300c727029c0294c/sdlc/planning/0182-ensembl-array-disposition.md). Final independent metadata review accepted `271cb66daf6997892b537a347e784a74140a54e0`; correction landed at `75e0fa48841801e6667edeeeb7fab5b933849bd4`.

Root verified hosted maintenance run [36849645376](https://github.com/genomoncology/biodata/actions/runs/36849645376) COMPLETED SUCCESS at exact BioMCP `11904b5658d8208d4d6b8bb79a3bbf49a6954344` against pin `c9938b99bd091ab4bf6da8b909ed239826e5ab6d`. This records root's supplied verified result without duplicate verification. It clears the hosted blocker for the maintenance baseline. BioData 0203 completion is now recorded on accepted main in [0203-hosted-verification-and-completion.md](https://github.com/genomoncology/biodata/blob/59fde6246c1a07ac115d6fc9f16471354a725bff/sdlc/records/0203-hosted-verification-and-completion.md). Root owns its scoped cleanup; the persistent BioMCP 1.0 lane remains. Historic release-handoff waits are superseded. The next start condition is this design's fresh review and root implementation assignment. Reconcile the actual passed candidate and maintenance tip before code starts and again before code review. A moved tip needs its own classified delta and exact review/proof.

## Source custody and product conversion

Acquire bytes through existing MyGene transport. Keep request plans, field lists, retries, cache mode, body bounds, HTTP status and JSON content-type checks. Pass original acquired bytes directly to `parse_mygene_query` with `MyGeneProfile::Get` or `Search`. Keep page digest, original ordinal, provider total, disposition, source text states, source raw row and adapter losses until conversion completes. Never rewrite a response to satisfy the parser.

Use `select_mygene_get` with the trimmed requested symbol. Two exact matches produce Ambiguous before any rejected row; otherwise rejection precedes mismatch. Only an empty page yields not-found and permits the existing alias retry or CLI suggestion. Exact comparison is case-sensitive. Search checks every disposition before filtering or ranking; one rejection fails the whole page. Missing provider total remains a product failure. Convert supplied u64 total to usize with a checked conversion. Never derive total or locators from filtered rows or substitute `_id` for an NCBI Gene code.

Convert BioData symbol/name/aliases and namespace-qualified NCBI Gene/Ensembl codes into the existing product fields. Keep admitted missing/null/blank states internally when display collapses to blank. Decode only source-only summary, type, coordinates, MIM, UniProt, pathways and needed transcript/protein members from the accepted raw row. Preserve source-only shape checks and enrichment behavior. Keep BioData identities and adapter loss together with a small product conversion report until assembly completes. The report carries fixed field/reason losses for alias display filtering/cap, canonical HGNC lexical conversion and singular Ensembl display. Do not invent a new CLI/MCP envelope or publish raw rejected text. Product tests inspect the internal report and existing public fields. Strict `Document::GeneIdentity` round-trip belongs in the converted-identity proof; it does not add a public command.

MyGene symbol/name/alias and HGNC codes remain source assertions. Qualified nomenclature remains absent. The separate HGNC publisher adapter in accepted BioData requires its own publisher origin and qualification binding. This slice performs no publisher lookup, joins, indexed input qualification or source admission.

For existing GenCC matching, normalize source-asserted HGNC codes in BioMCP: trim optional case-insensitive prefix, require ASCII digits, check nonzero u32, emit `HGNC:<canonical decimal>`, then deduplicate equivalent normalized spellings. Invalid text, overflow or multiple distinct canonical IDs remain inconclusive. Retain source spellings, numeric-shape flags and conversion loss. ClinGen keeps its independent lookup. Preserve prefetch cache context, stale-serve collection, cancellation, settlement and section outcomes. Neither join grants HGNC naming authority.

Remove duplicate authoritative get/search identity structs, identity transform reads and HGNC wire parsing after adopted get/search, alias and source-only enrichment proofs pass. Any temporary decoder must name that removal trigger and be gone before completed adoption. Keep batch-symbol decoding, query escaping and needed coordinate/transcript/protein helpers.

## Exact affected files and protected contracts

All paths below are relative to BioMCP. Source inspection found the listed gene, CLI, MCP, spec, integration-error and focused-runner files unchanged between the September 30 readiness base `6ca701f2513554636c08423a2a2e801f8c3e0b52` and actual `11904b5`. This is static continuity, not runtime acceptance.

| Files | Bounded change or retained contract |
| --- | --- |
| `Cargo.toml`, `Cargo.lock` | Pin accepted exact BioData revision and regenerate lock under later build authority. No floating/path dependency. |
| `tools/check-biodata-boundary.py`, `tests/test_biodata_boundary.py` | Update exact revision/version oracle from 0.0.28; retain retired-owner, dependency and document checks. |
| `tests/test_source_package_boundary.py`, `tests/test_biodata_model_reference.py`, `tests/test_biodata_publication_reference.py`, `tests/test_biodata_model_reference_surfaces.py` | Inspect existing hardcoded version/pin expectations and update only exact-pin requirements made stale by adoption. Preserve trial/publication behavior and public documents. |
| `src/sources/mygene.rs` | Separate byte acquisition from decode; get/search project through BioData. Replace legacy identity structs and HGNC parser. Retain request plans, transport, batch-symbol and UniProt helper behavior. |
| `src/transform/gene.rs` | Convert accepted identity and narrow source-only record. Preserve alias policy, summary, coordinates/build/provenance, OMIM, UniProt and KEGG cap. Three rewritten card tests and source-only extraction proof. |
| `src/entities/gene.rs` | Use projected rows in canonical aliases, local filters and get/search assembly. Preserve uniqueness/ambiguity, NCBI requirement, exact-result promotion, first-wins dedupe, distinct IDs, 50/51 follow-up, region/type/chromosome filters, slicing and counts. Keep `mygene_query_term`, used by variant diagnostics. Normalize existing GenCC HGNC input. |
| `src/entities/gene/gencc.rs`, `src/entities/gene/clingen.rs`, their tests and `src/sources/gencc*`, `src/sources/clingen*` | Inspect as protected consumers; retain current matching and maintenance behavior. No enrichment redesign or new ClinGen route. |
| `src/cli/gene/mod.rs`, other existing `src/cli/gene/` files, `tests/unit/cli/gene.rs` | Retain routes, help, section metadata, rendering and not-found alias suggestion. The included gene test module runs in the library as `cli::gene::tests::*`. |
| `src/mcp/shell.rs`, `src/mcp/shell/typed_get.rs`, `src/mcp/shell/typed_get_tests.rs`, `src/mcp/shell/modern.rs`, `src/mcp/shell/http_server.rs` | Preserve flat schema roots, mapper validation, raw/typed dispatch and structured output. Existing catalog/section tests remain. No schema rollback. |
| `tests/json_error_contract.rs` | Retain actual executable not-found and fixture shutdown. Add one authored get/search/Ensembl error table using bounded local fixture behavior under later implementation authority. |
| Proposed `src/entities/gene/identity_surface_tests.rs` and its module declaration in `src/entities/gene.rs` | One actual CLI/raw/typed MCP table over shared local fixture cases. Reuse existing dispatch/fixture support; create no general framework. Proposed selector `entities::gene::identity_surface_tests::cli_raw_typed_gene_identity_table`. |
| `spec/entity/gene.md` | Keep actual command get/search, alias, pagination, sections and JSON surface assertions. Reconcile only deliberate identity compatibility losses. |
| `tools/biodata-1.0-focused.toml`, `tools/check-biodata-1.0`, `tests/test_biodata_branch_workflow.py` | Add reviewed selections, include integration target in discovery/run, validate every name exactly once and test target inclusion. Preserve selected trial/literature/Python regressions and network isolation. |
| `tools/rust-source-size-inventory.json` and other existing changed-source inventories | Reconcile measured source ownership after implementation; no automatic ceiling increase. |
| `src/sources/mygene/tests/live.rs`, `tests/live_smoke.rs` | Six selectors stay outside offline gates; two adopted-path live replacements wait release-candidate authority. |

The new surface test filename and selector are a reviewable recommendation. Root can choose existing support placement without broadening behavior. Enrichment helper extraction may stay in the existing source/transform modules; the narrow source-only record must contain no second identity owner.

## Selector disposition and receipts

[selectors.tsv](selectors.tsv) preserves each historical donor declaration, behavior, reason and current fully wired name. All 61 current function declarations resolve by exact Git source inspection. The totals remain 44 offline retained, 10 rewritten, 1 omitted and 6 live outside the gate. Retention uses two current MCP names: `typed_branches_stay_entity_specific` and `typed_schemas_publish_flat_roots_and_reject_bad_input`. Direct typed-get test functions have no extra `tests::` module. Integration functions have top-level names. Actual nextest discovery remains prospective.

The ten rewrites cover two get/search legacy decode tests, HGNC wire normalization, fixture-derived UniProt extraction, three gene-card conversions and three canonical-alias conversions. Keep their existing distinct behavior assertions over projected rows. Source corpus coverage belongs to BioData; consumer conversion remains in BioMCP. The omitted `string_or_vec_into_vec` is covered by `utils::serde::tests::string_or_vec_helpers_cover_all_shapes`. Verify that retained helper declaration during implementation. Keep the unrelated legacy batch fixture as behavior proof, without promoting it to source truth. BioData's ten current gene test declarations, including two 0198 numeric HGNC tests, are separate from the donor count.

[static-evidence.json](static-evidence.json) records all five exact capture SHA-256 comparisons against `testdata/sources/capture-receipts.json`. Each matches at the exact BioMCP base. Four get captures have single-object nonblank scalar Ensembl genes; BRAF search has no Ensembl field. FLT3 supplies an HGNC source claim. BRAF aliases include display-filtered `B-raf` and `BRAF-1`. Receipts for the August captures state whitespace normalization after capture; FLT3 states unmodified bytes. Digest verification proves correspondence to those stored receipts, not untouched historical acquisition or redistribution permission. Keep captures in BioMCP. Legacy `search_egfr.json` remains unverified source evidence.

## Finite future offline proofs

Reuse the accepted BioData corpus for exhaustive source/document constructors, unknown/duplicate members, field states and limits. Add compact consumer tables with authored inputs and independently declared expected fields/errors. No fixtures or test implementation are created here. Existing receipted captures remain unchanged.

The Ensembl table has these fourteen families. Use the same authored page cases in CLI JSON, raw `biomcp`, and typed get/search. A page has explicit total and original ordinals; assert digest/disposition/loss internally and the existing sanitized public envelope externally.

| Family | Adopted result |
| --- | --- |
| E01 whole field omitted/null | Projected; no ID |
| E02 standalone object missing/null gene | Projected; no ID |
| E03 standalone blank/whitespace gene | Projected; no ID; explicit blank-display loss |
| E04 empty array | Projected; no ID |
| E05 missing first array gene with later valid gene | Identity rejection; terminal get/page failure |
| E06 null first array gene with later valid gene | Identity rejection; terminal get/page failure |
| E07 blank/whitespace first array gene with later valid gene | Identity rejection; terminal get/page failure |
| E08 valid first gene with later missing/null/blank member | Identity rejection despite usable first member |
| E09 two distinct genes | Preserve both claims; display first object; report singular-display loss |
| E10 valid genes with heterogeneous transcript/protein members | Preserve raw source-only members and source order; same display rule |
| E11 repeated identical genes | Preserve raw repeat; one shared code and adapter duplicate loss |
| E12 gene vector in single object or array object | Accept claims; first code of first object displayed; report other distinct claims |
| E13 empty gene vector, including first empty vector then valid later member | Accept; no ID; never substitute later member; report omission |
| E14 numeric gene, null member, nested array or scalar Ensembl | Identity rejection; terminal get/page failure |

E05–E08 deliberately replace old successful absent/blank/first-valid display with terminal errors. Search now validates supplied Ensembl that legacy search ignored. The receipts do not establish legitimate missing-gene array members or universal provider invalidity. No byte sanitation, later-member repair or legacy parser fallback is permitted.

The page table has ten families from accepted 0213: empty hits; exact-symbol mismatch/case difference; missing/null/blank symbol/name; missing/null/blank/repeated provider ID; two exact matches; one exact match plus a rejected nonmatching row; two exact matches plus a rejected row; distinct/equivalent HGNC claims; duplicate JSON/malformed envelope/limits; missing or overflowing total. Preserve Ambiguous-before-rejection precedence. Only actual not-found retries aliases. Search cannot hide a rejection behind local filters. Parsing failure before a page exists cannot promise ordinal/total retention.

Add the existing success and conversion checks to those tables: numeric/string Entrez, scalar/array alias custody with five-item display loss, checked HGNC joins including zero/overflow/conflict, strict gene-document round trip with absent qualified nomenclature, accepted capture fields, source-only coordinates/summary/UniProt/OMIM/pathways, search provider order versus retained ranking/count policy, and section continuity. Do not expose internal source records in errors or add public fields. Keep the three conversion and three alias rewrite obligations distinct; consumer source-state assertions may share one compact table.

## Focused execution and timing after authority

The current runner discovers and executes `--lib` only. Later adoption adds `--test json_error_contract` to both nextest discovery and execution, selecting retained top-level `json_mode_gene_not_found_error_writes_json_stdout_and_exit_1` and `mygene_fixture_without_request_stops_on_drop`, plus proposed `gene_identity_error_table`. Extend its existing runner contract test to prove both targets are present in discovery and execution. Add the library surface table selector and the retained/rewritten gene selectors to the manifest. Deduplicate already selected catalog proof. Retain all selected trial/literature and Python selectors. Discovery must prove exactly one runnable match per selection and reject zero/duplicate matches; no ignored live selector enters the list.

Future commands use the existing authorized BioData Linux isolation path after dependency preparation outside isolation: locked offline metadata validation; `cargo build --locked --offline --no-default-features --bin biomcp`; worktree-local `BIOMCP_BIN` and `tools/check-biodata-1.0 --already-isolated`. Record the build command wall separately. Record the complete focused command wall including preflight, receipt checks, discovery, compilation, nextest, Python collection and process overhead. Mark separate compilation or prebuilt execution unmeasured unless measured directly. The current harness does not prove the proposed names execute until discovery runs.

Record exact BioMCP/BioData pair, incorporated maintenance revision, host/platform, toolchain/image, command order, dependency/target cache state, load/competing work, exit codes, complete logs and digests for each proof. Compare only like host/cache/order measurements. Run required BioData lint/test/spec under 0182 and retain the active 866-second full-test guard; it is neither a BioMCP ceiling nor a gene adoption measurement. Both new gene build and complete offline-test times are currently not measured. No estimate or favorable repeat is authorized here.

After fresh code review and required local proof, root advances the dedicated public branch and dispatches the existing hosted verifier for that exact public tip. Record its derived dependency and selected result separately from maintenance run 36849645376. A started workflow or earlier pin's PASS supplies no gene acceptance. Live source checks and broad final release qualification remain separate. Record final behavior, selector reconciliation, receipts and unresolved gaps under existing 0182 and BioMCP completion metadata.

## Review handoff and deferred gaps

Root assigns a fresh read-only design reviewer to this exact metadata commit. Review must return ACCEPT or finite findings on file/contract coverage, selector count and wiring, source uncertainty, HGNC authority, strict Ensembl compatibility loss, integration-target selection, finite proof and timing boundaries. This author performs no independent self-approval and creates no child worker. Once reviewed, root can assign implementation against the accepted maintenance baseline.

Defer HGNC publisher acquisition/qualification, index rebuild/parity, cross-source reconciliation, other gene enrichment migrations, providers, private data, performance calibration and release publication. Ian can overturn the admitted-source default or compatibility decisions through a recorded source-contract amendment. Root can revise routine preparation details within accepted 0182.
