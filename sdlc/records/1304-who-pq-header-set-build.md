# 1304 build record — the WHO PQ export's real header set tracked

Landed 2026-10-06, merge 80d2759b8, including the reporter
trailing-space fold. Review verdicts, including the post-landing code
review that fills the missing-review gap, live on sdlc/tickets/1304-track-the-who-pq-exports-real-header-set.md. Closes external issue
#288 (reporter answered; the fix ships in the next release). Ticket
2010's line-cap split landed inside this branch as plain commits.

Outcome: the re-derived header sets (dropped alternative-listing
column, either applicant spelling, the vaccines trailing-space byte),
per-file sync outcomes with nonzero exit, and EMA-style degrade for
region-less search. Known gap carried by ticket 2021: JSON and MCP
callers cannot yet see the degrade reason. Gates: branch CI green,
yellow gates rc=0. Record backfilled 2026-10-07 under ticket 2020.
