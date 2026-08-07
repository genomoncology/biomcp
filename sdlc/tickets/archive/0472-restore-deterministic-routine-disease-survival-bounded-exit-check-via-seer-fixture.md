---
flow: build
priority: 5
---
# Restore deterministic routine disease-survival bounded-exit check via SEER fixture

A 2026-06-30 hotfix relocated the "Disease Survival Commands Exit After Rendering" bounded-exit check from the routine `spec/surface/cli-contract-ratchet.md` into the live lane (`spec/entity/disease.md`) because it asserts live SEER Explorer survival-card landmarks and a live SEER/disease-resolution blip (`MONDO:0011996` "not found") was reddening the routine baseline and blocking the whole biomcp queue. That restored stability but lost the *routine* (deterministic) bounded-exit signal — the check now runs only in `make verify`. Ticket 467's intent was a deterministic routine bounded-exit proof; this ticket restores it by mocking SEER.

Completed under March on 2026-07-01, as March ticket 472. Imported as history when this
repo moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/472-restore-deterministic-routine-disease-survival-bounded-exit-check-via-seer-fixture
