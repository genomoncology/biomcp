---
base: e8c760f9f2d390d16946c3f956763a32f460348f
head: 41633308eb92d6ed688a242f4d92b43541bb90cb
---
ClinicalTrials.gov registry eligibility can omit the thresholds needed to evaluate a patient. KarMMa-1 (`NCT03361748`) exposes only “Inadequate organ function,” while its posted protocol PDF contains the laboratory thresholds. CTGov API v2 already returns `documentSection.largeDocumentModule.largeDocs`, but BioMCP does not deserialize that section or expose its fixed-CDN files. BioMCP should expose source documents and say clearly when eligibility is registry text rather than pretend the registry card is complete.

Imported from March ticket 511. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/511-expose-posted-ctgov-trial-documents-and-eligibility-provenance
