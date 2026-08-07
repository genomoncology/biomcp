---
flow: build
priority: 10
---
# Pathway section truthfulness and guidance

Post-expansion review found that KEGG pathway section requests can exit successfully with blank or near-blank human output. `events` remains Reactome-only in docs, but runtime validation, renderer behavior, and suggested next commands still allow KEGG flows that feel broken. This breaks the truthful-degradation contract and undermines user trust in the expanded pathway surface.

Completed under March on 2026-03-19, as March ticket 016. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/016-pathway-section-truthfulness

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
