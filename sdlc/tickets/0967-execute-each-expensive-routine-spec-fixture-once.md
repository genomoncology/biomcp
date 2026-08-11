---
flow: build
priority: 10
deps: ["0951"]
---
# Execute each expensive routine spec fixture once

`spec/entity/variant-article-identity.md` launches the same fixture script six
times. Each launch starts a server and performs six BioMCP requests to recreate
the same JSON report. Other pages repeat rendering or fixture helpers for
separate assertions.

## Test contract

Give a page one named execution result that multiple expectations can consume,
or prepare one immutable captured result before the page assertions. Instrument
the expensive helpers and prove one routine spec run invokes each scenario the
declared number of times.

## Done when

- The variant-article identity page computes its complete report once while all
  existing claims remain executable.
- Inventory every routine Markdown helper invoked more than once and either
  share its result or record why distinct inputs require distinct executions.
- Cached or shared results are private to one run, identify all inputs that
  affect them, and cannot survive a changed binary or fixture.
- Missing, stale, or malformed prepared output fails rather than silently
  falling back to another BioMCP executable.
- Before/after per-page and complete `make spec` timings are recorded.

## Authorized test changes

Design commits may restate routine Markdown pages, their fixture helpers, the
runner preparation contract, and runner tests. No behavioral assertion may be
deleted merely because it shared an execution with another assertion.

The src line ceiling may not rise.
