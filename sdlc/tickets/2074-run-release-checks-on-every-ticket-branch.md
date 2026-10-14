# 2074 — Run release checks on every ticket branch

Status: OPEN.

Milestone: 0.9.3

## Outcome

A ticket branch fails CI when the release would fail, so a tag needs one rehearsal, and process-only tickets need no changelog bullet.

## Evidence

- The 0.9.2 tag needed four rehearsals. Two failed 13 s into the release workflow's version check, each after a 26-minute CI run: one found a missing changelog bullet, the other a `gh` flag only the test stub had (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 9).
- 2057 and 2058 existed only to add changelog bullets for changelog fixes (idea: let the coverage check skip tickets that change no shipped behavior).
- Two of three review rounds on 2052 went to removing ticket numbers from code comments (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 10).

- Starts from: the release workflow's version-check job and `tools/` changelog coverage check.
- Keeps: the tag-time version check.
- Changes: run the changelog coverage and green-CI query steps in CI on every ticket branch with the real `gh`; let the coverage check skip tickets whose diff touches only `sdlc/`, `docs/` process files or CI; add a lint step that fails on a ticket number in a source comment.
- Proof: a branch with a missing bullet fails CI; a process-only ticket passes without a bullet; a planted comment ticket number fails lint and main passes; the 0.9.3 tag passes its first rehearsal.
- Defers: the changelog's file shape and the hand-written 0.9.3 ticket list in the changelog test (2083).
