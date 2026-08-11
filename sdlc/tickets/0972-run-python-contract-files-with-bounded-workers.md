---
flow: build
priority: 10
deps: ["0951"]
---
# Run Python contract files with bounded workers

The complete Python contract lane is still sequential even though its tests
already pass under pytest-xdist. A measured four-worker run passed all 500
tests in 43.28s versus about 105s sequentially.

## Test contract

Declare one bounded Python worker setting in the Makefile and prove the
canonical `test-contracts` target consumes it with file-based distribution.
Retain a one-worker override for diagnosis. Do not change test assertions,
timeouts, or which files the gate collects.

## Done when

- `make test` runs the same complete Python contract corpus with four workers
  and `--dist loadfile`.
- `PYTEST_WORKERS=1` provides a deterministic diagnostic path without editing
  the Makefile.
- A source contract rejects an unbounded `-n auto`, missing distribution mode,
  or a sequential default.
- Two complete four-worker runs pass without shared-file, environment, fixture,
  or process-lifecycle failures.
- Before/after wall time is recorded under the same prepared binary condition.

## Authorized test changes

Design commits may change only Python gate orchestration, its source contract,
and timing documentation. Product behavior and test assertions do not change.

The src line ceiling may not rise.
