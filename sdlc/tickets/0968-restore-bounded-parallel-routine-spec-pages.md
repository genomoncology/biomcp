---
flow: build
priority: 10
deps: ["0967", "0969"]
---
# Restore bounded parallel execution to routine spec pages

Routine Markdown specs ran with file-level pytest-xdist parallelism before the
standalone Mustmatch migration. The current binary has no parallel option, so
the runner executes roughly twenty paths serially. A warm-binary `make spec`
takes 592 seconds while consuming only 53 seconds of user CPU.

## Test contract

Add a runner-level bounded worker pool over isolated spec files. Begin with a
conservative explicit worker count. A synthetic suite must prove concurrency,
failure aggregation, interrupt cleanup, deterministic output attribution, and
the ability to force one worker for diagnosis.

## Done when

- Independent Markdown paths execute concurrently while commands within one
  path retain Mustmatch order.
- Known shared-state pages are isolated or placed in an explicit serial group
  with a checked reason; there is no broad undocumented serial fallback.
- One failing page stops new work, reports the exact page and Mustmatch output,
  and all running fixture process groups are reaped.
- The existing parallel-isolation contract passes repeatedly at the chosen
  default worker count under loaded-machine conditions.
- Before/after complete `make spec` timings and worker counts are recorded.

## Authorized test changes

Design commits may restate `scripts/run-specs.sh`, fixture ownership helpers,
routine-path metadata, runner lifecycle tests, and timing documentation. Product
behavior and spec assertions do not change.

The src line ceiling may not rise.
