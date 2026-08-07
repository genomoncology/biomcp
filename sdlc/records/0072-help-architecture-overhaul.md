---
base: fb58fb132e6905be3d8e11c3faddf2853286c14f
head: aaedb2888f7a62489290388b3629ec3410bffa8f
---
Research 005's BioASQ answerability audit tested 58 questions against BioMCP's actual surfaces. 73% of failures were answerable — the data was there but the agent didn't find the right surface. The root cause is that agents learn BioMCP from a 200-line static skill file instead of from the CLI itself. When an agent encounters BioMCP for the first time (or via GEPA prompt optimization), it needs the CLI to teach it:

Imported from March ticket 72. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/072-help-architecture-overhaul
