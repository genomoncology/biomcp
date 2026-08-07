---
flow: build
priority: 5
---
# Fix drug interactions always empty

Drug interaction lookups always return empty because MyChem.info public API only serves drugbank_open, which excludes drug_interactions.

Completed under March on 2026-03-16, as March ticket 001. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/001-drug-interactions-empty

The landed commit range could not be recovered from git, so no
record accompanies this entry. The work products above are the
evidence that survives; the absence of a record is a gap in what
git can still prove, not a sign the work is missing.
