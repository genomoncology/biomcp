---
flow: build
priority: 6
---
# Normalize search drug JSON envelope across --region modes (UX-4)

`biomcp search drug --region all` (default) returns a nested envelope (`us{count, results}, eu{count, results}, who{...}`) while `biomcp search drug --region eu` returns a flat `{pagination, count, results, _meta}` envelope. Scripts and agents navigating `search drug --json` must handle two structurally different shapes. Tracked as UX-4 since v0.8.20; not a regression, but a usability wart worth closing before v0.9.

Completed under March on 2026-04-16, as March ticket 223. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/223-normalize-search-drug-json-envelope-across-region-modes-ux-4
