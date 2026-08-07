---
flow: review
priority: 8
---
# Audit source failures that collapse into domain facts

Several recent defects shared one dangerous failure mode: BioMCP converted incomplete retrieval into a confident biomedical statement. Truncated author data looked complete; a PubMed parse failure looked like unavailable article metadata without a cause; and an upstream asset-package 404 became “this article has no assets.” Agent callers usually stop searching after a well-formed negative, so a silent false negative can corrupt the conclusion more than a visible source error.

Completed under March on 2026-07-15, as March ticket 532. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/532-audit-source-failures-that-collapse-into-domain-facts

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
