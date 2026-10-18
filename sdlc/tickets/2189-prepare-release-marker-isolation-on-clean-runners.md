---
priority: 1
---

# 2189 — Prepare release marker isolation on clean runners

Status: OPEN.

Milestone: 0.9.3

## Outcome

The release marker job prepares the Linux isolation prerequisites before invoking the existing packaged-release gate. Reuse the existing Linux preparation owner so release and qualification jobs do not drift. Keep refusal without isolation, read-only rehearsal permissions and push-only publication.

- Evidence: full rehearsal39081464829 at c91929c9a7ababa9d52574c12fc489a99d8bfc29 passed every qualification job and staged documentation, then release-gate job117541976588 failed immediately in `make release-gate-markers`. Its repo-audit prerequisite entered `tools/run-offline`, which reported `offline routine gates require bubblewrap; refusing to run without network isolation`. Python setup and Cargo fetch had succeeded, but the runner had no Bubblewrap. Archive, wheel and container stages were skipped.
- Changes: claims `.github/workflows/release.yml` `tests/test_release_workflow_provenance.py` `CHANGELOG.md`. Report the existing shared Linux preparation helper and any owned pin/checksum file before editing additional files. Prefer reuse over copied setup. Do not weaken isolation, install optional tools without a named need, or rewrite the workflow.
- Proof: the actual failed job is direct RED. Use the smallest existing workflow/preparation contract to catch an unprepared release marker runner, run affected checks through a build job, then obtain fresh review. Independently run the existing `make release-gate-markers` once on the Linux build machine while this repair develops, reusing exact-source preparation where valid; this checks the previously unreached packaged gate before another full rehearsal. Land the reviewed repair separately through the configured gate, check actual main, then run the complete nonpublishing rehearsal at the repaired candidate.
- Keeps: direct candidate qualification, package cold-compile assertions, repository audit, feature checks, existing command inputs, compiler-cache behavior, manual dispatch and release-only triggers. Product source and existing measured workloads stay unchanged unless a concrete finding justifies a separate repair.

## Reproduction

- Candidate c91929c9a7ababa9d52574c12fc489a99d8bfc29.
- Run https://github.com/genomoncology/biomcp/actions/runs/39081464829 ; release-gate job117541976588, command `make release-gate-markers`, exit2 before marker tests.
- The coordinator retains the complete failed job log in temporary scratch.

## Progress

- 2026-10-17 started
