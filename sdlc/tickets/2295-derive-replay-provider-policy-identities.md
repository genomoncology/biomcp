---
priority: 1
---

# 2295 — Derive replay provider policy identities

Status: OPEN.

Milestone: 0.9.4

Depends on: 2293

## Outcome

The replay inventory identifies the existing source policy for every approved recorded provider using the existing canonical source identity. Correct the HPA, HPO and Monarch lookup gaps without changing licensing policy, weakening restrictions or treating unknown providers as allowed. Reuse one shared identity resolver across callers; do not create a second list of policy facts.

Changes: `scripts/replay_drift.py`, `tools/check-replay-live`, `tests/test_replay_live_drift.py`, `sdlc/records/2295-provider-policy-identities.md`, `CHANGELOG.md`.

## Evidence

The recorded-command combined job2236/1792514132327-30644 passed232checks, then its offline inventory reported107total,92ready,10disabled,5blocked. The five blockers are one hpa, two hpo and two monarch routes with provider-has-no-source-policy. These are already approved public captures. The earlier PMC naming gap repaired in2293 belongs to the same identity duplication class. This ticket removes that remaining lookup cause rather than changing provider access rules.

## Proof

Before code, reproduce these exact five inventory dispositions through a focused controlled offline case against accepted2293. Resolve their existing canonical policies from repository truth, preserve unknown/refused/restricted controls and MIME qualification scopes, and show the corrected offline inventory without public calls. Reuse the completed232check coverage proof; do not rerun the full golden suite. Run affected identity/checker controls and mechanical checks through pm job. No new parser, live capture or permission framework.

Landing alone: this changes the release qualification checker dependency and therefore takes its permitted separate gate.
