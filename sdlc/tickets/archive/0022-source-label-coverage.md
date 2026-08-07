---
flow: build
priority: 8
---
# Improve source labeling in entity detail outputs

The paper's traceability audit found that only 78% of sampled claims carry explicit source labels in BioMCP output. Disease (4/6), drug (3/6), and gene (3/6) entity types are the worst. For a paper claiming "source-linked" outputs, this rate needs to be above 90%. Every section of a `get` response should identify which upstream source supplied the data.

Completed under March on 2026-03-19, as March ticket 022. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/022-source-label-coverage

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
