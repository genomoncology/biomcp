---
base: 1883a35bad4b33a29aa303608f3da7ffec87ad77
head: ecc6a436f1ff9ed321f657586791c99aad352d81
---
Two cross-entity See-also links emit commands that are broken or misleading. Gene cards always emit `biomcp get pgx <symbol>` which hard-fails for non-pharmacogenes (TP53 exits 1) or silently returns the wrong entity (BRAF returns G6PD's card). Disease cards emit `biomcp search drug <name>` (positional name search) instead of `biomcp search drug --indication <name>`, returning irrelevant drug-name matches instead of treatment options. These are the primary cross-entity teaching surfaces and they teach wrong/broken commands.

Imported from March ticket 077. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/077-fix-gene-to-pgx-and-disease-to-drug-see-also-commands
