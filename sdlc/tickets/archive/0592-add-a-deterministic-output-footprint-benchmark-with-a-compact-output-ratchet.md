---
flow: quickfix
priority: 4
---
# Add a deterministic output-footprint benchmark with a compact-output ratchet

The efficiency thread (strategy P1, ticket 579) exists because a blind agent-eval measured BioMCP at ~5× the tokens of a raw-API agent and 0 MCQ lift at 19× cost — the agent's context is the scarce resource. Ticket 579 shipped compact-by-default article search (measured 2.6× smaller than `--full`: 6,187 vs 16,289 bytes on a 15-row BRAF search, 2026-07-18), but there is **no regression guard** on output size, so a future change could silently re-inflate the agent-facing footprint. We also cannot currently *prove* the per-call efficiency win with a repeatable number.

Completed under March on 2026-07-21, as March ticket 592. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/592-add-a-deterministic-output-footprint-benchmark-with-a-compact-output-ratchet
