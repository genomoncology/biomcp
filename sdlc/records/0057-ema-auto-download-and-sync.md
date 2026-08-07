---
base: 71912c735f60f880dd773ec31e8e39607860aa1f
head: 47f08eff34d0b6dacbd6391163dc32e41f33b4ce
---
EMA data requires a manual multi-step curl download before any EU drug command works. Users hit a wall on first use with a "Missing required EMA file(s)" error and a URL to go figure it out. The data is public, small (~11 MB), and has no auth — BioMCP should just fetch it automatically.

Imported from March ticket 057. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/057-ema-auto-download-and-sync
