---
base: 55dac615d3fb05dfbfd2554f4fa666927f8bece3
head: 63565f653637087a0ad815c92f13e401fab32f08
---
74 BioASQ tasks (41 stopped-early + 33 over-investigated) scored 0 because the agent found relevant data but failed to commit to an answer. In several cases the agent achieved 100% gold PMID recall, read the paper, and still returned no_answer. This is the largest actionable failure class after scoring overlays are applied.

Imported from March ticket 167. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/167-add-answer-commitment-guidance-to-agent-skill
