# 1293 build record — article search bounded at 60 seconds with partial sources

Landed 2026-10-05, merge 6bf7ca552. Review verdicts and change detail
live on sdlc/tickets/1293-bound-article-search-time-and-report-partial-sources.md.

Outcome: federated article search stops at 60 seconds, keeps the rows
already fetched, and names the source that went silent. The bound did
not hold on every path — the single-backend path, post-deadline spin,
generic I/O error, and uncancellable construction under the epoch lock
were found later and fixed by ticket 1299 (landed 2026-10-07). The
CHANGELOG credits 1293 and 1299 together. Gates at landing: branch CI
green, yellow gates rc=0. Record backfilled 2026-10-07 under ticket
2020.
