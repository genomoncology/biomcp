---
flow: build
priority: 8
---
# Add missing evidence URLs to entity detail outputs

The paper traceability audit found 6 claims where BioMCP returns data but attaches no evidence URL for the user to verify it. Every datum in a `get` response should have a clickable link back to its source. These are small gaps concentrated in a few entity types.

Completed under March on 2026-03-19, as March ticket 025. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/025-evidence-url-gaps

The landed commit range could not be recovered from git, so no
record accompanies this entry. That is a known gap for the
earliest tickets, not a sign the work is missing.
