---
base: 8ad14e020f1ade17ccc221d9aff016f54cbca1df
head: f85f67d461ae1505e89b8cb1cb5b5bdfe52b05f4
---
BioASQ evaluation shows 63% of wrong-command failure tasks have gold PMIDs from 2002-2014 that are unreachable because article search returns recent papers by default. Across 19 wrong-command tasks, the agent found only 10 of 137 gold PMIDs (7.3% recall). A date range filter would let the agent (or user) target older literature when keyword search returns only modern papers.

Imported from March ticket 182. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/182-add-date-range-filter-to-article-search
