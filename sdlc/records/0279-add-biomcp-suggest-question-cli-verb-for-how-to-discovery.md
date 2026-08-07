---
base: a84acb684e963d15cb0910bf5e30ba26b3286ff0
head: 991c20c68b135ba000850b620439ee69526189d3
---
The agent has no native way to look up "what's the right command sequence for this question?" The current path is `biomcp skill list` → read the table → match by hand → `biomcp skill <slug>`. Across 192 tasks in the 009 deep dive, **zero** invoked `biomcp skill list`. The discovery primitive is missing.

Imported from March ticket 279. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/279-add-biomcp-suggest-question-cli-verb-for-how-to-discovery
