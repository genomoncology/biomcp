---
priority: 1
---

# 2297 — Resolve installed wheel build dependency failure

Status: OPEN.

Milestone: 0.9.4

## Outcome

Restore the current main full check by identifying and correcting the installed Linux wheel build failure. Keep the production pinned toolchain, dependencies and wheel qualification intact. If the cause is stale build state, remove only the proven regenerable state, record the recovery, and introduce no product change.

## Reproduction

Actual main175c00427685f31c79160bff08d3af6f0d34d5f6 failed `make main-gate` in job2295/1792517340129-5977. The native manylinux wheel stage reports E0463: cannot find crate find_msvc_tools which cc depends on, while compiling the zstd-sys build script. The previous main22255931e6e23b9087ac40f688d3121e31fb1af7 passed the same stage. The source change2295 touched only provider policy resolution and the offline inventory gate, not Cargo or wheel build definitions.

## Proof and stop rule

Before any source change, read the exact failed log and relevant build state, restate the outcome, and establish the narrow cause. Reproduce only the affected wheel stage through pm job. A source fix must have an outside-in failing check and its corrected pass. An environment-only recovery retains the failed original and successful unchanged-main affected stage, then the coordinator runs the full check on actual main. No speculative dependency upgrade, cache framework, new gate, public release or broad investigation. Escalate unresolved material choices to the coordinator. All builds through pm job on the configured host; never use the exclusive performance host while its timing hold remains active.

Changes: `sdlc/records/2297-installed-wheel-build-recovery.md`.
