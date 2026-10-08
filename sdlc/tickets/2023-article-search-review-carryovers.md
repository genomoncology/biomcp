# 2023 — Article search review carryovers

Status: OPEN.

Milestone: 0.9.2

## Outcome

Article search paging, deadline errors, and the tests behind them say what is true.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 13).

1. 1298: the Europe PMC date and citation sort branch (`src/entities/article/search.rs:904`) has no offset-plus-limit check. `search article -k melanoma --source europepmc --sort date --offset 1300 --limit 5 -j` took 19 s and returned `count 0`, `has_more: true`. The relevance path refuses above 1250.
2. 1299: when both main sources fail, the federated path returns PubTator's error (`search.rs:535`). A fast non-retryable PubTator failure hides the deadline. Changing the fixture's 503 to 400 fails `deadline_expiry_without_rows_names_the_deadline_error`.
3. 1299: `construction_under_a_held_epoch_lock_honors_the_deadline` (`search/tests/deadline.rs:391`) fails under `cargo test --lib entities::article` because the shared client is built once per process. It passes only under nextest. The PubMed fixture route `ends_with("/esearch.fcgi")` (line 80) never matches because the target carries the query string; the "PubMed rows survive" assertions pass on a Semantic Scholar row.
4. 1298: the build record claims an absent-cursor stop at three requests. The fixture's third page is empty, so the empty-page check stops it. Removing both cursor stops leaves the spec output unchanged.
5. 1294: `push_word_separated` (`jats.rs:549`) splits words inside small caps, styled content, monospace and footnote markers ("T ABLE", "PD L1").

- Starts from: tickets 1294, 1298 and 1299.
- Keeps: the deadline translation, cursor walk, and whole abstracts.
- Changes: refuse offset plus limit above 1250 on every sort; prefer the deadline error when any source hit it; make the lock test independent of process state and fix the PubMed route; add a fixture whose last page has rows and no cursor; protect the inline elements that join word parts.
- Proof: each item has a test that fails on `37631c357` and passes under plain `cargo test` as well as nextest.
- Defers: nothing.

## Root cause

1. Only the relevance arms call the shared `limit.saturating_add(offset) >
   MAX_FEDERATED_FETCH_RESULTS` guard (`search.rs`). The non-relevance
   dispatch in `search_page_dispatch` passes raw `limit` and `offset` to
   `search_europepmc_page` and `search_pubtator_page`, so a date or
   citations sort with a deep offset walks up to fifty provider pages,
   discards every row against the offset, and reports `has_more: true`.
2. `collect_federated_article_rows` takes the PubTator leg's error whenever
   both primaries fail. A non-retryable PubTator failure settles before the
   deadline, so its plain error wins even when Europe PMC failed on the
   deadline. The 503 fixture only passes today because retries on a 503
   outlast the deadline and turn PubTator's own error into a deadline error.
3. The shared HTTP client latches in a process-global `OnceLock`, so
   `construction_under_a_held_epoch_lock_honors_the_deadline` only exercises
   construction when it runs first in its process: nextest gives each test a
   fresh process, plain `cargo test` does not. The same test's PubMed rows
   are also fictional: the fixture matches `ends_with("/esearch.fcgi")`
   against a target that always carries the query string, so the PubMed leg
   reads a 404 and the surviving-row assertions pass on a Semantic Scholar
   row.
4. The cursor fixture's exhausted third page carries no rows, so the
   empty-page stop ends the walk before the absent-cursor stop can matter;
   the absent-cursor stop in `backends.rs` has no proof that runs without
   the empty-page stop also firing.
5. `push_word_separated` inserts a boundary space at every unmarked
   element edge. Small caps, styled content, monospace and footnote
   markers wrap fragments of one word (`T<sc>able</sc>`, `PD<sc>L1</sc>`),
   so the renderer splits those words in two.

## Success criteria

1. `search_page` refuses `--offset + --limit` above 1250 with
   `InvalidArgument` on every sort and every backend plan, before any
   provider request.
2. When both primaries fail, the terminal error names the deadline whenever
   either primary's error is a deadline error, including when PubTator fails
   fast with a non-retryable status.
3. The held-epoch-lock test passes identically under plain `cargo test` and
   nextest regardless of test order, and the deadline fixture's PubMed route
   answers esearch and esummary so a PubMed row actually reaches the
   assertions that claim one.
4. A cursor fixture whose last page has rows and no `nextCursorMark` stops
   the walk on the absent cursor, and a test fails when both cursor stops
   are removed.
5. Words wrapped in small caps, styled content, monospace and footnote
   markers render unsplit, while the surname/given-names boundary space
   and the marker-attached runs keep their behavior.

## Build status

- Built on branch `tickets/2023-article-search-review-carryovers`,
  commits 5783f13d2 and a066bf6b9, 2026-10-07, across timeout
  revivals with checkpoints (nothing lost); stopped before spec runs
  under the Beelink disk directive, with the spec lane deferred to
  Yellow — recorded here, and the branch claims no spec run.
- Code review: ACCEPT 2026-10-07 (fresh reviewer; branch reviewed via
  a materialized read-only diff after the worktree's removal). All
  five items verified with recomputed arithmetic: the offset guard
  before any request on every sort; deadline precedence over a fast
  PubTator failure (both proof shapes); the process-independent lock
  test with a test-only reset seam and the PubMed route fix requiring
  both rows; the absent-cursor stop with the lying corpus's third page
  carrying rows (45 rows over 3 requests, siblings unchanged); JATS
  word-joining at both boundaries with the surname/given-names spacing
  kept. One report-only P2: an empty word-joining element glues
  neighboring words (degenerate markup; real elements carry text).
  Yellow confirms the spec expectation on the release-shaped binary
  before landing.

## Landing gates record

- Branch CI green at ec27dfbcf (run on the rebased tip).
- Yellow: lint green, spec green, test phase attempted under the
  interim deadlock rule (fourth futex hang captured; the issue
  sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md
  carries the forensics and the rule).
- Post-review delta ACCEPT (the rebase unions, the two coordinator
  commits, the repin) at ec27dfbcf; the cosmetic seam fold at
  9187de358. Merged-tree CI running at 05eeda3f3.
