# Release Process

BioMCP publishes from one workflow, `Release` in
`.github/workflows/release.yml`. The workflow runs when a GitHub release is
published, and an operator can start it by hand to publish only the container
image for a release that is already public. The workflow never creates the
release, the tag, or the public-version commit.

## What a published release runs

A `release` event with `types: [published]` starts five jobs:

- `build` compiles the five shipped targets, packages each artifact, writes a
  `.sha256` sidecar, and uploads both files to the GitHub release. The five
  artifacts are `biomcp-linux-x86_64.tar.gz`, `biomcp-linux-arm64.tar.gz`,
  `biomcp-darwin-arm64.tar.gz`, `biomcp-darwin-x86_64.tar.gz`, and
  `biomcp-windows-x86_64.zip`.
- `pypi-build` builds wheels for Linux x86_64, macOS arm64, macOS x86_64, and
  Windows x86_64 and uploads them as workflow artifacts.
- `pypi-publish` runs after `pypi-build` in the protected `pypi` environment and
  uploads those wheels to PyPI.
- `homebrew-tap` runs after `build`, downloads the published checksums, and
  updates the formula in `genomoncology/homebrew-biomcp`. Without a
  `HOMEBREW_TAP_TOKEN` secret the job logs the skip and exits clean.
- `container-publish` runs after `build` and publishes the container image; the
  next section covers it.

## Container publication

`container-publish` checks out the release tag, downloads the release's two
Linux tarballs, and verifies them against their published sidecars before it
unpacks each `biomcp` executable into the image build context. It then pushes
one image index to `ghcr.io/genomoncology/biomcp:<version>` that carries
`linux/amd64` and `linux/arm64`, assembled from those executables rather than
recompiled in the job.

After the push, the job pulls both platforms back from the registry and runs
`biomcp --version`; the arm64 run goes through QEMU. Both runs must show a
non-root user and an `org.opencontainers.image.revision` label equal to the
tag's commit. Only then does the job move `latest` with
`docker buildx imagetools create`, and only when `gh release view` reports the
tag as the repository's latest release. A backfill for an older release keeps
its versioned tag and leaves `latest` alone. A failed or cancelled `build`
skips the job. If the push succeeds but a smoke fails, the workflow stops,
`latest` stays on the previous image, and the versioned tag holds the
unverified push until a rerun replaces it.

## Container-only dispatch

A manual run takes two inputs. `tag` (required) names the release tag to
publish from. `container_only` (boolean, default `false`) skips `build` and
`pypi-build`.

With `container_only: true`, `pypi-publish` and `homebrew-tap` are skipped
because their `needs` are skipped, so only `container-publish` runs. That path
rebuilds the image for an already-published release, for example to backfill
v0.9.0, and cannot touch PyPI, the release assets, or the tap.

A manual run has no release upload URL, so a dispatch without `container_only`
fails at the `build` job's asset upload. Use `container_only: true` for manual
runs that only need the image.

## Version metadata

Package versions are committed metadata, not values stamped from tags.
`scripts/check-version-sync.sh` checks the mapping from `Cargo.toml`,
`pyproject.toml`, `manifest.json`, both `server.json` version fields,
`CITATION.cff`, and any concrete Homebrew formula version while those files
track the latest reachable stable tag. The private development candidate is
Cargo `0.9.1-dev.1` and Python `0.9.1.dev1`; public metadata stays on the latest
published release, v0.9.0, until one reviewed commit moves it.

## Separate manual directory actions

The release workflow does not publish `server.json` to the official MCP
Registry or submit BioMCP to third-party directories. After a release, an
operator reviews the committed registry metadata and performs each official
submission separately. Record acceptance before describing any directory as
updated.
