---
base: 3c3697f6c4c579bc5adad790f21fd7fec3f7c829
head: e24618e93296d498c52c9ee70de4f4292e984365
---
The efficiency thread (strategy P1, ticket 579) exists because a blind agent-eval measured BioMCP at ~5× the tokens of a raw-API agent and 0 MCQ lift at 19× cost — the agent's context is the scarce resource. Ticket 579 shipped compact-by-default article search (measured 2.6× smaller than `--full`: 6,187 vs 16,289 bytes on a 15-row BRAF search, 2026-07-18), but there is **no regression guard** on output size, so a future change could silently re-inflate the agent-facing footprint. We also cannot currently *prove* the per-call efficiency win with a repeatable number.

Imported from March ticket 592. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/592-add-a-deterministic-output-footprint-benchmark-with-a-compact-output-ratchet
