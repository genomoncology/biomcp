---
flow: build
priority: 8
---
# Add disease clinical feature model and MedlinePlus retmax support

Survey issue 1 blocks the port because `src/sources/medlineplus.rs` hardcodes `retmax=3`, while the validated clinical-features spike needs `retmax=5`. Survey issues 2, 3, and 4 also need a safe foundation before algorithm work: the disease model has no `clinical_features` field, the disease section parser has no `clinical_features` section, and there is no Rust disease config fixture for MedlinePlus source queries and expected symptom patterns.

Completed under March on 2026-04-19, as March ticket 252. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/252-add-disease-clinical-feature-model-and-medlineplus-retmax-support
