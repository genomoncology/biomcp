---
priority: 1
---

# 2310 — Repair Linux wheel scratch ownership

Status: OPEN.

Milestone: 0.9.4

## Outcome

The existing Linux installed-wheel recipe removes its own temporary build scratch after successful and failed container stages, without permission errors or deletion outside its verified scratch. Preserve the primary build or smoke failure when cleanup also fails. Keep the existing pinned container, build, installation and product smoke behavior.

Changes: `scripts/check-linux-wheel.sh`, `tests/test_installed_wheel_smoke.py`, `CHANGELOG.md`, `sdlc/records/2310-wheel-scratch-ownership.md`.

## Reproduction before code

Gator job 2308/1792530689487-1311653 runs `make installed-wheel-smoke` on the unchanged recipe. Wheel build and installed smoke subprocesses complete, then Python TemporaryDirectory cleanup fails with PermissionError on container-owned target/CACHEDIR.TAG. Overall exit is 2 and stays failed. Bounded owned-scratch cleanup continuation 2308/1792531418898-1375131 passes; its separate receipt does not change that job result. Raw inputs and output remain under experiment 2102-biomcp-2258-final-live-replay/security-pins/2308. The same recipe is already running for 2309.

## Proof

A bounded real Linux ownership control reproduces the current cleanup failure without a cold package rebuild. The repaired command cleans its exact owned scratch after both success and an intentional stage failure, and preserves that original failure. Verify no unrelated path is changed. Reuse actual retained build and smoke receipts accurately; do not relabel their failed aggregate jobs as passing. Run affected tests and the required routine through pm job, then obtain fresh review. The final untagged release rehearsal still must pass the complete packaging workflow.

## Keeps

No general recursive ownership change, new privileges or permission relaxation. Never discard a build or smoke exception. No product, dependency or publication changes. No extra cold build solely to replace an old failed job status.
