# BioMCP plan

This file holds the 1.0 destination, the order of 1.0 work, and each milestone's exit. It never says what is done. Read status from `pm daily`, `pm next`, `pm item` and `pm inbox`, filtered with `--milestone 1.0`.

Rulings: [decisions](decisions/).

**Scope.** This plan covers the 1.0 line. The 0.9 release line on `main` is driven by a separate lane with its own order, and nothing here sets its priorities.

## Outcome

BioMCP 1.0 answers from shared typed values rather than from shapes it defines itself. Every command returns a type the shared library owns, so a caller gets the same value whichever surface it asks through, and a parsing fix reaches every consumer at once.

Today most commands return shapes defined here, and a handful return shared types. The gap is the work.

## How the two lines relate

`main` is the public download surface. Someone installing BioMCP gets it. Ian ruled on October 9 that [the release line flows into 1.0, never the reverse](decisions/2026-10-09-the-release-line-flows-into-1-0.md).

So the 1.0 branch merges `main` in continuously and never pushes code to it. Only records go to `main` from this lane: 1.0 tickets live in `main`'s ticket folder under the 1.0 milestone, so one database serves both lines and numbers cannot collide.

## Milestones

**1. Zero drift.** The 1.0 line carries the release line's tip.

Exit: the branch is never more than a day behind `main`, and a check reports its distance rather than anyone measuring it by hand.

**2. A suite that runs.** The 1.0 line's own gate passes on the build host.

Exit: the gate command completes with a reported result. Today the suite stalls when run as threads in one process because tests mutate process-global state, so the per-process form is the only one that finishes, and nobody has recorded the outcome either way.

**3. Shared types at the surface.** A command's output shape is not defined here.

Exit: every command returns a shared typed value, the hand-written pattern matching over provider payloads is gone, and no exported record carries raw bytes where a shared decoder exists.

**4. One ticket database.** The two lines never fork their records again.

Exit: every 1.0 ticket lives on `main` with its milestone, a check refuses a ticket number that exists on both lines in different files, and `pm next --milestone 1.0` is the whole queue.

## Order

Open 1.0 tickets only, in the order I want them taken. Status comes from pm, never from this list.

- 2006
- 2005
- 2003
- 2004

2006 leads because it is the check that stops the two lines colliding again, and milestone 4 cannot close without it. 2005 follows because it removes the reason a developer bypasses the commit hook, which is how unchecked work reaches a branch. 2003 and 2004 are the remaining patient-facing capability work.

Returning the 1.0 line to current `main` needs its own ticket and does not yet have a number. Drawing one requires push access this lane does not currently hold.
