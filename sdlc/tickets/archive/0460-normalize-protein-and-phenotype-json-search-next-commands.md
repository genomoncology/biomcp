---
flow: build
priority: 8
---
# Normalize protein and phenotype JSON search next commands

Successful JSON entity searches are supposed to teach the next executable step through `_meta.next_commands`. The review found `search protein --json` and `search phenotype --json` returning only `pagination`, `count`, and `results` because they use a bare generic search JSON helper. That is an agent-facing correctness gap: scripts get a valid JSON object but no follow-up contract.

Completed under March on 2026-06-29, as March ticket 460. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/460-normalize-protein-and-phenotype-json-search-next-commands
