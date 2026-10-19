---
priority: 2
---

# 2219 — Repair Windows deadline test selection

Status: WITHDRAWN.

Opened as: 2026-10-18, superseded by 2225.

Milestone: 0.9.4

Depends on: 2201

Superseded by: 2225

## Outcome

The manual Windows check actually selects the existing body-limit cache-lock deadline case after its production-path rename. Keep the working cache concurrency proof unchanged.

- Evidence: manual Windows run39159496454 at b85880f25dc489c725ab85ad82eb9f6cba66c902 passed, but the body-limit command ran zero tests with4174 filtered out. Existing2199 change dc85d0e95 renamed the case to `body_limit_epoch_lock_contention_preserves_variant_article_time`; the workflow retains its former name. This is a stale test selector, not a reproduced product failure.
- Changes: `.github/workflows/ci.yml` `tests/test_ci_workflow_contract.py` `CHANGELOG.md`. Correct the existing selector and its existing workflow expectation. Keep the fix small; do not introduce a selection framework.
- Proof: reproduce the zero-selection log, update the existing workflow contract, and run the named case on Windows through the approved manual Windows-only check. Record a nonzero passing selection; reuse unchanged concurrency results. No automatic triggers.

## Superseding proof

PM approved withdrawal after2225 landed as2a711534e4f4d588f1b2be6f48f3f3cd16cd864a. The selector and definition guard are included there. Actual Windows39167643933 atbb799c1b3de8a09b8d59c536a1fbdd978e0d2e20 selected body_limit_epoch_lock_contention_preserves_variant_article_time and reported1passed,0failed,in0.01s. Fresh component review confirmed the guard semantics and disabled automatic triggers.2225 also records the verified released/current cache comparison and its limits; no extra Windows run or duplicate configuration landing is needed.
