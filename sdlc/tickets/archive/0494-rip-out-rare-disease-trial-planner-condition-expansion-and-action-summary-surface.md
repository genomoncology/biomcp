---
flow: build
priority: 5
---
# Rip out rare-disease trial planner, condition expansion, and action-summary surface

BioMCP presents a general rare-disease trial-planning and action-summary capability, but the production planner is a curated Phelan-McDermid/SHANK3/22q13 seed and the classifier recognizes only one trial type and one caveat through substring checks. A live Rett syndrome action-summary returned a valid row but omitted the documented `trial_type` and `access_caveats` fields. This is the same output-honesty failure class as the retired `biomcp suggest` router and the removed three-disease `clinical_features` answer key.

Completed under March on 2026-07-10, as March ticket 494. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/494-rip-out-rare-disease-trial-planner-condition-expansion-and-action-summary-surface
