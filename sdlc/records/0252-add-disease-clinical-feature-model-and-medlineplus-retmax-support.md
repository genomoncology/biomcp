---
base: 41fde8d336675ab580634331a74c15cef30904f4
head: cad385e7713829c3774b653bec5b1445e90789de
---
Survey issue 1 blocks the port because `src/sources/medlineplus.rs` hardcodes `retmax=3`, while the validated clinical-features spike needs `retmax=5`. Survey issues 2, 3, and 4 also need a safe foundation before algorithm work: the disease model has no `clinical_features` field, the disease section parser has no `clinical_features` section, and there is no Rust disease config fixture for MedlinePlus source queries and expected symptom patterns.

Imported from March ticket 252. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/252-add-disease-clinical-feature-model-and-medlineplus-retmax-support
