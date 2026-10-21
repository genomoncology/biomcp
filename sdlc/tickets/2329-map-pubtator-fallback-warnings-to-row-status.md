# 2329 — Tie PubTator3 fallback warnings to row status

Status: OPEN.

Milestone: 0.9.4

## Outcome

Changes: `CHANGELOG.md`, `src/entities/article/enrichment.rs`, `src/entities/article/mod.rs`, `src/entities/article/variant_search.rs`, `src/cli/variant/articles.rs`, `src/render/markdown/article.rs`, `templates/article_search.md.j2`, `tests/test_variant_article_metadata_status.py`, `docs/user-guide/cli-reference.md`, `docs/user-guide/variant.md`.

When variant article search falls back because PubTator3 visible metadata is unavailable, each affected result row carries an in-band status naming what is missing, and the stderr warning names the same class instead of "unavailable". A reader of the JSON or Markdown alone can tell which rows are incomplete and why.

- Proof: an outside-in replay test with PubTator3 metadata failing for some rows shows a matching in-band status on exactly those rows, a warning that names the class, and the overall incomplete flag set; a test with no failure shows no warning and no row status.

## Evidence

- PM ruling m3004 on 2026-10-21 approves `metadata_status` with `status: incomplete`, `missing_fields`, `source: pubtator`, and the existing specific `failure_class`. Class means failure cause; missing fields name absent metadata. The warning names the same fields and cause. Fully recovered rows stay unmarked.
- Starting revision: d3b261106c3e2d7ce4eaabe35eb21091f26b2700. Gator job j1506 replayed two exact-name results with the existing captured PubTator HTTP 400 and Europe PMC absent bodies, plus an authored successful recovery for the second row. JSON says `complete:false` and `truncated:true`; PMID 42894377 lacks abstract, citation count and row status, while PMID 42891779 recovers both fields. Markdown shows both without a row status. Each stderr has one visible-metadata fallback warning with existing `class=internal`. This confirms the row-status defect; it does not claim the historical unavailable-class cause.
- Before product edits, job j1519 ran `pm job run --machine gator 2329 d3b261106c3e2d7ce4eaabe35eb21091f26b2700 --disk 8 --memory 7 -- bash -c 'set -e; printf "%s" "$1" | base64 -d | git apply; make prepare-test-contracts; BIOMCP_BIN="$PWD/target/debug/biomcp" .venv/bin/python -m pytest tests/test_variant_article_metadata_status.py -q' red PAYLOAD`. The test-only patch SHA256 is fcdec587e8b19a6db015382ca4e46b608160d78ee46d9310444697490a584d82. The uncovered-row case failed because `metadata_status` is absent; covered and healthy controls passed. Job j1499 stopped at a probe import error before replay and supplies no behavior evidence.
- Implementation remains within visible variant metadata: retain row identity, source counts, provider failure classification and overall completeness rules. Return the per-row failure from enrichment, carry it through full and compact variant responses, and display it on the matching Markdown row. Add no shared state or provider requests.

- On 2026-10-21, the research team's reviewed recheck of 0.9.4 (commit cdb9f4a1b) found that variant article searches still emit PubTator3 visible-metadata fallback warnings with the class "unavailable". The overall result says incomplete or truncated, but nobody has shown that every failing row maps to an in-band status. Research filed it as a concern under R3 (2269), not as a verified defect.
- 2269 fixed the original glioblastoma search, which research could not reproduce, and section quality.
- Start by reproducing and confirming the concern. If every failing row already carries a status, close this ticket with that evidence.
