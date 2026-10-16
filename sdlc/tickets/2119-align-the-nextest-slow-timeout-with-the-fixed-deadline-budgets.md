---
flow: build
priority: 2
deps: [2102, 2103]
---

# 2119: Align the nextest slow timeout with the fixed deadline budgets

Status: WITHDRAWN.

Opened as: 2026-10-16, superseded by 2126.

Milestone: 0.9.3

Superseded by: 2126

## Evidence

- 2102 raised the article-query test's deadline budget to 600 s via the test seam, but nextest's default 240 s slow-timeout kills it first under co-load (four closing-gate failures on 2026-10-16; with NEXTEST_SLOW_TIMEOUT=600000 the suite passes 4276/4276). The watchdog predates the fix it now contradicts.

## Work

- Set per-test slow-timeout overrides in .config/nextest.toml for the tests whose deadline budgets exceed 240 s, keeping the 240 s default for everything else; closing gates then need no env prefix.

## Acceptance

1. A plain closing-gate make test passes on a co-loaded host without NEXTEST_SLOW_TIMEOUT, with the named tests still bounded by their configured watchdogs.
