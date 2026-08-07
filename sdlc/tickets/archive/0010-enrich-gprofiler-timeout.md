---
flow: build
priority: 8
---
# Investigate and fix g:Profiler enrich timeout

QA testing of v0.8.16 found that `biomcp enrich` consistently times out when calling g:Profiler, even with a small 3-gene input. The feature was working during earlier QA (Ian's testing returned results), so this may be a transient g:Profiler issue, a timeout configuration problem, or an endpoint change. The enrich command is part of the "gene set to knowledge graph" workflow and needs to be reliable.

Completed under March on 2026-03-18, as March ticket 010. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/010-enrich-gprofiler-timeout

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
