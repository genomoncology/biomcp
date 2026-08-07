---
flow: build
priority: 6
---
# Add alias fallback to get commands via discovery layer

`biomcp get gene ERBB1` fails with "gene 'ERBB1' not found" even though `search gene ERBB1` correctly resolves to EGFR. Same for `get drug Keytruda` which returns almost no enrichment because downstream sources need "pembrolizumab." These are the #1 and #2 failure modes in benchmark testing.

Completed under March on 2026-03-20, as March ticket 029. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/029-get-alias-fallback

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
