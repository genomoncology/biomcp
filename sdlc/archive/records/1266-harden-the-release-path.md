---
ticket: 1266
head: 80095256
merge: 61d11ab3
---

# 1266 — the release path waits for the public release

## What changed

The Homebrew tap job now depends on `publish-release` (the GitHub release is public before the formula lands) and on the PyPI job. The wheel smoke test asserts real asset content instead of a 200 status. A structural workflow assertion requires `--release --locked` on every cargo/maturin build step in the release workflow, so a future edit cannot quietly build an unlocked debug binary into the artifacts.

## Evidence

- Branch head `80095256` ("Update the runbook for the reversed tap"); merge `61d11ab3`. The branch CI run predates the Actions API's retrievable window at record time; no run ID is cited.
- Code review: the ticket file's verdict lines are the record; the 2026-09-29 review round re-read this workflow work and raised no new findings against it.
