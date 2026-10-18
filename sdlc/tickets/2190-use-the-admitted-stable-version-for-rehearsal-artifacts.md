---
priority: 1
---

# 2190 — Use the admitted stable version for rehearsal artifacts

Status: OPEN.

Milestone: 0.9.3

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
