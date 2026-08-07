---
flow: build
priority: 7
---
# Make `get drug --region` a first-class CLI flag across help, docs, and specs

The v0.8.18 review found that `biomcp get drug` accepts `--region` at runtime but does not declare it in the clap option block. The EMA path works, yet the flag is effectively hidden from operators and scripts.

Completed under March on 2026-03-25, as March ticket 049. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/049-get-drug-region-first-class-flag
