---
flow: build
priority: 8
---
# Entity-aware HATEOAS on article search (harden ticket 242 heuristic)

When an agent's question is about a known gene/drug/disease, the agent often jumps directly to `biomcp search article -k "<entity>"` instead of `biomcp get gene|drug|disease <entity>`. The 009 deep dive (192-task panel on BioASQ + gpt-5-mini + BioMCP 0.8.21) measured 23 of 32 entity-typed stopped-early tasks (72%) skipping the structured command; 8 finished zero.

Completed under March on 2026-04-21, as March ticket 277. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/277-entity-aware-hateoas-and-workflow-ladders-on-article-search-and-first-call-responses
