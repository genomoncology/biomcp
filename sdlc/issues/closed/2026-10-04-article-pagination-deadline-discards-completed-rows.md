# page-2 deadline expiry discards already-fetched rows in plain article pagination

Status: closed.

Resolution: ticket 1299 (make the article search deadline honest end to end), changes 1 to 3; reproduced in experiment 439.

Filed from source review at the named revision. No runtime reproduction was claimed at filing; reproduced 2026-10-05 in experiment 439.

Donor revision: `d8c6ce8ba495639b8a798e69e58d379d3762b92a` (main, 2026-10-04, "Clear the lane table: tickets 1290 and 1293 landed and their worktrees are gone").

Priority: P2.

## Symptom

A plain single-backend article search (`--source pubtator`, `--source europepmc`, `--source pubmed`, or a plan that resolves to one of them) assembles a requested results page by fetching sequential provider pages. When the per-invocation 60-second deadline expires during a later provider page fetch, the whole invocation fails and the rows already materialized from completed provider pages are discarded. A consumer paging through results (`--offset` past the first page, or a deep query needing several provider batches) loses rows it already paid for and receives a hard deadline error instead of a partial page. The federated all-sources path already keeps the rows that answered and names the held source degraded; the plain single-backend paths do not.

## Source paths (at the donor revision, relative to the repository root)

Deadline scope and terminal behavior:

- `src/entities/article/search.rs` — `search_page` wraps the dispatch in one 60-second `VariantArticleDeadline` when no caller owns one, and deadline-class errors become `article_search_deadline_error` (a hard, retryable invocation failure). The comment records that a caller already scoping a deadline (the variant-article path) keeps it.
- `src/entities/article/search/deadline.rs` — `ARTICLE_SEARCH_DEADLINE` is 60 seconds.

Single-backend dispatch discards on deadline error:

- `src/entities/article/search.rs` — the `EuropeOnly` and `PubTatorOnly` arms do `let page = result?;`: a deadline error from the backend loop discards everything, with no partial page and no degraded source status.  The federated path, by contrast, already returns the rows that answered with sources that outlasted the deadline reported degraded (the retention ticket 1293 landed).

Pagination loops that accumulate and then discard:

- `src/entities/article/backends.rs` — `search_pubmed_page_with_context` (batch loop from the start of the result set, skipping to the offset), `search_europepmc_page_with_context` (provider page loop), `search_pubtator_page_with_context` (provider page loop), each accumulate visible rows in `out` across provider page fetches and propagate any error from `variant_article_request(...).await?` out of the loop, dropping the accumulated rows. On the plain path `variant_article_request` is a plain `future.await`, so the deadline error from a stalled later page fetch discards the rows committed by earlier completed fetches.

## Invariant

When the invocation deadline expires during a plain single-backend article search, rows already materialized from completed provider page fetches must be returned with the affected source's status degraded and the deadline named. A search in which no row was materialized keeps failing as the retryable deadline failure it is today. Ordinary source errors (non-deadline) and the variant-article path, where the caller explicitly owns the deadline, are unchanged.

## Evidence (source reading only)

The code paths above are quoted from the donor revision. The single-backend arms' `result?` and the backend page loops' error propagation are visible in source; the federated path's retain-and-degrade mechanism at the same revision shows the intended shape already exists in this file family. No runtime reproduction was performed by this job.

## Bounded remediation (proposed)

In the plain single-backend paths (and the single-backend dispatch arms), catch deadline-class errors (deadline exhausted plus the search-deadline error discriminator) after at least one provider page has committed visible rows, and return the materialized rows as a partial page with the source status degraded, mirroring the federated path's mechanism. With zero materialized rows, keep the current retryable deadline failure. Ordinary errors and the variant-article path keep their current behavior. No provider protocol, ranking, or pagination-cursor change.

## Finite future offline proof (to be written by the fixing ticket; not run here)

1. Serve a completed first provider page and signal that the second request is held. Explicitly advance a controlled invocation clock. Assert retained row identity, totals and degraded source status for PubTator, Europe PMC and PubMed. No sleep or elapsed-time threshold.
2. Hold the first request under the same controlled clock; assert retryable deadline failure with zero rows. Add a successful empty response control.
3. Return an ordinary second-page error; assert ordinary error propagation remains unchanged.
4. Retain existing explicit variant ownership and settlement checks.

## Relationship to existing records

Ticket 1293 ("bound article search time and report partial sources", complete, landed `6bf7ca55`) introduced the invocation deadline and retention on the federated legs; its deferral list names only NCBI rate-limit handling, and the single-backend retention gap is not recorded there or in any open `sdlc/issues/` file. The resolved issue on source failures rendering as empty or complete results (closed by ticket 1242) is the honesty theme behind this gap, not the same defect.

Root source verification: Semantic Scholar candidates use one provider response here, so this draft makes no Semantic Scholar page-2 claim.

## Reproduced 2026-10-05

Experiment 439 (`~/workspace/experiments/439-reproduce-1293-p2-and-1291-switch/`), binary at main a877443f, live sources. `search article --source europepmc "glioblastoma treatment resistance" --limit 50 -j` with `BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS=6000`: 3 provider pages fetched, then a hard failure with **0 rows** — the rows already fetched were discarded. Confirmed twice, once through a logging proxy and once direct (proxy ruled out). Two behaviors beyond the filing: the invocation ran 37–42 s wall against the 6 s budget (25–35 s of CPU/IO spin after expiry, no network calls, root cause not yet isolated), and the error surfaced as generic `io / I/O operation failed` rather than the intended `article_search_deadline_error`. A federated control under the same deadline also failed whole: all four sources "internal failure", generic io error, 0 rows — the partial-page retention from QA 0003 appears when sources are slow but successful, not when the deadline fires mid-flight. Root cause in code confirmed at `src/entities/article/search.rs:706, 725, 744` (`let page = result?;`).
