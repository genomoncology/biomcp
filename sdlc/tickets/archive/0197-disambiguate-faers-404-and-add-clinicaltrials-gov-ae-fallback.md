---
flow: build
priority: 5
---
# Disambiguate FAERS 404 and add ClinicalTrials gov AE fallback

`biomcp drug adverse-events <name>` currently renders "No adverse events found" for both an OpenFDA 404 (drug not in FAERS at all) and an OpenFDA 200 with empty `results` (drug in FAERS but no matching events). These are different situations and deserve different messages. More importantly, the 404 case hides a capability BioMCP could provide: investigational or recently-approved drugs have no FAERS footprint by design — FAERS is a post-marketing database populated from MedWatch reports after approval — but their pre-approval adverse events are available on ClinicalTrials.gov in each study's `adverseEventsModule`. That is real, public, pre-approval AE data which is exactly what a user asking about a drug's safety wants.

Completed under March on 2026-04-16, as March ticket 197. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/197-disambiguate-faers-404-and-add-clinicaltrials-gov-ae-fallback
