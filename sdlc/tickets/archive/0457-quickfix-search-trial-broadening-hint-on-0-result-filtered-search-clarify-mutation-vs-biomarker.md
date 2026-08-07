---
flow: quickfix
priority: 5
---
# Quickfix: search trial — broadening hint on 0-result filtered search + clarify --mutation vs --biomarker

`biomcp search trial` silently returns 0 results for a reasonable, well-formed query and gives the agent no way to recover, producing a false "no trials exist" conclusion.

Completed under March on 2026-06-29, as March ticket 457. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/457-quickfix-search-trial-broadening-hint-on-0-result-filtered-search-clarify-mutation-vs-biomarker
