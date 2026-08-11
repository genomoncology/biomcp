---
flow: build
priority: 10
deps: ["0951"]
---
# Scan tracked lint inputs once

Warm `bin/lint` takes about 40 seconds even when Clippy finishes in less than
one second. The shell implementation repeatedly launches text-processing
commands for individual tracked files and, in the documentation leak scan, for
individual lines.

## Test contract

Add fixtures for every current credential, TBD, stale pycache, deprecated-doc,
and documentation-code-leak outcome. Instrument the lint implementation so one
test can prove each tracked input corpus is collected once and no check starts
a subprocess per file or line.

## Done when

- Every existing positive and negative lint fixture retains the same result and
  useful path/line diagnostic.
- Tracked paths are collected once and text needed by multiple audits is read
  once per invocation.
- No audit loops over files or lines by starting grep, sed, awk, or a shell for
  each item.
- The credential scan still excludes only its explicit test fixture and never
  prints unrelated environment values.
- A warm loaded-machine benchmark records `bin/lint` before and after with
  Clippy already warm.

## Authorized test changes

Design commits may restate `bin/lint`, add one small lint helper, and change
`tests/test_lint.py`. No lint rule or covered path may be deleted to obtain the
speedup.

The src line ceiling may not rise.
