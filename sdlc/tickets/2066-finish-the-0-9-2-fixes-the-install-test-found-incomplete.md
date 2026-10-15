# 2066 — Finish the 0.9.2 fixes the install test found incomplete

Status: COMPLETE.

Milestone: 0.9.3

Reviews: revision a37d733, accept

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
  - `src/entities/disease/article_links.rs`: `crosswalk_hit_holds_name` accepts a match on the name, label or synonyms; `exact_named_crosswalk_hit` (line 71) requires exactly one holder.
  - Tests: `src/entities/disease/resolution/tests.rs`, `src/entities/disease/get/tests.rs`, `src/entities/trial/search/nci/tests.rs`, `src/cli/article/tests/entities_disease_links.rs`.
- Keeps: every recorded refusal and resolution 2032, 2040 and 2047 pinned.
- Decisions:
  - MM: the clinical reading's disease becomes the first line under "Candidates:", in the same `- label (ID)` form, followed by the other matches. The "Clinical reading" line stays.
  - Trial search: `nci.rs` rewrites the refusal's retry to `biomcp search trial -c "<reading label>" --source nci`.
  - Disease links (ruled 2026-10-15 after the recorded MESH:D012878 reply): MONDO:0002531 matched through its own MONDO label, "skin neoplasm". The card shows "skin benign neoplasm" because its title comes from the Disease Ontology name. A synonym rule cannot fix this, and a rule on the card title alone would drop the pinned "Neoplasms" link (MONDO:0005070, titled "disease of cellular proliferation"). Keep the current matching on the primary name, MONDO label and synonyms, and add one refusal: a hit does not hold the row's concept name when the card title it would show is a narrower form of that name. A title is narrower when it holds every word of the concept name, after the existing case, punctuation and final-word plural folding, plus at least one more word. "skin benign neoplasm" is narrower than "Skin Neoplasms", so the skin row finds no holder and keeps its text-search command. The pinned links for MONDO:0021117, MONDO:0004970, MONDO:0004972 and MONDO:0005070 keep linking.
  - The card title itself stays as it is. Titling disease cards from the MONDO label when the Disease Ontology name disagrees changes every card and is out of scope.
- Changes: claims `src/entities/disease/resolution.rs` `src/entities/disease/resolution/**` `src/entities/disease/get/tests.rs` `src/entities/disease/article_links.rs` `src/cli/article/tests/entities_disease_links.rs` `src/entities/trial/search/nci.rs` `src/entities/trial/search/nci/**` `testdata/sources/mydisease/**` `testdata/sources/capture-receipts.json` `spec/entity/disease.md` `CHANGELOG.md` `tests/test_docs_changelog_refresh.py`.
- Proof: recorded-reply tests for all three fixes, each failing on current main; the skin test uses the recorded MESH:D012878 reply; the 2047 test (MESH:D008175 and the other pinned descriptors) still passes unchanged; one live run of each command, recorded in the review.
- Defers: Europe PMC paging (2084); shared drug names (2085).
- Order: safe alongside anything; it shares no code file with 2065, 2070, 2071, 2084 or 2085.
