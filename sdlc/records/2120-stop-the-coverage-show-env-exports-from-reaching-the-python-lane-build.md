# 2120 — Stop the coverage show-env exports from reaching the Python lane (build record)

Status: implementation complete and pushed (tip 98bf13b60, ci.yml green at
that exact tip); the acceptance coverage run is fully specified and was
queued on the Linux gate host, which the 0.9.3 close-out queue kept busy for
the whole budget window — its verdict is the one open item (see "Acceptance
run" below for the relaunch).

## Claim

`tools/measure-0-9-3-baseline` scopes the coverage build environment to the
suites that compile instrumented artifacts. The `cargo llvm-cov show-env`
exports plus `CARGO_INCREMENTAL` and `CARGO_TARGET_DIR` apply to the
instrumented build and the two Rust suites only; the Python lane and the spec
lane run under the routine environment beside `LLVM_PROFILE_FILE`, so the
cargo-driving Python tests that read those exports pass inside the
instrumentation and the outside-in and combined reports can print numbers
instead of `incomplete`.

## Change

- `COV_BUILD_ENV_KEYS` collects every key the show-env eval exports (plus the
  two pins) while the eval runs.
- The unit suite moved next to the integration suite, so every suite that
  compiles runs while the build environment stands.
- An `unset` of exactly those keys runs before the Python lane, and
  `LLVM_PROFILE_FILE` is re-set to the absolute outside-in pattern; the
  Python lane and spec lane commands themselves are unchanged.
- The report step's `env -u` prefixes stay, now as a guard; the comment
  records that the unset already dropped every eval'd key.

## What show-env actually exports (measured)

`cargo llvm-cov show-env` on the Linux gate host, cargo-llvm-cov 0.9.1
(diagnosis job of 2026-10-17): `LLVM_PROFILE_FILE`,
`__CARGO_LLVM_COV_RUSTC_WRAPPER`,
`__CARGO_LLVM_COV_RUSTC_WRAPPER_RUSTFLAGS`,
`__CARGO_LLVM_COV_RUSTC_WRAPPER_CRATE_NAMES`, `RUSTC_WRAPPER` (pointing at
cargo-llvm-cov itself), `CARGO_LLVM_COV`, `CARGO_LLVM_COV_SHOW_ENV`,
`CARGO_LLVM_COV_TARGET_DIR`, `CARGO_LLVM_COV_BUILD_DIR`; the script adds
`CARGO_INCREMENTAL` and `CARGO_TARGET_DIR`. This cargo-llvm-cov instruments
nested cargo invocations through a rustc wrapper keyed on the workspace crate
names, so any cargo-driving test that inherits the environment builds
instrumented artifacts into the redirected coverage target directory whether
or not `RUSTFLAGS` is set. The `COV_BUILD_ENV_KEYS` mechanism strips every
line show-env prints, so a future key cargo-llvm-cov adds is stripped with
the rest.

## Evidence

- Mechanism (the macOS gate host, pm job 1792205005177-12182 at 43252305f):
  that host has no cargo-llvm-cov, so the eval exported nothing and the
  Python lane passed green (1155 passed, 8 skipped, exit 0) with only
  `CARGO_TARGET_DIR` and `CARGO_INCREMENTAL` set — the failures come from
  the show-env lines themselves, not from the target-dir pin alone.
- Red and green halves at the main tip (the Linux gate host, pm job
  1792205434706-2110573 at 43252305f): after `make prepare-test`, the lane
  ran twice. With the show-env exports evaluated exactly as the script does,
  the lane failed (exit 1) on
  `test_release_smoke_script.py::test_default_release_smoke_rebuilds_stale_target_binary`,
  whose release rebuild lands in the redirected coverage target directory
  while the smoke script watches `target/release/biomcp`; with the eight
  stripped keys the lane passed green (1160 passed, 3 skipped, exit 0). The
  same job printed the show-env key list quoted above. The 26-failure
  evidence at v0.9.2 stands in the 2069 record and this ticket's Evidence
  section; the main tip's tree has fewer cargo-driving failures, and the one
  that remains is the same class.
- Script mechanics: `bash -n` clean; the unset/array constructs exercised
  standalone (all eight keys strip; the empty dry-run shape stays safe under
  `set -u`); `tools/check-tracked-text` passes on the branch.
- CI at the pushed tip: ci.yml run 38998134752 completed success (8m39s) at
  98bf13b60 — the full routine suite green at the exact tip, so no separate
  routine rerun was spent on identical code.

## Acceptance run (open)

The acceptance needs one run of the coverage section at the tip on the Linux
gate host (the only host with cargo-llvm-cov; the ruling kept the coverage
family there for 2082's like-for-like):

    mkdir -p .cache/measure/tmp
    make prepare-spec        # writes .cache/spec-artifacts/* the hand-written spec env reads
    bash tools/measure-0-9-3-baseline . --only host,clone,quiet,coverage

The queued job (1792214312619-2746280, command visible in `pm job list
2120` on that host) wraps exactly this with a 30-minute quiet-wait loop; the
verdict reads from its log: every `RESULT coverage_<suite>` line must show
exit 0 with no failed tests, and `RESULT coverage_outside_in_lines_pct` and
`RESULT coverage_combined_lines_pct` must print numbers, not `incomplete`.
Two earlier attempts of the same job died at the script's own quiet section
while other tickets' `make test` held the host (pm job 1792207008248-2200489
exited 3; the requeue reached position 8 behind up to six concurrent tickets
without starting). Resubmit that command when the host quiets; the
prepare-spec artifacts persist in the ticket's job checkout, so the rerun
pays only the measure itself (~35 minutes).

## Notes

- The measure script's `--only coverage --dry-run` aborts with
  `CARGO_TARGET_DIR: unbound variable` — the same abort exists in the
  pre-change script at 43252305f (verified locally), so it is pre-existing
  and untouched here.
- One edit iteration found and fixed a real trap: after the unset,
  re-exporting `LLVM_PROFILE_FILE` through `$CARGO_TARGET_DIR` would abort
  under `set -u`; the re-export uses the stable `$PWD/$COVT` path instead.
