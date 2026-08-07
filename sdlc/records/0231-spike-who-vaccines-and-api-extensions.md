---
base: e0c6e0fbd51dc2c09a8171339fbfa350320b5b27
head: 0b2dcf79f0f9c8c69ee3f488d15ba384c54611fe
---
WHO Prequalification currently covers finished pharmaceutical products only. The same WHO portal also publishes prequalified vaccines (distinct list with different schema) and active pharmaceutical ingredients (190+ raw substances). Before extending the drug entity loader, we need to understand the data shape, overlap with existing drug identity (MyChem INNs), and whether vaccines need special handling (target pathogen, schedule, cold chain) vs fitting into the existing drug model.

Imported from March ticket 231. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/231-spike-who-vaccines-and-api-extensions
