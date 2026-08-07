---
base: 6de0b82dc5a1f0cc67560915b77fb7b0ebcfd215
head: 08496ef083fe96244042247e35d8babfcf383c49
---
When an agent's question is about a known gene/drug/disease, the agent often jumps directly to `biomcp search article -k "<entity>"` instead of `biomcp get gene|drug|disease <entity>`. The 009 deep dive (192-task panel on BioASQ + gpt-5-mini + BioMCP 0.8.21) measured 23 of 32 entity-typed stopped-early tasks (72%) skipping the structured command; 8 finished zero.

Imported from March ticket 277. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/277-entity-aware-hateoas-and-workflow-ladders-on-article-search-and-first-call-responses
