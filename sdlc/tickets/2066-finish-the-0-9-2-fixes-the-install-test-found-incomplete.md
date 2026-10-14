# 2066 — Finish the 0.9.2 fixes the install test found incomplete

Status: OPEN.

Milestone: 0.9.3

## Outcome

`get disease MM` lists multiple myeloma as its first candidate, `search trial -c MM --source nci` refuses with a trial-search retry, and `article entities 22663011` no longer links the "skin neoplasms" row to the skin benign neoplasm card.

## Evidence

Filed 2026-10-13 from the post-release install and product test of v0.9.2 (tag `4049666f5`), run on Linux x86_64 and macOS arm64 against every install channel. Rewritten 2026-10-14: the Europe PMC paging part moved to 2084 and the shared drug name part to 2085, so the three can be built apart.

- `biomcp get disease MM` lists one candidate, "Miyoshi muscular dystrophy (MONDO:0009685)", and names multiple myeloma only on a later "Clinical reading" line. `search trial -c MM --source nci` prints the same refusal, telling the user to retry `biomcp get disease` rather than how to fix the trial search.
- `biomcp article entities 22663011` prints `biomcp get disease MONDO:0002531` for the row "skin neoplasms" (MeSH D012878), and that card is "skin benign neoplasm". 2047 fixed the same shape for Lung Cancers only.

- Starts from:
  - `src/entities/disease/resolution.rs:797-805`, which builds the refusal.
  - `src/entities/disease/resolution/ambiguity.rs`: the `CLINICAL_ABBREVIATION_READINGS` table and `clinical_reading_line` (line 91), whose retry is a `get disease` command.
  - `src/entities/trial/search/nci.rs:65-71`, which passes the disease refusal through unchanged.
  - `src/entities/disease/article_links.rs`: `crosswalk_hit_holds_name` accepts a match on synonyms; `exact_named_crosswalk_hit` (line 71) requires exactly one holder.
  - Tests: `src/entities/disease/resolution/tests.rs`, `src/entities/disease/get/tests.rs`, `src/entities/trial/search/nci/tests.rs`, `src/cli/article/tests/entities_disease_links.rs`.
- Keeps: every recorded refusal and resolution 2032, 2040 and 2047 pinned.
- Decisions:
  - MM: the clinical reading's disease becomes the first line under "Candidates:", in the same `- label (ID)` form, followed by the other matches. The "Clinical reading" line stays.
  - Trial search: `nci.rs` rewrites the refusal's retry to `biomcp search trial -c "<reading label>" --source nci`.
  - Disease links: a disease record matches a row's concept name only through its primary name or MONDO label, not through synonyms. A row with no match keeps its text-search command.
  - Before coding the link rule, record the MyDisease reply for MESH:D012878 and confirm the benign card matched through a synonym. If it matched through its label, stop and report to the coordinator.
- Changes claims: `src/entities/disease/resolution.rs` `src/entities/disease/resolution/**` `src/entities/disease/get/tests.rs` `src/entities/disease/article_links.rs` `src/cli/article/tests/entities_disease_links.rs` `src/entities/trial/search/nci.rs` `src/entities/trial/search/nci/**` `testdata/sources/mydisease/**` `testdata/sources/capture-receipts.json` `spec/entity/disease.md` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`
- Proof: recorded-reply tests for all three fixes, each failing on current main; the 2047 Lung Cancers test (MESH:D008175) still passes; one live run of each command, recorded in the review.
- Defers: Europe PMC paging (2084); shared drug names (2085).
- Order: safe alongside anything; it shares no code file with 2065, 2070, 2071, 2084 or 2085.
