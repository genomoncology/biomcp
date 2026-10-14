# 2085 — Refuse a drug name several products share

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 5101919, accept

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
- Changes: claims `src/entities/drug/get.rs` `src/entities/drug/get/tests.rs` `testdata/sources/openfda/**` `testdata/sources/capture-receipts.json` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`.
- Proof: a recorded openFDA reply for "Pain Relief" gives exit 2 and the candidate list; the 2031 and 2039 tests still pass; one live run, recorded in the review.
- Defers: nothing.
- Order: safe alongside anything.

- Landed on `ticket/2085-shared-drug-names` (head `3049d8a5f`), 2026-10-14. `openfda_label_identity_candidate` now returns a three-way answer; its disagreement state becomes `ambiguous_drug_name_error` (`InvalidArgument`, exit 2) at the name-miss site, while the combination-card and sparse-card sites keep the card they hold. The candidates are the distinct single ingredients the counted records carry — the owner records when any carry the exact brand, else the voting page records — deduped as ingredient word sets, spelled as most records spell them, counted by records, sorted by record count then name, capped at 10, then `Try `biomcp get drug "<ingredient>"``. A record that lists no structural ingredients contributes its own vote line when that line names one ingredient, so the 2031-era trimmed fixtures still list theirs.
- Recorded reply: `testdata/sources/openfda/label_pain_relief_20261014.json` (live total 1249, minimized to identity fields) with its receipt in `capture-receipts.json`; the coupling inventory repins that file's digest (`tools/zero-coupling-historical.json`).
- Proof (recorded): `get_drug_pain_relief_refuses_as_ambiguous_with_the_recorded_candidates` replays the recorded reply through the fixture server and asserts exit 2 plus the exact refusal (`- lidocaine`, `- menthol`, `- acetaminophen`, `- hexylresorcinol`); three unit tests pin the ranking, the cap, and the owner-records rule. The `brand_resolution` Pain Relief and Sleep Aid tests, which pinned the old not-found refusal, now pin the ambiguous refusal.
- Proof (keeps): TAGRISSO → osimertinib, Motrin → ibuprofen, Opdualag keeps its combination card — all targeted nextest runs green; the whole `entities::drug::test_support` + `drug::get::tests` lane (79) and the transform/search/label neighborhood (86) pass.
- Proof (red on main): probe branch `probe/2085-red` (main plus the recorded-reply test alone) failed with `drug 'Pain Relief' not found. No drug card matches "Pain Relief" …` — the exit-1 path; branch deleted after the run.
- Proof (live, Yellow, 2026-10-14, built debug binary of `46174ae18`): `get drug "Pain Relief"` → exit 2, candidates lidocaine, menthol, acetaminophen, hexylresorcinol; `get drug "Sleep Aid"` → exit 2, diphenhydramine hydrochloride, doxylamine succinate; `get drug "Cold and Flu"` → exit 2, acetaminophen, dextromethorphan hydrobromide, phenylephrine hydrochloride, chlorpheniramine maleate, doxylamine succinate, triprolidine hydrochloride; `get drug Motrin` → the ibuprofen card and `get drug TAGRISSO` → the osimertinib card still resolve.
- CI on `3049d8a5f` (run 38564870347): every job green except `canonical-gates`, which fails only on the rustls "Unauthenticated ServerHello" advisory — the same job and advisory main itself fails on at `46087a6b6` (ticket 2093 owns the bump), so the branch adds no new failure.
- Beyond the claimed files: `src/entities/drug/test_support.rs` (two fixture helpers made `pub(super)` so the claimed test file can use them), `src/entities/drug/test_support/brand_resolution.rs` (the two superseded refusal tests), `tools/zero-coupling-historical.json` (digest repin).
- Follow-up after the lander's rebase onto green main: `make lint` failed silently at the quality ratchet because the `rust_source_size` audit pins `src/entities/drug/get.rs` at 1428 lines and the refusal grew it to 1577. Re-stamped the entry through `tools/update-rust-source-size-inventory --authorize src/entities/drug/get.rs --ticket 2085` (floor stays 1273, delta 304, with the reason and removal condition recorded). `tools/check-quality-ratchet.sh` exits 0 locally; the ratchet, zero-coupling, receipts, and test-wait checks all pass.
