# 1299 — make the article search deadline honest end to end

Filed 2026-10-05 by the BioMCP 0.9 lead, from experiment 439; replaces two P2 issues (article client construction outliving the invocation deadline; page-2 deadline expiry discarding completed rows).

Status: OPEN.
Milestone: 0.9.2.

## Outcome

An article search that hits its deadline returns the rows that already answered, from every backend plan, exits near the deadline, and names the deadline in its error. Holding the cache epoch lock cannot push an invocation past its deadline.

## Evidence

- Starts from: Experiment 439 (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`), binary at main a877443f, live sources, forced via `BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS`. Three defects reproduced.
- Keeps: The 60-second default budget, the federated partial-page behavior for slow-but-successful sources, and `SourceUnavailable` when nothing answered.
- Changes: (1) Single-backend pagination discards rows: `search article --source europepmc ... --limit 50` under a 6 s deadline fetched 3 pages, then failed with 0 rows — the fetched rows were discarded at `let page = result?;` (`src/entities/article/search.rs:706, 725, 744`). Single-backend plans must return the accumulated rows with the held source named degraded, as the federated path already does. (2) The invocation outlives the deadline: the same run took 37–42 s wall against the 6 s budget, with 25–35 s of CPU/IO spin after expiry and no network calls; a federated control failed whole the same way (all four sources "internal failure", 0 rows). Isolate the spin (suspect: retry or cache writes under cancellation) and make expiry prompt. (3) The error is wrong: expiry surfaced as generic `io / I/O operation failed`, not the intended `article_search_deadline_error`. (4) Construction cannot be cancelled: with the cache epoch lock held (`~/.cache/biomcp/.body-limit-cache-v1.lock`) and a 3 s deadline, a PubMed-only search produced no output for 90 s — the plain branch's synchronous constructors (`src/entities/article/backends.rs:244, 410, 574, 673`; `src/entities/article/enrichment.rs:178, 334, 345`) reach `lock.lock_exclusive()` (`src/cache/migration.rs:130-132`), a blocking lock with no await point. The deadline-aware `ensure_body_limited_cache_epoch_until` (`migration.rs:150-176`) already exists; use it on every path.
- Proof: Spec cases with recorded fixtures for each: single-backend expiry returns partial rows with the source degraded; expiry names `article_search_deadline_error`; construction under a held epoch lock honors the deadline. A measurement that a forced-deadline invocation exits within the budget plus a small grace.
- Defers: The plain path's high floor (~35 s wall for a 3-row PubMed search, 10 s user CPU, lock-free) is recorded in experiment 439; fixing the floor is its own future ticket if it survives 1298.
