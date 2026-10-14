# 2085 — Refuse a drug name several products share

Status: OPEN.

Milestone: 0.9.3

## Outcome

`get drug "Pain Relief"` refuses as an ambiguous drug name, exits 2, and lists the ingredients of the products that carry that brand name.

## Evidence

Split from 2066 on 2026-10-14. Found by the post-release install and product test of v0.9.2.

- `get drug "Pain Relief"` refuses, but exits as not found and lists "Pain (HP:0012531)" and "Pain Relief (Symptom, NCIT:C112216)". The 0.9.2 changelog promises it names what matched among drugs.

- Starts from:
  - `src/entities/drug/get.rs`, `openfda_label_identity_candidate` (line 297), which returns nothing when the openFDA label records disagree on the ingredient; its comment names "Pain Relief" at line 293.
  - `resolve_drug_base` (line 770), which then returns `original_not_found` (line 786): not found, exit 1.
  - The disease refusal at `src/entities/disease/resolution.rs:800` is the shape to copy: `InvalidArgument`, a "Candidates:" list, exit 2.
- Keeps: the 2031 and 2039 resolutions (TAGRISSO, Motrin, Opdualag).
- Decisions: when the label records whose brand name equals the query name two or more distinct single ingredients, return `InvalidArgument` with "Ambiguous drug name '<query>': several drug products carry this name.", then "Candidates:" and one `- <ingredient>` line per ingredient, sorted by record count and then by name, capped at 10, then "Try `biomcp get drug "<ingredient>"`". "Sleep Aid" and "Cold and Flu" get the same refusal.
- Changes claims: `src/entities/drug/get.rs` `src/entities/drug/get/tests.rs` `testdata/sources/openfda/**` `testdata/sources/capture-receipts.json` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`
- Proof: a recorded openFDA reply for "Pain Relief" gives exit 2 and the candidate list; the 2031 and 2039 tests still pass; one live run, recorded in the review.
- Defers: nothing.
- Order: safe alongside anything.
