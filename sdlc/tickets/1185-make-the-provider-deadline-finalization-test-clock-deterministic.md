---
flow: quickfix
priority: 9
deps: []
---

# Make the provider-deadline finalization test clock deterministic

## Goal

`provider_deadline_waits_for_post_publish_fail_closed_finalization` proves its
deadline and finalization contract on every run, under load, without
wall-clock sensitivity.

## Current behavior

The test at `src/sources/tests/provider_network.rs:357-405` mixes a 40 ms
fixture delay, a 10 ms `VariantArticleDeadline`, and a
`tokio::time::sleep(20 ms)` before asserting `deadline.is_exhausted()`. Under
parallel load the sleep can stretch past the fixture's delayed response,
changing whether the deadline is exhausted before publication and flipping the
assertion. It failed once on 2026-09-09 in a loaded full-suite rerun and passed
in isolation and in the next full run.

## Required behavior

Convert the test to paused tokio time (`start_paused`), as other deadline
tests in this repository already do, or otherwise remove wall-clock
sensitivity. Preserve the fail-closed proof: an exhausted deadline still waits
for cache publication and returns the sanitized error.

## Observable success

Repeated full-suite runs stay green under load.

## Boundaries

Production deadline and cache code unchanged.
