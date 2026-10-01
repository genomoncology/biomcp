# Implement primary MyGene identity through BioData

Date: October 1, 2026. Status: implementation ready for fresh independent code review. Root approved the corrected design at `10a8dc4d1a071ab1101bc5c0c384137665ad8b84` and recorded implementation authority at `95fbc4330184ad269e121574c77b43c0dc48a5ab`. BioData 0182 remains product authority. BioMCP 2007 records this lane and its completion. Product adoption acceptance, shared branch advancement, hosted verification, and release remain pending.

The checked code candidate is `ec1a9b6a426f908fa31c97594c8cda8be41714b4`. This record is a metadata child of that candidate. All implementation fixes were committed and pushed to the ticket branch. No agent, worker, Pi job, or other model was invoked.

## Ownership and retained behavior

MyGene transport acquires original bounded bytes and retains request plans, cache mode, status/content-type checks and retries. Get uses BioData's Get profile and exact selector. Search uses its Search profile and rejects every failed disposition before local filtering. Only actual empty get permits existing alias resolution and suggestion. Required totals use checked conversion. The page stays with each projected record through assembly. It retains digest, total, ordinals, source field states, raw accepted rows and adapter losses.

Product conversion reads shared symbol, name, aliases and namespace-qualified codes. Source-only decoding retains get summary, type, coordinates, MIM, UniProt, pathways and transcript/protein shape checks. Search retains its narrower former enrichment checks. Product losses describe alias filtering/cap, HGNC lexical join conversion and singular Ensembl display. MyGene text and codes remain source assertions. No publisher lookup, qualified nomenclature, indexed qualification, or new public envelope was added.

Strict Ensembl rejection and first-object display follow accepted 0213. Missing/null/blank array genes fail terminally; later members do not repair a rejection. Admitted distinct/vector claims remain shared claims. Empty first vectors never select a later gene. Fixed losses describe undisplayed claims and blank/empty display. GenCC receives checked canonical codes; equivalent spellings deduplicate and conflicting or invalid values remain inconclusive. ClinGen retains its independent lookup and cancellation/context behavior.

The consumer controls pass before retirement is accepted. The completed candidate contains no legacy get/search identity struct, identity transform decoder, or HGNC wire parser. The boundary checker now rejects the five retired struct/enum names. Batch decoding, query escaping, coordinate helpers and source-only transcript/protein checks remain. Compilation exposed two additional callers: cell-line Ensembl lookup and variant diagnostics alias candidates. They now consume projected identity without changing their routes.

CLI JSON, raw MCP, typed get/search, Markdown, aliases, filtering, ranking, counts, errors and sections retain their public formats. One shared authored control table drives the library surface proof and integration error proof. It covers E01–E14, page/selection neighbors, required totals, original-byte digest/ordinal/total custody, strict document round trips, absence of qualified naming, sanitized failures, and terminal errors without alias retry. Four receipted gets and the receipted BRAF search prove conversion. Local-filter, alias filtering/cap, source-only field and contextual cancellation controls remain distinct.

## Dependency and maintenance

Cargo regenerated the actual lock and locked offline metadata resolves only BioData `59fde6246c1a07ac115d6fc9f16471354a725bff`, version `0.0.36`. The sole lock delta is BioData version and immutable source URL revision. Dependencies and other package/checksum entries are unchanged. Metadata contains 490 resolved packages. HTTPS preparation requested authentication and failed without using credentials. The approved commit already existed locally and was fetched into an experiment-owned Cargo Git cache. The declared Git URL and exact revision remain intact; no path/source override was introduced.

The exact catalogs and relevant historical receipts are unchanged between old and new library pins. Stale website adoption metadata and generated revision citations were refreshed without changing catalog, provider fixture or download bytes. The source package retains its prior 1,449 members and adds exactly three modules: projected source conversion, library surface controls and their authored shared table. The checked package count is 1,452.

[Maintenance reconciliation](2007-maintenance-reconciliation.md) classifies the full `a2e5a901..9e960ee3` interval. Normal merge `44f893a4c7fedbf160cd8de00a755d79786df68a` preserves ancestry and records the development changelog/test resolution. Fetches immediately before the handoff still return main `9e960ee32102f638e4a95e69e70b88264b496f07`; the subsequent remote interval is empty. The persistent 1.0 worktree, 0.9 checkout and root main were not edited or advanced by this builder.

## Offline implementation evidence

Use the local receipt directory `experiments/633-biomcp-2007-consumer` in the workspace. Artifacts remain unpushed. It retains exact commands, stage stdout/stderr, exit codes, walls, log digests, locked metadata, dependency classification, final binary digest, source hashes, source-size/dead-code results and executed-selection reconciliation. Failed preparation, build and test receipts remain intact.

