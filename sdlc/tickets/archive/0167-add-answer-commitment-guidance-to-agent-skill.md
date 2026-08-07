---
flow: build
priority: 8
---
# Add answer commitment guidance to agent skill

74 BioASQ tasks (41 stopped-early + 33 over-investigated) scored 0 because the agent found relevant data but failed to commit to an answer. In several cases the agent achieved 100% gold PMID recall, read the paper, and still returned no_answer. This is the largest actionable failure class after scoring overlays are applied.

Completed under March on 2026-04-09, as March ticket 167. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/167-add-answer-commitment-guidance-to-agent-skill
