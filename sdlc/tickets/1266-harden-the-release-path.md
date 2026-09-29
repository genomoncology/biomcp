# Harden the release path

From sdlc/issues/2026-09-28-review-follow-ups-after-1263-and-1264.md
(Release section, bullets three through five).

## Problem

1. The Homebrew tap updates while the GitHub release is still a
   draft: `publish-release` runs after `homebrew-tap`, so the
   formula's tarball URLs point at a draft release until the last
   job succeeds. A failed publish-release leaves `brew install`
   broken until a rerun.
2. The wheel smoke's asset checks claim to "name real content", but
   `skill article-follow-up` and `chart bar` check only the exit
   code; on 0.9.0 both printed errors and exited 0.
3. Nothing asserts the wheel builds pass `--release --locked`. The
   step hash pin is the only guard, and a repin clears it.

## Fix

1. Reverse the order: `publish-release` no longer waits for the tap
   (it keeps waiting for build, PyPI, container, and docs), and
   `homebrew-tap` now waits for `publish-release`. The release is
   public before the formula lands, so the tarball URLs resolve the
   moment `brew install` can see them.
2. The smoke asserts a known skill title in the skill output and a
   known chart heading in the chart output, both from embedded
   content (no network).
3. A structural assertion parses the workflow and requires every
   cargo/maturin build step in `pypi-build` (all matrix variants)
   and `build` to carry `--release` and `--locked`. Proven red on a
   planted copy with `--release` dropped.

## Acceptance

- The workflow-provenance suite passes with the new needs
  expectations, and its needs mutations still go red.
- The new build-flag assertion fails on the planted copy and passes
  on the real workflow.
- The full yellow gate runs on this branch before merge.

## Review

- Design review: n/a (the review file named the fixes)
- Code review: ACCEPT with notes 2026-09-28 (runbook staleness folded; the build-job step pin lands with ticket 1269, which pins every release job)
- Verification: yellow gate green; merged in the 535747d9 chain — yellow gate on this branch before merge (full gate on this branch before merge)
