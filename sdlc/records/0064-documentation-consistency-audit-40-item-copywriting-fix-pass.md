---
base: 3ada44fb84b96d4ee73237489b078bd859c002f1
head: b2e0c11fac3def27d57b4936e112a9dc4a420b64
---
A full documentation audit (README, docs/, examples/, demo/, scripts/, paper/, benchmarks/) revealed 40 concrete consistency issues. These range from factual contradictions (title says "35 tools," body says "36") to structural drift across peer pages (entity pages missing sections that siblings have) to terminology inconsistencies ("fulltext" vs "full-text" vs "full text"). The README and docs/index.md were recently rewritten with marketing voice (ticket 061), but the rest of the docs tree wasn't touched — so the entry points sound great but the interior pages have accumulated drift. Fixing these in one pass prevents them from compounding as new docs are added.

Imported from March ticket 064. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/064-documentation-consistency-audit-40-item-copywriting-fix-pass
