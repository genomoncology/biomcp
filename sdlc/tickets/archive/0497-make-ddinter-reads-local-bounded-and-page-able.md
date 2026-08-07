---
flow: build
priority: 5
---
# Make DDInter reads local, bounded, and page-able

`biomcp --json drug interactions warfarin` took 150 seconds on a machine that already had a usable DDInter bundle and returned 864 detail rows. A normal read first tried to refresh all eight stale DDInter CSV files, then issued MyChem lookups across the uncapped partner set. That makes a lookup behave like maintenance, lets an auxiliary source dominate latency, and sends far more data than an agent normally needs.

Completed under March on 2026-07-10, as March ticket 497. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/497-make-ddinter-reads-local-bounded-and-page-able
