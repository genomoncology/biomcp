---
base: 241d02615af04788a7997c61d2978414222bc246
head: 1d332fa060254e44020c5e03cbf465e3189ecad1
---
BioMCP presents a general rare-disease trial-planning and action-summary capability, but the production planner is a curated Phelan-McDermid/SHANK3/22q13 seed and the classifier recognizes only one trial type and one caveat through substring checks. A live Rett syndrome action-summary returned a valid row but omitted the documented `trial_type` and `access_caveats` fields. This is the same output-honesty failure class as the retired `biomcp suggest` router and the removed three-disease `clinical_features` answer key.

Imported from March ticket 494. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/494-rip-out-rare-disease-trial-planner-condition-expansion-and-action-summary-surface
