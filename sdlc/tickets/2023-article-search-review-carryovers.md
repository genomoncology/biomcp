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
