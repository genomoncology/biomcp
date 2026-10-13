# 2058 — Close the changelog loop at the 0.9.2 tag

Status: COMPLETE.

Milestone: 0.9.2

## Outcome

Every ticket merged into the 0.9.2 tag carries its changelog bullet, the release gate proves it, and the repository rule keeps it true for later releases.

## Evidence

Filed 2026-10-12 from the second gated rehearsal run 38320558917: version-check failed with "CHANGELOG 0.9.2 is missing described bullets for tickets merged since v0.9.1: 2057". The gate counts every landed ticket, so a ticket whose only change is another ticket's bullet still needs its own; each fix ticket would otherwise spawn the next.

- Starts from: main `69e338aa4`.
- Changes: one Internal bullet with closing group (2057) for the 2055 note it landed, one with closing group (2058) for this landing, both in this branch so the tag's changelog closes in one merge; the expected-ticket set gains 2057 and 2058; the Releases section of AGENTS.md gains the general rule — a ticket merged for a release carries its changelog bullet in its own landing.
- Proof: a scratch-tag run of scripts/check-changelog-coverage.py at this branch's head reports the changelog covers every ticket merged since v0.9.1 including 2057 and 2058; the changelog and coverage tests pass; the third rehearsal's version-check passes.
- Defers: nothing.

Reviews: revision 31db0bb, accept
