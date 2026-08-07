---
flow: build
priority: 7
---
# Add gnomAD constraint metrics for variant interpretation

Constraint metrics (pLI, LOEUF, mis_z) are standard in variant interpretation. "Is this gene loss-of-function intolerant?" is one of the first questions a geneticist asks. BioMCP gets allele frequencies from gnomAD via MyVariant.info, but constraint metrics are gene-level data not available through MyVariant. Direct gnomAD GraphQL access fills this gap.

Completed under March on 2026-03-18, as March ticket 005. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/005-gnomad-constraint-metrics

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
