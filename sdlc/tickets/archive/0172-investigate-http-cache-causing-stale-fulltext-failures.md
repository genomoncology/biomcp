---
flow: quickfix
priority: 6
---
# Investigate HTTP cache causing stale fulltext failures

Even after fixing the fulltext sources, stale cached failures could mask the fix. Users would need to manually clear their cache to see improvements. Understanding and fixing the cache behavior ensures the efetch fix (ticket 171) takes effect immediately.

Completed under March on 2026-04-10, as March ticket 172. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/172-investigate-http-cache-causing-stale-fulltext-failures

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
