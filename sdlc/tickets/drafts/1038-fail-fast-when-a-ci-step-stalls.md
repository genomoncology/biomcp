---
flow: build
priority: 7
hold: draft for review; do not promote until Ian releases this
---
# Fail fast when a CI step stalls

The `canonical-gates` job on CI run 32302143809 ran for six hours and was then cancelled by GitHub's default job limit. It never reached a gate. The whole six hours was spent inside step 7, `Install canonical gate tools`, which started at 21:06:57 and was still running at 03:06:39. Steps 8 through 11 — the lint, test, and specification gates — were skipped. The same job normally finishes in about 25 minutes.

No workflow in `.github/workflows/` sets `timeout-minutes` on any job or step. That is why a stall costs six hours rather than failing in a few minutes with a message.

The step that stalled does several things that can hang on someone else's service: `sudo apt-get update` and an `apt-get install` of four pinned packages, then `cargo install cargo-nextest --locked` and `cargo install cargo-deny --locked`, which compile from source, then two `uv tool install` calls. Any one of those can block indefinitely on a network read.

The release workflow has the same step, at `.github/workflows/release.yml:81`. A release can therefore hang the same way, which matters more than a hung branch build.

The cost is not the runner minutes. It is that a six-hour cancelled job looks, on the checks list, much like a failure — so the response is to re-run rather than to read, and that is the same habit that hides a real failure. Five of the last twelve CI runs on `main` were red, and this run is part of why.

## The hard choice to settle

Decide between putting a time limit on the jobs and steps, removing the from-source builds by installing prebuilt binaries or restoring a cache, and doing both. A time limit alone turns a six-hour hang into a fast red, which is an improvement but still a red build. Removing the source builds shortens the window in which a stall is possible but does not close it. Pick one, apply it consistently to every workflow rather than only the one that failed, and say in the design why.

Whatever limit is chosen must have headroom over the observed honest duration of each job — `canonical-gates` around 25 minutes, `full-features` around 13, `windows-contracts` around 7 — so that a slow-but-working run is not killed. Say where the numbers came from.

## Done when

- A job or step that stops making progress fails within a bounded time instead of running to GitHub's default limit.
- The bound applies to every job in every workflow under `.github/workflows/`, including the release workflow, not only `canonical-gates`.
- A run killed by the bound says so in a way that distinguishes it from a gate failure, so a reader can tell a stall from a real red.
- Normal runs at their observed durations are unaffected.

## Related

`sdlc/tickets/drafts/1036` covers a deterministic Windows test making a live network call. Both are the same underlying complaint — an unattended gate judged on someone else's uptime — but the fixes are unrelated and they touch different files. Neither blocks the other.

## Existing tests that pin this

None. The workflow files are not asserted against by any test in `tests/`. Checked 2026-08-20. No restatement is needed or authorized.
