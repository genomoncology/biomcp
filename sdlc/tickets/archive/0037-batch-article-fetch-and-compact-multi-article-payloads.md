---
flow: build
priority: 8
---
# Add batch article fetch and compact multi-article payloads

Literature-first workflows still waste turns and latency when the agent has to open several PMIDs one at a time before choosing an anchor. BioMCP already has enough structure to collapse that work into a parallel fetch path and return a compact multi-article payload that helps the agent choose quickly without changing the typed-slot boundary.

Completed under March on 2026-03-20, as March ticket 037. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/037-batch-article-fetch-and-compact-multi-article-payloads

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
