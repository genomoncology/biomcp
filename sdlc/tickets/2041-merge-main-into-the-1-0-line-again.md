# 2041 — merge-main-into-the-1-0-line-again

Status: OPEN.

Milestone: 1.0

## Outcome

The 1.0 branch carries every commit on the release line. The release line flows into 1.0 continuously, so the branch must sit zero commits behind `main`. It is 206 behind at the time of writing, which is the largest single source of merge risk on this line.

After this merge, the branch merges `main` daily, and a day that cannot merge cleanly is reported rather than skipped.

## Evidence

- Starts from: `main` at its current tip and the long-lived branch `the 1.0 release branch`.
- Keeps: every commit on both sides. The merge direction is `main` into the branch, never the reverse.
- Changes: `.github/workflows` if the daily merge needs a check; otherwise no tracked file changes outside the merge commit itself.
- Proof: `git rev-list --left-right --count origin/the 1.0 release branch...origin/main` reports zero on the right, and the branch's own test run passes after the merge.
- Defers: the 1.0 feature work itself. This ticket only ends the drift.

## Risk this reduces

Every day the branch stays behind, a conflict that would have been one file becomes a conflict across many. The release line is what people download, so a 1.0 that cannot absorb it is a 1.0 that cannot ship.
