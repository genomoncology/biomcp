# 2076 — Replace the ratchet scripts with compiler lints

Status: OPEN.

Milestone: 0.9.3

## Outcome

Size and wait rules live in clippy, and no commit exists only to re-stamp a baseline file.

## Evidence

- `tools/check-quality-ratchet.py`, `tools/check-test-wait-ratchet.py`, four inventory files and about 300 Python tests of the checkers cost 70 s of `make lint` and 427 s of the Python lane; 56 of 659 commits since 2026-10-01 only re-stamped a baseline (`sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 5).
- Idea due after the 0.9.2 tag: keep each check that caught a real defect, delete each that only makes the builder re-stamp a baseline.

- Starts from: `sdlc/designs/2026-10-12-0-9-3-iteration-speed.md`, step 5.
- Keeps: any check that caught a real defect, named in this ticket's record with the defect; the capture-receipt manifest as data.
- Changes: `clippy::too_many_lines` at the current threshold and `disallowed_methods` for sleep and clock calls; delete the ratchet scripts, their inventory files, the structure tests and the checkers' Python tests.
- Proof: `make lint` and Python lane times before and after; clippy fails on a planted long function and a planted sleep; no commit touching only a `tools/*.json` file for the rest of 0.9.3.
- Defers: nothing.
