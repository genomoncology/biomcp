# 2130 — Keep the PubTator variant entity an rsID search finds

Status: OPEN.

Milestone: 0.9.3

## Outcome

An article search by rsID keeps the PubTator variant entity whose `db_id` carries that rsID and searches by it, so `search article` for `rs113488022` returns PubTator's annotated papers as well as the text matches.

## Evidence

Found 2026-10-16 from a literature-coverage test of an older install, and confirmed on main.

- PubTator3 autocomplete for `rs113488022` returns one variant entity: id `@VARIANT_p.V600E_BRAF_human`, `db_id` `rs113488022##`, name `p.M600E`. Its text search for the rsID returns 47 papers; searching the entity id returns 27,919.
- `autocomplete_source_identity` (`src/entities/article/query.rs:356-360`) builds the source identity from `name` and `id` only; the parsed `db_id` (`src/sources/pubtator.rs:432`) is never read. Neither field holds the rsID, so the comparison is undecided (`src/entities/variant/resolution.rs:1380-1382`) and only a full match is kept (`query.rs:466-472`). The annotation route returns nothing for an rsID-only query; the plain-text route still runs (`variant_search.rs:1797`).

- Also reported 2026-10-16 by a literature probe of 0.9.2: `biomcp --json variant articles rs372266620 --limit 50` returns resolution `ambiguous`, no results, and error `source_unavailable` ("all required variant article routes were unavailable") while every route reports ok. LitVar2 lists 39 papers. `--confirmed-only` on "SAMHD1 I448T" gives the same false error. The fix reports an empty search where every source answered as an ok result with exit 0. `source_unavailable` and exit 1 stay for a zero-row result where a provider source reported unavailable or timed out, as `docs/how-to/find-articles.md:109-113` and `docs/user-guide/cli-reference.md:863-869` state. Today `hard_error = rows.is_empty() && !complete` (`src/entities/article/variant_search.rs:3473`), and ambiguous resolution, indeterminate provider validation, unfinished identity verification or unfinished equivalence checks all clear `complete`. Report those as their own non-outage status with `complete: false` and no error. The error message names the failing sources instead of saying "all required variant article routes were unavailable".
- ABCC8 paper 16429405, returned by 0.8.25, is missing from every 0.9.2 query. Find why while fixing the rsID route; file a separate ticket if the cause lies elsewhere.

- Starts from: the lines above.
- Decisions:
  - Read the rsID from `db_id` (the part before `##`) into the source identity, so an rsID request matches on it.
  - The entity's own name stays out of the match when the rsID matched; PubTator's `p.M600E` label does not change what was asked for.
- Changes: claims `src/entities/article/query.rs` `src/entities/article/query/**` `src/entities/article/variant_search.rs` `src/sources/pubtator.rs` `src/sources/pubtator/**` `testdata/sources/pubtator/**` `testdata/sources/capture-receipts.json` `CHANGELOG.md`.
- Proof: a recorded-reply test of the `rs113488022` autocomplete reply keeps the entity and searches by its id, failing on current main; the existing gene-plus-change entity tests pass unchanged.
- Order: before 2131, which adds a second variant-literature source beside this path.
