---
base: 2658c1d8ee93deaafc4e86f46a2619a52f069e3b
head: 9cde031c45cdfe5dee3f4c12f49766b5e3ac5934
---
BioMCP covers genes, drugs, trials, and literature but cannot answer "what is the 5-year survival rate for CML?" — the most fundamental cancer outcome metric. SEER (Surveillance, Epidemiology, and End Results) is the authoritative US source for cancer survival statistics. The SEER Explorer exposes PHP endpoints that return JSON with survival rates, standard errors, confidence intervals, case counts, and time series from 1975–2021 across 74 cancer site codes. No authentication or API key required.

Imported from March ticket 173. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/173-add-seer-explorer-survival-data-integration
