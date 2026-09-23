# Release dispatch publishes for any tag

Filed 2026-09-23 from an independent review of the 1.0 line's merge of main.

- `.github/workflows/release.yml` accepts `workflow_dispatch` from any branch with a caller-named tag.
- The `homebrew-tap` and `container-publish` jobs run on that dispatch. Neither checks for a stable tag or the default branch. Only `container_only` and a missing Homebrew token stop them.
- A dispatch naming a pre-release tag with a GitHub release would push a `ghcr.io/genomoncology/biomcp` image and update the Homebrew formula. `latest` moves only when the tag is the latest release.
- PyPI is safe. Its job runs only on a published release, in the protected `pypi` environment.
- Fix: both jobs refuse a tag that is not a stable release version.
