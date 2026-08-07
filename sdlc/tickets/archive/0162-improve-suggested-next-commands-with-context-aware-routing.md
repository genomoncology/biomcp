---
flow: build
priority: 5
---
# Improve suggested next commands with context-aware routing

BioMCP's suggested follow-up commands ("See also", "More") are largely template-based and don't use what was actually found in the current response to generate targeted suggestions. For example, when `get disease "Dravet syndrome"` returns SCN1A as the causal gene with score 0.872, the suggestions should include `biomcp get gene SCN1A clingen constraint` — not generic templates like `biomcp search article -d "Dravet syndrome"`.

Completed under March on 2026-04-11, as March ticket 162. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/162-improve-suggested-next-commands-with-context-aware-routing
