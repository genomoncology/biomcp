---
flow: build
priority: 3
deps: []
---

# 2006: Keep ticket numbers unique across main and the 1.0 line

## Outcome

`make lint` fails when a ticket number on the 1.0 line duplicates a number main already uses in a different file, or when a 1.0-only ticket uses a number below 2001. Main's 1000s and the 1.0 line's 2000s never collide again.

## Current Facts

- Main files tickets in the 1000s. The 1.0 line files tickets numbered 2001 and up (`AGENTS.md`, "BioData 1.0 migration verification"; ticket 2001's own numbering note).
- The two lines once collided on ticket numbers 1234 through 1236, one file on each line sharing a number.
- `make lint` already runs `tools/check-quality-ratchet.sh` and other repository-owned checks (`Makefile`, `lint` target).

## Scope

- A lint script reads the ticket numbers filed in `sdlc/tickets/` on the current branch and the ticket numbers filed in `sdlc/tickets/` on `origin/main`.
- It fails when the current branch is the 1.0 line and a ticket number there matches a main ticket number attached to a different file path.
- It fails when the current branch is the 1.0 line and a ticket file there is numbered below 2001.
- It passes silently otherwise, including on main itself.
- Wire the script into `make lint`.

## Exclusions

No change to how tickets are numbered or reviewed. No check of ticket content, only the number and the branch it is filed on.

## Acceptance

1. Planting a ticket file on the 1.0 line numbered to match an existing main ticket, in a different file, makes the script fail.
2. Planting a 1.0-line ticket numbered below 2001 makes the script fail.
3. The script passes on the current tree as pushed.
4. `make lint` runs the script.

`make lint` passes on the gate host at the pushed SHA.

## Dependencies

None.

## Complexity

- Contract score: 1 (one numbering rule, checked against `origin/main`)
- State and timing score: 0
- Reach score: 1 (lint gate only)
- Proof score: 1 (a planted duplicate and a planted low number)
- Cost of error score: 1 (a missed collision loses a ticket's history when a number is reused)
- Total: 4
- Minimum level floor: none
- Final level: 1
- Reasons: a scoped check added to an existing lint gate
- Selected model: claude-sonnet

## Review

- Design review: pending
