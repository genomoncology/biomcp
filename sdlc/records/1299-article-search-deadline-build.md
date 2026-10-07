# 1299 build record — the article search deadline is honest

Landed 2026-10-07, merge 37631c357 (branch tip 14ed1c62e including the
review fold). Review verdicts, including the post-landing code
re-review, live on sdlc/tickets/1299-make-the-article-search-deadline-honest.md.

Outcome: a deadline that hits mid-pagination keeps fetched rows and
names the silent source; the deadline error names itself; construction
honors the deadline including under a held cache epoch lock; the
discover keyword lookup sharing the defect is bounded. Gates: branch
CI green, yellow gates rc=0 at 14ed1c62e. Merge stripped three spec
fences; ticket 2020's lane restored them (2026-10-07, adf71898e) and
the three cases run again. Record backfilled 2026-10-07 under ticket
2020.
