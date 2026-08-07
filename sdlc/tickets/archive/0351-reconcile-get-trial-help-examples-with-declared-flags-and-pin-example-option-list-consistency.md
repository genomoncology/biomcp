---
flow: build
priority: 4
---
# Reconcile get trial help examples with declared flags and pin example/option-list consistency

`biomcp get trial --help` shows an `EXAMPLES:` line that uses `--offset 20 --limit 20`, but `--offset` and `--limit` do not appear anywhere in `get trial`'s option list. Users who copy the example get `unexpected argument` errors. The 348 outside-in review confirmed this against the release binary and classified it as a minor cosmetic inconsistency that nonetheless misleads first-run users following the in-binary documentation.

Completed under March on 2026-04-30, as March ticket 351. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/351-reconcile-get-trial-help-examples-with-declared-flags-and-pin-example-option-list-consistency
