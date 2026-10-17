---
priority: 1
---

# 2177 — Keep fixture output out of repository scans

Status: OPEN.

Milestone: 0.9.3

## Outcome

Fulltext replay fixtures create temporary server and cache state inside ignored test output, so concurrent repository scans never read files that fixture cleanup can remove.

- Starts from: main beb08a9f3 and the existing fulltext fixture entry point.
- Changes: claims `crates/biomcp-mcp-contract-client/src/lib.rs` `tests/fulltext_fixture_output_location.rs` `CHANGELOG.md`. Request an expansion before changing other files.
- Keeps: recorded replies, exact-loopback authorization, Unicode cache paths, cleanup, bounded child lifecycle and the existing repository audit coverage of tracked and new source files.
- Proof: invoke the real fixture entry point on current main and show its generated paths appear in the untracked repository source inventory. After the fix, the same generated state belongs to the existing ignored test-output area, stays absent from that inventory while the fixture is live, and is removed by normal cleanup. Keep source scanning intact. Run the affected fixture and audit contracts through the build job.
- Defers: fixture identifier refactoring, broad audit rewrites and unrelated process changes.

## Evidence and implementation brief

Gate 1792260718966-18437 ran make landing-gate on the integrated supplement candidate. The Python quality-wrapper contract failed in check-zero-coupling.py while read_bytes opened a temporary fulltext fixture cache lock that concurrent cleanup had removed. The path was beneath an untracked biomcp-rmcp-fulltext directory at the repository root. That fixture directory layout also exists on current main. The gate separately found two supplement worker helper conventions; those remain in 2152.

Restated outcome: runtime fixture files must never enter the repository source scan. Reproduce their visibility through the actual fixture command before changing its output location. Prefer the existing ignored test-output directory; do not suppress audit errors or hide tracked source. Name the exact controlled red command and result before the product edit. Keep this repair small and independent of supplement extraction code.

Claim correction after source inspection: the shell already places server state under ignored .cache output. The caller provision_article_fulltext_fixture in the MCP contract client instead creates its outer temporary directory directly beneath the repository root and places its Unicode cache beside .cache. Repair that allocation in the Rust helper. The root integration test must invoke the real public helper and shell, inspect repository inventory while live, and verify Drop cleanup; it must join the existing routine test archive. Do not change the already-correct shell output location.

## Executed proof

Controlled red job 1792261305049-65518 on published beb08a9f3 applied only the new root integration test and ran cargo test --locked --no-default-features --test fulltext_fixture_output_location -- --nocapture. It failed one case because the live real fixture exposed a Unicode cache lock in Git’s untracked source inventory. The actual helper started its replay server and Drop cleanup ran while unwinding.

Developer candidate 3a714819e5a9dd05784d3c5a8d12481b2429b652 moves only the caller’s workspace parent to .cache/routine-test-tmp. Rust green 1792261398923-71243 passed that real-helper regression, formatting and targeted Clippy with warnings denied. Python-only follow-up 1792261479951-74998 prepared dependencies without installing the project and passed 36 contracts, including the original failing quality-wrapper case, plus tracked-text and whitespace checks. Source and Rust proof remained unchanged across that environment preparation. Fresh review remains required.

Fresh review accepts 494ae96192024087eae7ed57dfd4fc554b2372f2 without changes. The helper file has no other repository-root temporary allocator; other study and harness state uses system temporary storage. Reviewed product patches match the qualified Rust and Python job patches. The live inventory and cleanup regression exercises the actual helper, and replay/authorization behavior remains unchanged.
