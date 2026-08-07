---
base: 99f70df534c0fdcc8f115b006129d7057777cb49
head: 94a71ba7fa6bc865aec8fb47819dc94b650ad971
---
`biomcp get trial --help` shows an `EXAMPLES:` line that uses `--offset 20 --limit 20`, but `--offset` and `--limit` do not appear anywhere in `get trial`'s option list. Users who copy the example get `unexpected argument` errors. The 348 outside-in review confirmed this against the release binary and classified it as a minor cosmetic inconsistency that nonetheless misleads first-run users following the in-binary documentation.

Imported from March ticket 351. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/351-reconcile-get-trial-help-examples-with-declared-flags-and-pin-example-option-list-consistency
