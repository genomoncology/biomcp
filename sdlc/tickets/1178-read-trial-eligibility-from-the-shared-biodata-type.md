---
flow: build
priority: 8
deps: []
---

# Read trial eligibility from the shared BioData type

## Status

This ticket reserves id 1178. It does not specify the work.

The design and acceptance criteria already exist, written by the BioData migration team before the implementation, in `sdlc/records/1178-consume-biodata-clinical-trial-eligibility.md` on branch `origin/ticket/1178-implementation`. That record is the authority: it covers the provider flow for both sources, the exact JSON member contract, the Markdown rendering rules, the test plan, ten numbered acceptance criteria, and the growth ceilings. Read it there.

## Why this file exists

The implementation branch was named for id 1178, but no ticket claimed that id and `next-id` still returned it, so the next ticket filed anywhere would have taken 1178 and made the branch name wrong. This file holds the id until the migration team's record lands on main.

An earlier version of this file carried acceptance criteria reverse-engineered from the code diff. They were withdrawn on 2026-09-07 because two were wrong. They said default Markdown keeps showing the full eligibility section, when the design has default detail derive only the age line; and they said output stays the same as before, when the design intends a richer JSON contract with registry text and age moving under `eligibility`. Nothing in this file should be treated as a specification.

## Ownership

The BioData migration team owns this work: correcting this ticket, refreshing the implementation against current main, and completing migration review and validation. The BioMCP 0.9 queue does not carry it.

The branch is one commit ahead of its base `e95bb7a4` and behind current main, so it needs a rebase before review.
