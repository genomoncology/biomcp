---
flow: build
priority: 5
---
# Expose posted CTGov trial documents and eligibility provenance

ClinicalTrials.gov registry eligibility can omit the thresholds needed to evaluate a patient. KarMMa-1 (`NCT03361748`) exposes only “Inadequate organ function,” while its posted protocol PDF contains the laboratory thresholds. CTGov API v2 already returns `documentSection.largeDocumentModule.largeDocs`, but BioMCP does not deserialize that section or expose its fixed-CDN files. BioMCP should expose source documents and say clearly when eligibility is registry text rather than pretend the registry card is complete.

Completed under March on 2026-07-13, as March ticket 511. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/511-expose-posted-ctgov-trial-documents-and-eligibility-provenance
