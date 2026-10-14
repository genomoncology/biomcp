# 2075 — Build once and run CI lanes in parallel

Status: OPEN.

Milestone: 0.9.3

## Outcome

A full ticket-branch CI run takes the build job plus its longest lane, with a median wall time of at most 22 minutes over ten full runs, down from 25.4 to 27.4 minutes in the 2069 baseline.

## Evidence

- The 2069 baseline's full runs: wall median 27.4 then 25.4 minutes. The 2070 branch run 38545958060 took 28.8 minutes; inside canonical-gates, lint took 2.2, test 12.8 and spec 11.0 minutes, in series. release-panic took 13.9 to 20.0 minutes for one test. stress-lane rebuilds the nextest archive.
- The routine gates use `--no-default-features`, so only `full-features` checks AlphaGenome and the PNG charts (AGENTS.md, "make full-feature-check"). Moving it off ticket branches would let 2077's tonic upgrade land unchecked.
- PM decision 2026-10-14: the target is wall time only. The repository is public, so runner-minutes cost nothing. Only release-panic moves to main-only; full-features and windows-contracts stay on every push, because full-features is the only check of AlphaGenome and windows runs in parallel and is shorter than the longest lane. No split: the remaining work is one change.

- Starts from: 2070's Makefile, where `prepare-test` builds the nextest archive, `target/debug/biomcp` and the `rmcp_streamable_http_contract` example in one dev build; the `canonical-gates`, `stress-lane` and `release-panic` jobs in `.github/workflows/ci.yml`.
- Keeps: every check CI runs today; `full-features`, `windows-contracts`, `generated-sources` and `repository-contracts` on every push; the `changes` job and its docs-only skip, including 2086's route.
- Changes: claims `.github/workflows/ci.yml` `.github/actions/**` `tests/test_ci_workflow_contract.py` `tests/test_offline_gate_contract.py` `tests/test_python_contract_parallelism.py` `tests/test_routine_cargo_feature_contract.py` `tests/test_upstream_planning_analysis_docs.py` `tests/test_alphagenome_proto_generation.py` `architecture/technical/overview.md` `Makefile` `CHANGELOG.md`.
  - A `build` job runs `make prepare-test` and uploads the archive, `target/debug/biomcp` and the example binary as one artifact.
  - The `lint`, `rust-tests`, `python-lane` and `spec-lane` jobs need `build`, download the artifact and run in parallel. Each calls a Makefile target, so `make test` stays the developer's one command: split `test` into `test-rust` and `test-python`.
  - One composite action under `.github/actions/` holds the shared setup steps (toolchain, uv, bubblewrap and AppArmor, nextest).
  - stress-lane uses the uploaded archive instead of `make prepare-test`.
  - release-panic runs only on pushes to `main`. The tag needs green CI on its commit, so the check still gates every release.
  - `tests/test_alphagenome_proto_generation.py` asserts `setup-protoc` appears exactly twice in `ci.yml`; keep protoc out of the new lanes or update that count.
  - One Internal changelog bullet.
- Proof: wall time from the Actions API for ten full ticket-branch runs after landing, compared with the 2069 baseline's table; one run where a planted failing Rust test fails `rust-tests` while the other lanes still report.
- Defers: shortening the Python lane (2071), the lint step (2076) and the spec lane (2080).
- Order: after 2070, which builds the single dev binary these lanes share. Right after 2086, which also edits `.github/workflows/ci.yml`; before 2074 and 2076, which edit the same file. It edits the `Makefile` `test` target that 2071 also edits, so 2075 lands first and 2071 rebases onto it. 2073 and 2072 rebase onto it for `tests/test_upstream_planning_analysis_docs.py` and `tests/test_alphagenome_proto_generation.py`.
