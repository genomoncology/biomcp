# Release does not check the tag against the committed version

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`. Blocks 0.9.1.

## Symptom

`.github/workflows/release.yml` never compares `${TAG#v}` with the version in `Cargo.toml` or `pyproject.toml`. `scripts/check-version-sync.sh` runs only in `ci.yml`, on pushes and pull requests. No release job depends on it.

Main is at `0.9.1-dev.1` (Cargo) and `0.9.1.dev1` (Python). If v0.9.1 is published before the version bump commit:

- PyPI permanently receives `biomcp-cli 0.9.1.dev1`.
- GHCR `:0.9.1` holds a binary that reports `0.9.1-dev.1`, and `latest` moves to it.
- The Homebrew formula says `0.9.1` and installs the dev binary.

## Reproduction

```
grep -n 'version-sync\|Cargo.toml' .github/workflows/release.yml   # no match
grep -n '^version' Cargo.toml pyproject.toml
```

## Fix

Add a `version-check` job that fails unless `${TAG#v}` equals the Cargo version and the matching Python version. Make `build`, `pypi-build`, and `docs-live` need it. Add a test that removes the job and fails.
