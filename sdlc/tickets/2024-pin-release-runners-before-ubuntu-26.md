# 2024 — Pin release runners before Ubuntu 26

Status: OPEN.

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
