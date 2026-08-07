---
flow: build
priority: 7
---
# Add study CLI help descriptions and regression test

`biomcp study --help` lists nine subcommands with blank description strings. Individual subcommands (`study query`, `study filter`, `study compare`, etc.) also have blank descriptions for most flags. Every other command family in the CLI provides sentence descriptions at this level. The `biomcp list study` reference page has thorough descriptions for the same commands, so the information exists but is not surfaced where users look first.

Completed under March on 2026-04-15, as March ticket 210. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/210-add-study-cli-help-descriptions-and-regression-test
