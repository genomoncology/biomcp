---
base: f21710b5b3840d1f2627d0317b0bc939772f478b
head: aac7ab1649a4ef5bab3ff1c74f7a56cca5a4c9a2
---
BioMCP's entity-aware HATEOAS suggestions (ticket 202) only appear in human-readable text output. When agents use `--json`, they get structured data but no follow-up suggestions. BioASQ evaluation found 4/32 tasks used `--json` and missed all HATEOAS hints. The suggestions are already generated — they just need to be included in the JSON envelope.

Imported from March ticket 241. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/241-add-hateoas-suggestions-to-json-output-mode
