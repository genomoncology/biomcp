---
flow: build
priority: 5
---
# Add biomcp suggest question CLI verb for how-to discovery

The agent has no native way to look up "what's the right command sequence for this question?" The current path is `biomcp skill list` → read the table → match by hand → `biomcp skill <slug>`. Across 192 tasks in the 009 deep dive, **zero** invoked `biomcp skill list`. The discovery primitive is missing.

Completed under March on 2026-04-22, as March ticket 279. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/279-add-biomcp-suggest-question-cli-verb-for-how-to-discovery
