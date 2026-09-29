# Release dispatch publishes for any tag

Filed 2026-09-23 from an independent review of the 1.0 line's merge of main.

- `.github/workflows/release.yml` accepts `workflow_dispatch` from any branch with a caller-named tag.
- The `homebrew-tap` and `container-publish` jobs run on that dispatch. Neither checks for a stable tag or the default branch. Only `container_only` and a missing Homebrew token stop them.
- A dispatch naming a pre-release tag with a GitHub release would push a `ghcr.io/genomoncology/biomcp` image and update the Homebrew formula. `latest` moves only when the tag is the latest release.
- PyPI is safe. Its job runs only on a published release, in the protected `pypi` environment.
- Fix: both jobs refuse a tag that is not a stable release version.

## Resolved

Fixed by ticket 1234's release-gates rework: `homebrew-tap` and
`container-publish` (and every publishing job) carry
`if: github.event_name == 'push'`, so a workflow_dispatch naming any
tag can no longer publish a container or touch the formula; the
dispatch path that remains is the deliberate `container_only`
rebuild for an already-published release. The push path itself is
guarded by the version check's stable-tag rule
(scripts/check-release-versions.py). See
`sdlc/records/1234-rework-the-release-gates-so-v0-9-1-can-pass-them.md`.
