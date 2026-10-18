---
priority: 1
---

# 2190 — Use the admitted stable version for rehearsal artifacts

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 779b069, accept

## Outcome

A full untagged rehearsal builds archives and wheels with the admitted stable release version while retaining their actual candidate revision. Installed-wheel and local-container smokes validate the release payload without needing a tag. Ordinary development identity and tag publication behavior stay intact.

- Evidence: PM-directed downstream review at cc39ee28a2d02a4fa329b1ebbdc76de9d6882568 found that archive and wheel builds invoke `tools/with-build-identity` without a rehearsal version override. At an untagged candidate it emits0.9.3+gcc39ee28. All installed-wheel checks require exactly `biomcp 0.9.3`, and local-container smoke requires the same banner. The Docker VERSION argument labels the image; it does not change the embedded binary. The admitted rehearsal input already requires a stable version matching committed metadata.
- Changes: claims `.github/workflows/release.yml` `tests/test_release_workflow_provenance.py` `CHANGELOG.md`. Report the existing build-identity contract and any additional file before editing. Prefer the existing explicit build-version override and admitted input; do not create tags, alter committed versions, lie about source revision, weaken smoke assertions or change production publication.
- Proof: reproduce the current untagged identity mismatch through the existing wrapper/contract in a build job. Use the smallest existing artifact/workflow identity checks to require the stable rehearsal banner, real candidate revision, ordinary development suffix and unchanged tag behavior. Run affected checks, fresh review, configured landing gate and actual-main check. Then require a complete manual nonpublishing rehearsal of the repaired candidate, including wheel and local-container smokes.
- Keeps: full-SHA admission, stable metadata equality, read-only rehearsal permissions, push-only publication, release-only triggers, actual source identity and compiler-cache behavior. No broad version-system rewrite or performance recollection.

## Coordination

The current full rehearsal39083335506 continues at cc39ee28a2d02a4fa329b1ebbdc76de9d6882568 under the PM direction to run late findings alongside. Main stays fixed while this repair develops and is reviewed. The coordinator reports its result and this confirmed source finding to PM before the next landing.

## Progress

- 2026-10-17 started

## Implementation and checks

The validated admission step exports its stable version. Archive and wheel jobs pass that value through the existing build identity override; native builders and both existing manylinux environment-file paths retain the actual source revision. Ordinary development identity and tagged publication keep their existing behavior. No wrapper, committed version, publication condition or smoke assertion changed.

Developer revision: `0f6f6a0623c06b846d02fb950d48ef6bf9c1c210`. Build-job checks used base `0455496f59c412e03b23655b043ac4b9c3b8fb9a` plus inline patches.

- RED: job `1792284057571-93906` ran `tools/with-build-identity env` and the four native/manylinux archive/wheel regression cases. The real source wrapper emitted `0.9.3+g0455496f`, with actual revision `0455496f`; all four regression cases failed on stable-version mismatch.
- GREEN: job `1792284093917-38236` ran `pytest tests/test_release_workflow_provenance.py tests/test_build_identity_wrapper_contract.py tests/test_build_identity_rebuild.py tests/test_release_versions_check.py -q`: 178 passed. Ruff and tracked-text checks passed; workflow lint then stopped because its pinned tools were missing. Job `1792284132685-59010` bootstrapped those tools and passed workflow lint and the quality ratchet.
- Final test-reader adjustment restricts output to identity variables. Job `1792284174889-66455` passed the four affected cases, Ruff, tracked-text, workflow lint and quality ratchet. Its tested patch SHA-256 and committed diff both equal `dde81d1a7aec10bc6a24b0e17032aa842bcfac702640183d0cc62c645403c706`.

The coordinator inspected the diff, job outputs and matching final patch hash. Tests replace compilation with identity readers; existing rebuild contracts cover embedded identity. Review, configured landing gate, actual-main check and the full repaired-candidate rehearsal remain required. Previous timing and coverage observations retain their measured source labels; this workflow repair does not relabel them.
