---
flow: quickfix
priority: 9
---
# Close the routine lock descriptor in the two ClinGen CAR spec fixtures

Add the missing 8>&- redirection to all five leaking fixture spawns so an orphaned fixture server cannot hold the routine spec lock

Completed under March on 2026-07-26, as March ticket 626. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/626-close-the-routine-lock-descriptor-in-the-two-clingen-car-spec-fixtures
