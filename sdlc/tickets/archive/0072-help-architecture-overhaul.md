---
flow: build
priority: 9
---
# Help architecture overhaul: self-teaching CLI for agent navigation

Research 005's BioASQ answerability audit tested 58 questions against BioMCP's actual surfaces. 73% of failures were answerable — the data was there but the agent didn't find the right surface. The root cause is that agents learn BioMCP from a 200-line static skill file instead of from the CLI itself. When an agent encounters BioMCP for the first time (or via GEPA prompt optimization), it needs the CLI to teach it:

Completed under March on 2026-03-28, as March ticket 72. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/072-help-architecture-overhaul
