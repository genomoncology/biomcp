---
base: 1dc55cfd0b7960b9242301da374d65de59060c0a
head: bc22b14b7e3f0e4803c64d01f0a05e4e8f48522f
---
BioASQ evaluation research (project 009) found that agents using BioMCP miss obvious structured-data shortcuts. The agent searches articles 5+ times for gene localization when `get gene X protein` has it. It parses abstracts for disease associations when `get gene X diseases` lists them. Adding how-to recipes to SKILL.md costs nothing and steers agents to the right commands.

Imported from March ticket 240. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/240-add-bioasq-how-to-recipes-to-skill-md
