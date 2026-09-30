# 1278 — the CI cache and the docs-only skip

Filed 2026-09-29, retroactively: the first landing (2557b9f4, reworked bd75c81e, merged a82fda55) changed CI with no ticket, and the 2026-09-29 review found it unsound. This ticket owns the rework on branch `tickets/1278-ci-rework`.

## The defects the review found

1. The docs-only skip read `github.event.head_commit.message`, so a code push whose last commit said `docs:` skipped every Rust job and still showed green, and a `docs:` commit editing `src/` skipped the jobs that would have caught it. Tagged `docs:` commits could have met the release bar with no Rust tests. 2. The `docs:` skip also skipped `pytest tests/` and `mkdocs build --strict`, which live in canonical-gates' `make test` — so the commits that most needed the docs and record tests ran none of them. 3. The cache restored before checkout (keyed on nothing it could read) and before the toolchain install, so the key tracked the runner's default rustc, not the pinned 1.93.1. 4. The cache action was pinned to `f00db758…`, a commit that does not exist in the action's repository. Fabricated. The rework pins `6323deb102c322ba6fcbdcafc7e3dddab59af2b6`, read by dereferencing the `v2` tag through the GitHub API on 2026-09-29. 5. `cache-workspace-crates` defaulted to false, so the biomcp crate itself was rebuilt from scratch every run; full-features and release-panic showed no cache benefit. 6. Nextest installed through `curl | tar` with no checksum.

## The rework

- A `changes` job diffs every file changed across the whole push (`github.event.before..sha`); the Rust jobs skip only when every changed file is documentation, an `sdlc/` file, or a `notes/` file. Pull requests and tags always run everything. `scripts/ci-classify-push.sh` holds the logic.
- `repository-contracts` always runs, and gained `uv sync` + `pytest tests/ -m "not needs_binary"` + `mkdocs build --strict`, so docs-only pushes still run the docs and record tests.
- The `needs_binary` pytest marker covers every test module that drives a built binary or cargo (35 modules, scan-guarded by `tests/test_ci_workflow_contract.py`); the docs-only lane deselects them, the canonical lane runs everything.
- The cache step runs after checkout and after the pinned toolchain, uses the real commit pin, and sets `cache-workspace-crates: true`.
- `scripts/install-nextest.sh` downloads the pinned release tarball (0.9.146) and verifies its SHA-256 (`b64617e8…a6aa`, computed from the artifact itself) before installing; all three call sites use it.
- `tests/test_ci_workflow_contract.py` pins all of the above structurally.

## Before and after times

Recorded from the Actions API, job wall times in seconds:

| Run | full-features | release-panic | canonical-gates |
|---|---|---|---|
| `9eea4a86`, no cache | 927 | 1020 | 2058 |
| `bd75c81e`, cache after checkout (rework input state) | 698 | 743 | 1481 |
| rework branch run 36633279293 (green, all eight jobs) | 690 | 839 | 1563 |

The workspace-crates cache did not shrink full-features or release-panic on this run (690 s and 839 s sit at the earlier cached levels); the first run after a key change pays a full restore-miss. The honest comparison needs the NEXT run on the same key: if that run also shows full-features near 700 seconds and release-panic near 850, the workspace-crates benefit is not materializing and the follow-up belongs in the 1.0 speed-up issue (which Ian approved on 2026-09-29). The docs-only lane, by contrast, is proven: run 36625824357 finished in 2m6s because its push touched only the ticket file, with the docs and record tests still running (repository-contracts green).

Superseded by the table above: the rework branch's green run 36633279293 recorded its times, and main run 36636929043 confirmed them.

## Verdicts

- Code review: ACCEPT 2026-09-29 by a fresh worker-context reviewer, dispatch record 14f4e9e6-7a88-4bcf-9b75-18a44365b073 — same verdict text as ticket 1275's; the review covered this ticket's classify script, guard tests, cache placement and pin, and the nextest checksum script.
- The 2026-09-29 review round's CI findings close only when a fresh reviewer accepts the reworked workflow on this branch.
