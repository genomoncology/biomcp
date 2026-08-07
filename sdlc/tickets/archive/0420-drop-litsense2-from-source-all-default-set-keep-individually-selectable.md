---
flow: quickfix
priority: 8
---
# Drop litsense2 from --source all default set (keep individually selectable)

Federated `biomcp search article --source all` averages ~18s (range 13-27s), but a 2026-06-14 deep dive showed nearly all of that residual is one bad source: **litsense2 burns its full 12s per-source cap on every call, sets the federated latency floor, and returns zero results on timeout** (`source_status: degraded, "LitSense2 timed out after 12s"`, observed 100% of calls; standalone it runs ~123s — genuinely broken upstream). The healthy sources (pubtator ~1.7s, europepmc ~1.9s, pubmed ~8.5s) finish concurrently underneath the cap — the fastest federated run was 12.9s, essentially *just* the litsense2 cap.

Completed under March on 2026-06-16, as March ticket 420. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/420-drop-litsense2-from-source-all-default-set-keep-individually-selectable
