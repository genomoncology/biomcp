---
flow: build
priority: 9
---
# Improve typed variant alias resolution and recovery guidance

Variant workflows break down when the agent supplies a common shorthand that is clearly intended as a variant but does not already match the canonical forms BioMCP handles well. The variant path should normalize typed variant inputs or return variant-scoped recovery guidance, rather than deferring to generic discovery behavior that can misroute the query.

Completed under March on 2026-03-20, as March ticket 033. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/033-typed-variant-alias-resolution

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
