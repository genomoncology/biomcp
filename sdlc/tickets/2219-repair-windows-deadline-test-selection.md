---
priority: 2
---

# 2219 — Repair Windows deadline test selection

Status: OPEN.

Milestone: 0.9.4

Depends on: 2201

## Outcome

The manual Windows check actually selects the existing body-limit cache-lock deadline case after its production-path rename. Keep the working cache concurrency proof unchanged.

- Evidence: manual Windows run39159496454 at b85880f25dc489c725ab85ad82eb9f6cba66c902 passed, but the body-limit command ran zero tests with4174 filtered out. Existing2199 change dc85d0e95 renamed the case to `body_limit_epoch_lock_contention_preserves_variant_article_time`; the workflow retains its former name. This is a stale test selector, not a reproduced product failure.
- Changes: `.github/workflows/ci.yml` `tests/test_ci_workflow_contract.py` `CHANGELOG.md`. Correct the existing selector and its existing workflow expectation. Keep the fix small; do not introduce a selection framework.
- Proof: reproduce the zero-selection log, update the existing workflow contract, and run the named case on Windows through the approved manual Windows-only check. Record a nonzero passing selection; reuse unchanged concurrency results. No automatic triggers.
