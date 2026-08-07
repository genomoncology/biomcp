---
base: d33b6ea60972411bde5094fdb3de125b37bef8d9
head: 4c22a31c75fbf5eeb93e669754ee32811a0541cd
---
`biomcp --json drug interactions warfarin` took 150 seconds on a machine that already had a usable DDInter bundle and returned 864 detail rows. A normal read first tried to refresh all eight stale DDInter CSV files, then issued MyChem lookups across the uncapped partner set. That makes a lookup behave like maintenance, lets an auxiliary source dominate latency, and sends far more data than an agent normally needs.

Imported from March ticket 497. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/497-make-ddinter-reads-local-bounded-and-page-able
