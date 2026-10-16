# 2130 — Keep the PubTator variant entity an rsID search finds

Status: OPEN.

Milestone: 0.9.3

## Outcome

An article search by rsID keeps the PubTator variant entity whose `db_id` carries that rsID and searches by it, so `search article` for `rs113488022` returns PubTator's annotated papers as well as the text matches.

## Evidence

Found 2026-10-16 from a literature-coverage test of an older install, and confirmed on main.

- PubTator3 autocomplete for `rs113488022` returns one variant entity: id `@VARIANT_p.V600E_BRAF_human`, `db_id` `rs113488022##`, name `p.M600E`. Its text search for the rsID returns 47 papers; searching the entity id returns 27,919.
- `autocomplete_source_identity` (`src/entities/article/query.rs:356-360`) builds the source identity from `name` and `id` only; the parsed `db_id` (`src/sources/pubtator.rs:432`) is never read. Neither field holds the rsID, so the comparison is undecided (`src/entities/variant/resolution.rs:1380-1382`) and only a full match is kept (`query.rs:466-472`). The annotation route returns nothing for an rsID-only query; the plain-text route still runs (`variant_search.rs:1797`).

- Starts from: the lines above.
- Decisions:
  - Read the rsID from `db_id` (the part before `##`) into the source identity, so an rsID request matches on it.
  - The entity's own name stays out of the match when the rsID matched; PubTator's `p.M600E` label does not change what was asked for.
- Changes: claims `src/entities/article/query.rs` `src/entities/article/query/**` `src/sources/pubtator.rs` `src/sources/pubtator/**` `testdata/sources/pubtator/**` `testdata/sources/capture-receipts.json` `CHANGELOG.md`.
- Proof: a recorded-reply test of the `rs113488022` autocomplete reply keeps the entity and searches by its id, failing on current main; the existing gene-plus-change entity tests pass unchanged.
- Order: before 2131, which adds a second variant-literature source beside this path.
