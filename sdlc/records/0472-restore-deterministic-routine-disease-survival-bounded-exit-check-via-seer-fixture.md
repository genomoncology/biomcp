---
base: 6b4d03a1477bc740767e1b085b27c4d037942654
head: f778bba8c453829aae7ae0ab36e0f7e8c131f011
---
A 2026-06-30 hotfix relocated the "Disease Survival Commands Exit After Rendering" bounded-exit check from the routine `spec/surface/cli-contract-ratchet.md` into the live lane (`spec/entity/disease.md`) because it asserts live SEER Explorer survival-card landmarks and a live SEER/disease-resolution blip (`MONDO:0011996` "not found") was reddening the routine baseline and blocking the whole biomcp queue. That restored stability but lost the *routine* (deterministic) bounded-exit signal — the check now runs only in `make verify`. Ticket 467's intent was a deterministic routine bounded-exit proof; this ticket restores it by mocking SEER.

Imported from March ticket 472. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/472-restore-deterministic-routine-disease-survival-bounded-exit-check-via-seer-fixture
