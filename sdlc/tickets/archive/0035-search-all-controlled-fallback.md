---
flow: build
priority: 6
---
# Constrain search-all cross-routing to controlled typed fallback

`search all` is valuable as a typed, counts-first orientation card, but naive cross-routing can add noisy drill-downs and duplicated terms without improving the main answer path. The cross-routing logic should stay subordinate to the agent's typed slots, run only when ambiguity justifies it, and produce sanitized follow-up commands.

Completed under March on 2026-03-20, as March ticket 035. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/035-search-all-controlled-fallback

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