Execution used the cached Linux ARM64 image `sha256:0e8a27a766cfb6041ea572a4bb42659232272f17d8d2ce62d68b577d7e6c32db`, Rust/Cargo 1.93.1 and Python 3.12.3. Host helpers and child PATH use installed Python 3.14.5. Bubblewrap and network-none block public transport while allowing loopback fixtures. The owned Cargo/target caches were copied from existing preparation caches; the target was warm, not clean. Public nextest 0.9.132 was prepared outside isolation and its official release-asset digest verified. No global installation, credential acquisition, live provider, paid service, private payload or hosted dispatch ran.

| Implementation command | Result | Actual launcher wall seconds |
| --- | --- | ---: |
| Final locked offline metadata | PASS | 0.640084 |
| Final `cargo build --locked --offline --no-default-features --bin biomcp` | PASS | 17.944230 |
| Final selected nextest command, including compilation and execution | 63 passed | 60.481055 |
| Changed dependency/reference Python controls | 218 passed, one declared skip | 25.275020 |
| Final format check | PASS | 4.012020 |
| Final capture receipt check | PASS | 2.750851 |

The final Rust command ran on exact code candidate ec1a9b6a. It combines 57 gene selections with six existing GenCC/ClinGen/cache-context controls. Successful terminal events reconcile every selected name exactly once across library and integration targets. Nextest reported 15.970 seconds for its execution phase; the complete launcher wall above includes compilation and overhead. Build wall is measured separately. The Python controls ran on 1485ed10; their files and dependency-reference implementation are unchanged in ec1a9b6a. Later changes add Rust capture/filter assertions and formatting only. Source-size and dead-code inventories pass with zero findings and no ceiling increase.

The donor table remains exactly 61 declarations: 44 retained, 10 rewritten, one omitted with its retained utility replacement, and six live outside the gate. All 175 manifest Rust selections resolve exactly once. Discovery and execution both use `--lib --test json_error_contract`. The loader admits exactly three top-level integration names and qualified library names. Two exact schema assertions need a bounded exception to the pre-existing publication-word guard. They perform schema validation, not publication. Negative controls reject missing/duplicate/ignored discovery, missing/duplicate/extra/ignored terminal execution, arbitrary top-level names and forbidden live/provider/credential selections. Existing trial/literature/Python selections remain.

Preserved failures include HTTPS dependency preparation, missing nextest preparation, the first build's two additional callers, loader rejection of the two schema assertion names, incomplete container Git mounts, initial surface expectation/context fixtures, and stale website/package expectations. Every corrected stage has a later passing receipt. The 685-second guard is read from the exact BioData pin; it is not a BioMCP ceiling.

## Fresh review and finite final proof

Root assigns a fresh read-only code reviewer to the final pushed ticket tip. Review the original-byte boundary, selection precedence, singular display/loss, HGNC join conversion, source-only profiles, both additional callers, the exact dependency closure, website pin-only changes, maintenance resolution, removed identity ownership and runner reconciliation. The full 175-selection composite command remains unmeasured and unexecuted here. These are implementation checks; no final composite acceptance is claimed.

After ACCEPT, root prepares the exact reviewed source pair and runs these finite commands inside the accepted Linux offline isolation, measuring build and complete focused command separately:

```sh
cargo metadata --locked --offline --format-version 1
cargo build --locked --offline --no-default-features --bin biomcp
install -m 755 "$CARGO_TARGET_DIR/debug/biomcp" target/debug/biomcp
BIOMCP_BIN="$PWD/target/debug/biomcp" tools/check-biodata-1.0 --already-isolated
```

The install line applies to the recorded external target cache; omit it when Cargo already emits the worktree-local target binary. Require equal executable digests. The reviewed manifest supplies the finite 175 Rust selectors and retained 15 Python selectors. Do not substitute a broad BioMCP gate. Record actual complete composite wall including preflight, receipts, discovery, compilation, nextest and pytest overhead.

Root then runs the required BioData `sdlc/scripts/lint`, `sdlc/scripts/test` and `sdlc/scripts/spec` at the exact accepted dependency source, under its active 685-second test guard. Root must refresh BioData's hosted verifier manifest digest through its own reviewed metadata lane before dispatch: the current verifier still pins the old focused-manifest digest `4d36695e34edd8fbacc3e40132e0de194c444bc8c913201b1ab75b185c379577`. The immutable library pin remains 59fde62. This builder makes no BioData/root/shared-branch edits. Shared advancement and exact-tip hosted verification remain later root steps. Live qualification and publication remain separate.

Ian can overturn the accepted source/compatibility defaults. No new human decision or maintenance handoff wait is introduced.
