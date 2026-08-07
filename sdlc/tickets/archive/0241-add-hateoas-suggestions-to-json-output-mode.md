---
flow: quickfix
priority: 7
---
# Add HATEOAS suggestions to JSON output mode

BioMCP's entity-aware HATEOAS suggestions (ticket 202) only appear in human-readable text output. When agents use `--json`, they get structured data but no follow-up suggestions. BioASQ evaluation found 4/32 tasks used `--json` and missed all HATEOAS hints. The suggestions are already generated — they just need to be included in the JSON envelope.

Completed under March on 2026-04-18, as March ticket 241. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/241-add-hateoas-suggestions-to-json-output-mode
