---
priority: 2
---

# 2123 — Report coverage overlap and collection completeness

Status: OPEN.

Milestone: 0.9.3

## Outcome

The separate coverage command reports outside-only, internal-only, both and unobserved product lines over one stated denominator, and separately identifies failed or missing collections and incomplete results.

Depends on: 2117
Depends on: 2120

- Starts from: `tools/measure-0-9-3-baseline` and its existing profile collections.
- Keeps: LLVM source coverage, subprocess profiles, collection failures and existing collected profiles; group reporting does not rerun suites solely to derive partitions.
- Changes: claims `tools/measure-0-9-3-baseline` `CHANGELOG.md`. Derive line groups by file and line from machine-readable exports of the existing profiles. Exclude tests, including inline test modules, and generated code from the product denominator. Name a focused report-parser test in this claim before adding it.
- Proof: deterministic report cases cover outside-only, internal-only, overlap, unobserved, zero-hit and failed/missing collections; the four exclusive groups sum to the denominator. Run `--only coverage` through the build job separately from routine qualification and record completeness. A failed collection never produces an unqualified percentage.
- Defers: final comparison of both product versions to 2082.

PM decision: 2026-10-17. Coverage runs on the common build machine. Collection completeness is a separate reporting dimension, not a fifth disjoint set of source lines. Resolve the script's inline-test denominator before calling the result product-only.
