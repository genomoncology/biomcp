---
priority: 1
---

# 2286 — Repair the manylinux wheel rehearsal failure

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision 78e678a, accept

Reviews: revision 2b7f090, accept

## Outcome

An installed 0.9.4 wheel passes the existing manylinux 2.28 smoke on Linux x86-64 and ARM64: exact stable version and successful cache-statistics JSON. Fix the actual cause, preserve real failure detection and complete one bounded outside-in proof before another full untagged rehearsal. No tag or publication.

## Reproduction

Full rehearsal 39418376272 on main cffae4af5fb600039137b7c13d6955165ca46823 passed qualification, the release marker gate and all five archive/wheel builds. Linux wheel-smoke jobs 118632437009 (ARM64) and 118632437209 (x86-64) failed in Run the wheel inside the manylinux 2_28 container. Both installed the wheel and printed biomcp 0.9.4, then exited 1 without printing the cache-statistics result. The failing command is `/opt/python/cp312-cp312/bin/biomcp cache stats --json` captured under bash `-euo pipefail`. This locates the observed failure, not its cause.

Private raw job logs are temporarily in /tmp/biomcp-manylinux-failure-kunfr9_b/118632437009.log and /tmp/biomcp-manylinux-failure-kunfr9_b/118632437209.log. Retain them in an owned numbered experiment before scratch cleanup. Public run: https://github.com/genomoncology/biomcp/actions/runs/39418376272 .

## Scope

Read current main and reproduce with the existing built wheel and exact container command before editing product code. Restate the outcome, record the command/output, name the bounded files and failing outside-in test, and narrow Changes before the fix. Decide from evidence whether this is the wheel, runtime, container environment or smoke command. Keep the genuine installed-artifact check; do not make a failure pass by accepting empty output or ignoring the exit code. No new proof framework or unrelated refactor.

Changes: `src/cache/private.rs`, `tests/test_cache_sqlite.py`, `.github/workflows/release.yml`, `tests/test_release_workflow_provenance.py`, `tests/test_disable_sources_replay_walk.py`, `sdlc/records/2286-manylinux-wheel-rehearsal-failure.md`, `CHANGELOG.md`.

The approved smoke profile disables commercial-restricted sources, KEGG and Semantic Scholar in every wheel-smoke process, including the manylinux container. DDInter sync and interactions must fail with the explicit disabled-source message and exit 1. Live DDInter behavior remains unverified for this release. One replay assertion exercises the article smoke and requires no Semantic Scholar request. Use only the existing read-only public smoke commands, serially with their existing caps and pacing. Set `BIOMCP_DISABLE_SOURCES=commercial-restricted,kegg,semantic-scholar` for every invocation. No keys or new access are authorized.

The initial claim covers diagnosis and its record only. Extend it to the concrete bounded repair files after reproducing the cause. No other product lane is active. Source or workflow changes require fresh review, the existing landing gate, applicable local qualification and a new full untagged rehearsal. Changes to runtime inputs also require the relevant measurement/live evidence bindings to be reassessed; do not discard existing valid baseline data or call providers.

## Proof

Run builds and tests on the M5 through pm job; Yellow is available only for an occasional justified Linux check through pm job. Use the already-built rehearsal wheels when useful. Record one failing preimage and the corrected installed-wheel behavior with exact exit and JSON checks. Run only affected checks. Preserve image digests, source/artifact identity, commands, raw output and cleanup; use docker --rm and both pm labels. Never call KEGG or Semantic Scholar. Only the existing public workload under the approved disabled-source profile is authorized. Do not inspect or retain credentials. One implementation push when ready, fresh review, coordinator alone lands. Keep the failed rehearsal failed.

Keep: experiments/492-biomcp-manylinux-wheel — retain original and corrected wheels, raw logs, scripts, identities and approved smoke evidence while release admission 2258 consumes them. Regenerable source and build copies may be removed after their evidence is retained.

Landing alone: this is the sole blocking runtime and workflow repair. No other accepted product ticket is ready; release admission 2258 waits on its qualification and rehearsal.
