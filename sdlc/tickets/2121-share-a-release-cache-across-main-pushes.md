# 2121 — Share a release cache across main pushes

Status: WITHDRAWN.

Opened as: 2026-10-17, Automatic main CI ends under the approved release-only policy; a cache for repeated main CI runs no longer serves this release..

Milestone: 0.9.3

## Outcome

The release-panic and full-features CI jobs reuse a release-profile build cache saved by earlier main pushes, so each compiles incrementally instead of from zero.

## Evidence

- 2116 audit, 2026-10-16: both jobs set `cache: job` in `.github/workflows/ci.yml` (lines 310 and 331), so every run gets its own cache key and builds release output cold. Mean over 12 green main runs: release-panic 15.2 min, full-features 10.6 min, about 25.8 runner-minutes per green main push and about 97.6 runner-hours over the 227 green main pushes in two weeks.
- The build job already saves a `shared` key that only main pushes write and the lanes restore.

- Starts from: `.github/workflows/ci.yml`, `.github/actions/**`.
- Decisions:
  - Give the two jobs one shared release-profile cache key that only main pushes save; each job restores it. Keep the cache inputs (toolchain, `Cargo.lock`, features, profile) in the key so a change to them builds cold.
  - The jobs stay main-only, as 2075 set.
- Changes: claims `.github/workflows/ci.yml` `.github/actions/**` `tests/test_ci_workflow_contract.py` `CHANGELOG.md`.
- Proof: two consecutive green main runs after landing; the second shows both jobs restoring the shared cache, and its release-panic and full-features times are recorded in this ticket beside 15.2 and 10.6 min. Expected: about 2 to 5 min each.
- Order: shares `ci.yml` with 2094; land either first and rebase the other.
