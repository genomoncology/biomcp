---
base: af32527ce0afcdf7027dfa1adf3f20473686810e
head: 770b0ed6c17242085eed2047d8f3830ddbb27c61
---
`biomcp drug adverse-events <name>` currently renders "No adverse events found" for both an OpenFDA 404 (drug not in FAERS at all) and an OpenFDA 200 with empty `results` (drug in FAERS but no matching events). These are different situations and deserve different messages. More importantly, the 404 case hides a capability BioMCP could provide: investigational or recently-approved drugs have no FAERS footprint by design — FAERS is a post-marketing database populated from MedWatch reports after approval — but their pre-approval adverse events are available on ClinicalTrials.gov in each study's `adverseEventsModule`. That is real, public, pre-approval AE data which is exactly what a user asking about a drug's safety wants.

Imported from March ticket 197. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/197-disambiguate-faers-404-and-add-clinicaltrials-gov-ae-fallback
