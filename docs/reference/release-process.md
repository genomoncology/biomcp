# Release Process

BioMCP publishes from one workflow, `Release` in
`.github/workflows/release.yml`. The workflow runs when a GitHub release is
published, and an operator can start it by hand to publish only the container
image for a release that is already public. The workflow never creates the
release, the tag, or the public-version commit.

## What a published release runs

A `release` event with `types: [published]` starts seven jobs:

- `build` compiles the five shipped targets, packages each artifact, writes a
  `.sha256` sidecar, and uploads both files to the GitHub release with
  `gh release upload`. The upload step runs only for a `release` event, so a
  manual dispatch stops after packaging. The five artifacts are
  `biomcp-linux-x86_64.tar.gz`, `biomcp-linux-arm64.tar.gz`,
  `biomcp-darwin-arm64.tar.gz`, `biomcp-darwin-x86_64.tar.gz`, and
  `biomcp-windows-x86_64.zip`.
- `pypi-build` builds wheels for Linux x86_64, macOS arm64, macOS x86_64, and
  Windows x86_64 and uploads them as workflow artifacts. Every wheel builds with
  `args: --release --locked`, so the wheel binary carries the same release
  profile as the tarball executables instead of a dev build with unoptimized
  frames.
- `wheel-smoke` runs after `pypi-build`, installs the Linux x86_64 wheel into a
  virtual environment outside the checkout, and runs `search trial`,
  `drug interactions`, and `drug trials` through the installed `biomcp` binary.
  It fails on SIGABRT, on a stack-overflow message, and on any other crash, so a
  dev-profile wheel cannot reach PyPI. A clean source error still passes.
- `docs-live` runs on both triggers, resolves the tag's commit, and reads the
  pointer `https://biomcp.org/__biomcp_revision__/latest.txt` with no-cache
  request headers and a fresh cache-busting query on every attempt. It retries
  for up to ten minutes while the site deploy and the Pages cache catch up, and
  it passes when the live revision equals the tag's commit or is a descendant
  of it. A live revision behind or divergent from the tag fails the job.
- `pypi-publish` runs after `pypi-build`, `wheel-smoke`, and `docs-live` in the
  protected `pypi` environment and uploads those wheels to PyPI.
- `homebrew-tap` runs after `build` and `docs-live`, downloads the published
  checksums, and updates the formula in `genomoncology/homebrew-biomcp`. Without
  a `HOMEBREW_TAP_TOKEN` secret the job logs the skip and exits clean.
- `container-publish` runs after `build` and `docs-live` and publishes the
  container image; the next section covers it.

## Container publication

`container-publish` checks out the packaging ref, which is the workflow's own
ref rather than the release tag. A `release: published` event runs from the tag,
so the packaging is the tag's; a manual dispatch runs from the ref it was
started on, so main's `Dockerfile` and `.dockerignore` build the tag's content.
The job downloads the release's two Linux tarballs and verifies them against
their published sidecars before it unpacks each `biomcp` executable into the
image build context, and it reads the revision label from the tag's commit with
`gh api`. It then pushes one image index to
`ghcr.io/genomoncology/biomcp:<version>` that carries `linux/amd64` and
`linux/arm64`, assembled from those executables rather than recompiled in the
job.

After the push, the job pulls both platforms back from the registry and runs
`biomcp --version`; the arm64 run goes through QEMU. Both runs must show a
non-root user and an `org.opencontainers.image.revision` label equal to the
tag's commit. Only then does the job move `latest` with
`docker buildx imagetools create`, and only when `gh release view` reports the
tag as the repository's latest release. A backfill for an older release keeps
its versioned tag and leaves `latest` alone. A failed or cancelled `build`
skips the job, and so does a failed `docs-live`. If the push succeeds but a
smoke fails, the workflow stops, `latest` stays on the previous image, and the
versioned tag holds the unverified push until a rerun replaces it.

## Container-only dispatch

A manual run takes two inputs. `tag` (required) names the release tag to
publish from. `container_only` (boolean, default `false`) skips `build` and
`pypi-build`.

With `container_only: true`, `wheel-smoke`, `pypi-publish`, and `homebrew-tap`
are skipped because their `needs` are skipped. `docs-live` has no
`container_only` gate, so it still runs and `container-publish` waits for it: a
backfill also requires the live site to be at or past the tag's commit. That
path rebuilds the image for an already-published release, for example to
backfill v0.9.0, and cannot touch PyPI, the release assets, or the tap.

A dispatch without `container_only` builds and packages the artifacts but
uploads nothing, because the upload step is guarded on the `release` event. The
run never reaches PyPI: `pypi-publish` is gated on the `release` event as well,
so a dispatch skips it on every input. It fails at `docs-live` when the site
has not reached that tag's commit. When `docs-live` passes, `homebrew-tap` and
`container-publish` still run, so a dispatch that names an older tag rewrites
the public Homebrew formula backwards and republishes that tag's image; the old
early failure at the asset upload stopped a dispatch before either job. Use
`container_only: true` for manual runs that only need the image.

## Documentation publication

The site publishes from pushes to `main`, not from the release event. The
release commit lands on `main` before the release is published, so its push
runs `Publish documentation`: it builds the site strictly from that exact SHA,
deploys it to the `gh-pages` branch, requests a Pages build, and verifies the
live revision witness `https://biomcp.org/__biomcp_revision__/<sha>.txt` and
the published Markdown bytes against the local build. Confirm that run
succeeded for the release SHA before announcing the release. The site is edge
documentation and tracks `main`, not the latest tag.

Every deploy also rewrites the pointer at
`https://biomcp.org/__biomcp_revision__/latest.txt` with the same SHA, so the
newest deploy stays readable after a later push removes the per-revision file.
The `docs-live` job reads that pointer and retries for up to ten minutes while
the deploy and the Pages cache catch up. The job passes when the live revision
equals the tag's commit or is a descendant of it, and fails when the live
revision is behind or divergent. A release therefore stops before PyPI, the
tap, and the container image publish against stale documentation.

## Version metadata

Package versions are committed metadata, not values stamped from tags.
`scripts/check-version-sync.sh` checks the mapping from `Cargo.toml`,
`pyproject.toml`, `manifest.json`, both `server.json` version fields,
`CITATION.cff`, and any concrete Homebrew formula version while those files
track the latest reachable stable tag. The private development candidate is
Cargo `1.0.0-dev.1` and Python `1.0.0.dev1`; public metadata stays on the latest
published release, v0.9.0, until one reviewed commit moves it.

## Separate manual directory actions

The release workflow does not publish `server.json` to the official MCP
Registry or submit BioMCP to third-party directories. After a release, an
operator reviews the committed registry metadata and performs each official
submission separately. Record acceptance before describing any directory as
updated.
