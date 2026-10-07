# 1302 build record — citation evidence degrades honestly when Semantic Scholar refuses

Landed 2026-10-06, merge 52250af6f (branch tip 8e3c84498 including the
review P2 fold). Review verdicts, including the post-landing code
re-review of the fold, live on sdlc/tickets/1302-degrade-citation-evidence-honestly-when-semantic-scholar.md.

Outcome: a Semantic Scholar 429 or 5xx degrades to OpenCitations with
the provider named; a live 429 that took 22.12 seconds returns in
2.66; the fallback cannot fabricate an edge. Three non-blocking
honesty notes were folded by ticket 1306 (landed 2026-10-07). Gates:
branch CI green, yellow gates rc=0. Record backfilled 2026-10-07 under
ticket 2020.
