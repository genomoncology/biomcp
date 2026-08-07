---
base: 81ff089a61e53c1db9545a4504ef0b64c2ca9fae
head: 03b9c7d8c79d6c702e05c59be45f3688856713d7
---
VAERS (Vaccine Adverse Event Reporting System) is the primary US source for vaccine safety signals. It is completely separate from FAERS (which BioMCP already queries via OpenFDA). BioMCP's existing adverse-event search returns some vaccine reports from FAERS (3,387 for influenza vaccine), but VAERS is the canonical vaccine-specific reporting system with far richer coverage.

Imported from March ticket 235. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/235-add-vaers-vaccine-adverse-event-source
