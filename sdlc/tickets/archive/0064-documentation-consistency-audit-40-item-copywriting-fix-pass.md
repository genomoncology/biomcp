---
flow: build
priority: 5
---
# Documentation consistency audit — 40-item copywriting fix pass

A full documentation audit (README, docs/, examples/, demo/, scripts/, paper/, benchmarks/) revealed 40 concrete consistency issues. These range from factual contradictions (title says "35 tools," body says "36") to structural drift across peer pages (entity pages missing sections that siblings have) to terminology inconsistencies ("fulltext" vs "full-text" vs "full text"). The README and docs/index.md were recently rewritten with marketing voice (ticket 061), but the rest of the docs tree wasn't touched — so the entry points sound great but the interior pages have accumulated drift. Fixing these in one pass prevents them from compounding as new docs are added.

Completed under March on 2026-03-27, as March ticket 064. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/064-documentation-consistency-audit-40-item-copywriting-fix-pass
