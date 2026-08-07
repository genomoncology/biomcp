---
flow: build
priority: 9
---
# Fix gene-to-pgx and disease-to-drug See-also commands

Two cross-entity See-also links emit commands that are broken or misleading. Gene cards always emit `biomcp get pgx <symbol>` which hard-fails for non-pharmacogenes (TP53 exits 1) or silently returns the wrong entity (BRAF returns G6PD's card). Disease cards emit `biomcp search drug <name>` (positional name search) instead of `biomcp search drug --indication <name>`, returning irrelevant drug-name matches instead of treatment options. These are the primary cross-entity teaching surfaces and they teach wrong/broken commands.

Completed under March on 2026-03-28, as March ticket 077. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/077-fix-gene-to-pgx-and-disease-to-drug-see-also-commands
