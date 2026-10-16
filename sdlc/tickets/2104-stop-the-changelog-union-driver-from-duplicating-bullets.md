---
flow: build
priority: 2
deps: [2083]
---

# 2104: Stop the changelog union driver from duplicating bullets

Status: OPEN.

Milestone: 0.9.3

Reviews: revision fa667f9, accept

## Evidence

- Filed 2026-10-15 by the PM: CHANGELOG on main carries the 2085 bullet twice, the second copy misspelling hexylresorcinol, after several landings that relied on the union merge driver installed by 2083. A union merge of the same section from two branches can append both sides' copies of an unchanged bullet, duplicating it silently.
- The misspelled duplicate is deleted in the 2101 landing; this ticket owns the cause.

## Work

- Either drop the union driver for CHANGELOG.md and return to ordinary merges with a shape-only test keeping conflicts rare, or extend the driver/test to fail a merge that duplicates an existing bullet line.

## Acceptance

1. Two branches each landing a changelog bullet merge without duplicating any unchanged bullet, proven by a merge rehearsal in the ticket.
