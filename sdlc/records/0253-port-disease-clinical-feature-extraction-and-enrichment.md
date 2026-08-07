---
base: 2c5d83b12f1e9d14ff22bcdffcea3de3cf050695
head: d63151087a1c4accc288eee80aa393ddece7499e
---
Survey issues 4 and 5 block useful runtime behavior after the foundation exists: the Rust disease module still needs the spike's MedlinePlus disease configuration, multi-query topic loading, URL deduplication, direct-page topic selection, expected-symptom extraction, reviewed HPO mapping, and offline fixture fallback. Survey issues 2 and 3 are only fully addressed when the new `clinical_features` field is populated through the requested disease section.

Imported from March ticket 253. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/253-port-disease-clinical-feature-extraction-and-enrichment
