---
priority: 1
---

# 2297 — Resolve installed wheel build dependency failure

Status: OPEN.

Milestone: 0.9.4

## Outcome

Restore the current main full check by identifying and correcting the installed Linux wheel build failure. Keep the production pinned toolchain, dependencies and wheel qualification intact. After the unchanged-main retry passes, apply the PM-directed prevention: each wheel build uses its own clean run directory, which the existing runner removes afterward. Keep any causal claim limited to transient retained-build state rather than corrupt dependency contents.

## Reproduction

Actual main175c00427685f31c79160bff08d3af6f0d34d5f6 failed `make main-gate` in job2295/1792517340129-5977. The native manylinux wheel stage reports E0463: cannot find crate find_msvc_tools which cc depends on, while compiling the zstd-sys build script. The previous main22255931e6e23b9087ac40f688d3121e31fb1af7 passed the same stage. The source change2295 touched only provider policy resolution and the offline inventory gate, not Cargo or wheel build definitions.

## Proof and stop rule

Before any source change, read the exact failed log and relevant build state, restate the outcome, and establish the narrow cause. Reproduce only the affected wheel stage through pm job. A source fix must have an outside-in failing check and its corrected pass. An environment-only recovery retains the failed original and successful unchanged-main affected stage, then the coordinator runs the full check on actual main. No speculative dependency upgrade, cache framework, new gate, public release or broad investigation. Escalate unresolved material choices to the coordinator. All builds through pm job on the configured host; never use the exclusive performance host while its timing hold remains active.

Changes: `scripts/check-linux-wheel.sh`, `tests/test_installed_wheel_smoke.py`, `sdlc/records/2297-installed-wheel-build-recovery.md`, `CHANGELOG.md`.

## Progress

- 2026-10-20 started

## PM ruling and pre-code plan

The PM directs finishing the already running M5 retry. If unchanged main passes the formerly failing step, treat the failure as transient build-cache state and make the wheel check use its own clean target. The target currently persists across fresh containers even though their registry and toolchain are recreated; Cargo.lock is unchanged. Identical retained dependency hashes do not prove corrupt bytes.

Before source edits, the developer restates the prevention as local run isolation: move CARGO_TARGET_DIR from the persistent .cache/installed-wheel/<target>/target into the runner's existing TemporaryDirectory beside wheels. Strengthen the fake-docker outside-in case to assert the build target is the run's wheel parent/target, starts absent, can be created and is removed by runner cleanup. This assertion must fail with the persistent target, then pass with the bounded script change. No workflow, toolchain, dependency or smoke workload changes.
