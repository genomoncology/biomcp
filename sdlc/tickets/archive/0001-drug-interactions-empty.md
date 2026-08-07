---
flow: build
priority: 5
---
# Fix drug interactions always empty

Drug interaction lookups always return empty because MyChem.info public API only serves drugbank_open, which excludes drug_interactions.

Completed under March on 2026-03-16, as March ticket 001. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/001-drug-interactions-empty

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
