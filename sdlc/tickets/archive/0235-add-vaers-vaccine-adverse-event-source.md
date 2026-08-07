---
flow: build
priority: 6
---
# Add VAERS vaccine adverse event source

VAERS (Vaccine Adverse Event Reporting System) is the primary US source for vaccine safety signals. It is completely separate from FAERS (which BioMCP already queries via OpenFDA). BioMCP's existing adverse-event search returns some vaccine reports from FAERS (3,387 for influenza vaccine), but VAERS is the canonical vaccine-specific reporting system with far richer coverage.

Completed under March on 2026-04-18, as March ticket 235. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/235-add-vaers-vaccine-adverse-event-source
