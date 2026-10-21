---
priority: 1
---

# 2322 — Repair the final Linux routine failures

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 30bc63d, accept

## Outcome

The two Rust cases that failed in fresh Linux qualification pass under the same isolated environment on current main. Each failure has a demonstrated cause and a bounded repair. Preserve the product assertions, source-access rulings and shared timing policy.

- Proof: Reproduce both original failures on `b91c4d46bb33741e76065dbe81e3b406d0e1596c` through a Yellow build job with the qualification's actual compiler and isolation settings. Demonstrate the repair against those same cases and relevant controls before the normal candidate gate. Final release qualification remains owned by 2258.

Changes: `src/cli/stale_json_note_tests.rs`, `src/entities/article/graph/tests.rs`, `sdlc/tickets/2322-repair-final-linux-routine-failures.md`, `sdlc/records/2322-linux-routine-repair.md`, `CHANGELOG.md`. Correct this claim when the actual files are known.

## Evidence

Fresh Linux job `2258/j402` ran the published source above with Rust 1.93.1 after verified kernel network isolation and a fresh build. Preparation passed. The routine finished with 4,285 passed, two failed and 34 skipped; exit 100. Later stages did not run. All tracked source hashes stayed unchanged, and owned scratch was removed.

The failures were `cli::stale_json_note_tests::a_stale_clingen_prefetch_note_reaches_the_gene_card` at line 253, with `fixture includes an expired provider policy`, and `entities::article::graph::tests::citation_evidence_returns_the_bounded_error_when_the_bridge_stalls` at line 1010, with request count 0 instead of 1. Complete diagnostics and the actual job result are retained on Yellow under `/home/ian/workspace/data/biomcp/release-evidence/0.9.4/final-qualification/j402/`; its manifest SHA256 is `34b36b3ffe372a7204783e4b746c19c74ace63bf00d26f0223cf9aec9cda926c`.

## Development boundary

Restate the outcome, reproduce both cases with the exact command and output, then name the concrete cause and affected outside-in test before code. Check the qualification environment and shared fixture isolation before changing product behavior. Do not skip the cases, loosen their assertions, add fixed sleeps, or rerun whole qualification blindly. If the causes require unrelated changes, ask the coordinator to split them. Every build and test runs through `pm job`; one coherent review-ready branch push per completed repair.
