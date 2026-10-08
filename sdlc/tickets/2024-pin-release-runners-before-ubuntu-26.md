# 2024 — Pin release runners before Ubuntu 26

Status: OPEN (reopened 2026-10-07 by the second review).

Milestone: 0.9.2

## Outcome

Every workflow job runs on a pinned image, so the 2026-10-19 move of `ubuntu-latest` to Ubuntu 26 cannot change a release. Released binaries report their build identity. Local spec runs set up the Python test environment that CI uses.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 14). The sixth 0.9.1 review said to tag before 2026-10-19 or pin the runner; 0.9.2 will likely tag after it.

- `.github/workflows/release.yml` lines 514 (pypi-publish), 540 (homebrew-tap) and 609 (container-publish), and `.github/workflows/contracts.yml:9`, use `runs-on: ubuntu-latest`. Every other job uses `ubuntu-24.04`.
- The 0.9.1 wheel and tarball binaries print `biomcp 0.9.1 (git unknown, build unknown)`. The release workflow calls `cargo build` and `maturin build` directly and never `tools/with-build-identity`.
- `make spec` (`Makefile:96-99`) does not run `sync-python-dev`. In a fresh worktree `spec/entity/gene.md:408` fails on a missing `jsonschema`, and the parallel-isolation contracts never run. The developer recorded these as environmental.

- Starts from: the sixth 0.9.1 review's housekeeping list and ticket 1287.
- Keeps: the release job order and the rehearsal-proven steps.
- Changes: pin the four jobs to `ubuntu-24.04`; build release binaries through `tools/with-build-identity`; make `spec` depend on `sync-python-dev`.
- Proof: a workflow test fails on any `ubuntu-latest`; a wheel built by the workflow's command prints a commit; `make spec` in a fresh worktree runs `gene.md:408`.
- Defers: a rehearsal. The release-path change is small and the next real run verifies it under checklist section 4.

## Build status

- Built on branch `tickets/2024-pin-release-runners-before-ubuntu-26`,
  commit c037bddb3, 2026-10-07, after one timeout revival with a
  checkpoint (nothing lost).
- Code review: ACCEPT 2026-10-07. Verified: all four ubuntu-latest
  jobs pinned with job order and steps unchanged; build identity routed
  through with-build-identity on host legs and inherited via
  --env-file on manylinux legs with actionlint checking preserved; the
  ubuntu-latest ban reads whole workflow text with a planted-label red
  proof; make spec syncs Python with a coherent red-first fresh-worktree
  proof; the wheel prints its commit. Scope clarification recorded from
  the review: the outcome line's "every workflow job" means every
  Ubuntu job — macos-latest and windows-latest stay on matrix legs by
  the documented cross-compile choice, and finding 15 scoped this
  ticket to the Ubuntu 26 move. Three report-only P2s: that wording,
  the wheel proof predating the final amend (mechanism unchanged), and
  the container branch lacking its own red test (the real-file
  assertion still catches a dropped --env-file).

## Reopening (second review, 2026-10-07)

Landed early as f6eca26b3 on a green main-merged branch CI
(407940cad) to clear the October 19 date risk; the second review then
found the Windows defect below, so the ticket reopens for the fix,
which lands before the 0.9.2 tag.

- tools/with-build-identity hands off with os.execvpe, which on
  Windows exits 0 at once while cargo keeps running — the step passes
  with no binary. Replace with subprocess.run, pass the exit code
  through, and prove it on a Windows job.
- The release matrix still uses macos-latest and windows-latest while
  the Outcome says every job. Pin them, or narrow the Outcome with a
  recorded reason.
- The review's "no green CI run" names run 37658260567, the branch's
  base-stale run; the main-merged run at 407940cad is green. Recorded
  here so the record carries both.

## Fix round (2026-10-07 reopening)

Built on branch `tickets/2024-runner-windows-fix`.

- tools/with-build-identity now runs the child with subprocess.run
  and exits with the child's code. os.execvpe replaced the wrapper
  process, which on Windows exited 0 at once while the child kept
  running. The env export mode is unchanged: the child still runs
  under the identity environment.
- The windows-2022 CI job runs the wrapper with a trivial cargo
  command and fails unless a failing child exits nonzero through the
  wrapper and a child exit of 42 arrives as 42. Branch CI run
  37698460019 (head d07642eb3) shows windows-contracts green with
  that step; every other job except canonical-gates is green in it.
  canonical-gates cancelled there at its 45-minute cap on the known
  deadline-test deadlock
  (sdlc/issues/2026-10-07-single-backend-deadline-test-can-deadlock.md,
  which records the same cancellations on main that night and this
  branch's hangs, three tests of one module). Attempt 3 of that run
  cleared the deadline tests and failed only on the source package
  file count, recounted in 059c61cb5 (1_402). The branch changes no
  Rust; the green lane evidence is make lint and the Python contracts
  on the build host plus the run's other seven jobs, and a rerun may
  be needed to clear the flake.
- The macos-latest and windows-latest matrix legs in release.yml are
  pinned to macos-15 and windows-2022. This supersedes the
  Ubuntu-only scope clarification above: every workflow job now runs
  on a pinned image, as the Outcome states.
- Contract tests pin the wrapper source, the Windows job's wrapper
  step, and the three pinned matrices, and ban ubuntu-latest,
  macos-latest and windows-latest from workflow text.
