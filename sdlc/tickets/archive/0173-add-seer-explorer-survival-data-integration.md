---
flow: build
priority: 7
---
# Add SEER Explorer survival data integration

BioMCP covers genes, drugs, trials, and literature but cannot answer "what is the 5-year survival rate for CML?" — the most fundamental cancer outcome metric. SEER (Surveillance, Epidemiology, and End Results) is the authoritative US source for cancer survival statistics. The SEER Explorer exposes PHP endpoints that return JSON with survival rates, standard errors, confidence intervals, case counts, and time series from 1975–2021 across 74 cancer site codes. No authentication or API key required.

Completed under March on 2026-04-10, as March ticket 173. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/173-add-seer-explorer-survival-data-integration
