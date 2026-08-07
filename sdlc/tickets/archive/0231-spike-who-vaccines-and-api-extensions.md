---
flow: spike
priority: 6
---
# Spike: WHO vaccines and API extensions

WHO Prequalification currently covers finished pharmaceutical products only. The same WHO portal also publishes prequalified vaccines (distinct list with different schema) and active pharmaceutical ingredients (190+ raw substances). Before extending the drug entity loader, we need to understand the data shape, overlap with existing drug identity (MyChem INNs), and whether vaccines need special handling (target pathogen, schedule, cold chain) vs fitting into the existing drug model.

Completed under March on 2026-04-17, as March ticket 231. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/231-spike-who-vaccines-and-api-extensions
