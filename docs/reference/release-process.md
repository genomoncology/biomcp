# Release Process

BioMCP publishes from `.github/workflows/release.yml`. A push of a stable tag matching `v*` starts a release. The workflow creates the GitHub release as a draft, validates and publishes the artifacts, then makes the release public last. A manual dispatch only validates an existing tag or republishes its container image.

## Tag-push release

Prepare and commit every public version file and the versioned CHANGELOG section before pushing the tag. Pre-release tags are not supported.

The workflow runs these jobs:

1. `version-check` checks out the tag with full history. `scripts/check-release-versions.py` requires a stable `v`-prefixed tag, compares it with Cargo and Python package versions, and runs `scripts/check-version-sync.sh`. `scripts/check-changelog-coverage.py` derives tickets only from merged `tickets/NNNN-*` branches and requires a descriptive version-section bullet or explicit internal-only bullet for each.
2. `create-draft` creates the GitHub release as a draft. The draft gives asset upload a target without exposing a partial release.
3. `build` compiles the five tarball/zip targets, writes SHA-256 sidecars, and uploads them to the draft with `--clobber` so a failed-job rerun replaces partial assets. The two Linux tarball legs build inside `quay.io/pypa/manylinux_2_28` containers (the ARM64 leg on GitHub's free `ubuntu-24.04-arm` runners, native, no cross toolchain), and the pre-tar binary passes the same glibc 2.28 floor scan the wheels run, so one documented floor covers every Linux artifact: RHEL 8, Debian 10, and Ubuntu 20.04 onward. Linux older than that needs `cargo install`; there is no sdist on purpose (a pip sdist install would require a user-side Rust toolchain).
4. `pypi-build` builds five release-profile wheels: Linux x86_64, Linux ARM64, macOS arm64, macOS x86_64, and Windows x86_64. The Linux wheels build inside `quay.io/pypa/manylinux_2_28` containers — the glibc 2.28 floor is true by construction, maturin pins the `manylinux_2_28` platform tag and the job asserts the produced filename carries it, and a symbol scan enforces the floor against the shipped artifact. The container legs install rustup from the versioned `rustup/archive/1.28.2` URLs with checksum verification; nothing pipes a remote script into a shell.
5. `wheel-smoke` installs every wheel on its native runner and, for Linux, also installs and runs the wheel inside the matching manylinux_2_28 container to prove the floor at runtime: the version banner plus the offline `cache stats --json` deep command, which must print a JSON object. The deep trial and drug commands must exit 0; the absent-FAERS case must return its exact code and text; exit 101, a signal exit, stack-overflow text, malformed JSON, and missing skill assets fail the job.
6. `docs-live` requires the live documentation revision to equal or descend from the tag commit. It retries tag resolution and the public revision pointer.
7. `pypi-publish`, `homebrew-tap`, and `container-publish` run only after their declared validation gates. PyPI uses the protected `pypi` environment and trusted publishing. The tap writes with `HOMEBREW_TAP_TOKEN`.
8. `publish-release` makes the draft public after every publisher succeeds, then moves the GHCR `latest` pointer only when the newly public tag is the repository's latest release.

The GitHub release, its assets, and the container `latest` pointer stay non-public until every check passes. PyPI and the Homebrew tap publish after their own gates and can briefly precede the GitHub release if a later publisher fails. Do not re-run the whole workflow after PyPI succeeds; re-run failed jobs only because version collisions fail loudly.

## Container publication

`container-publish` downloads the two Linux archives and their published sidecars from the draft or existing release, verifies them, and builds one image index for `linux/amd64` and `linux/arm64`. It smokes both registry platforms, checks the non-root user and revision label, and leaves the versioned image in place. `publish-release` moves `latest` only after those smokes and only for the newest public release.

## Container-only dispatch

A manual run takes `tag` and `container_only`. `TAG` comes from the input on dispatch and from `github.ref_name` on a tag push.

With `container_only: true`, only `version-check`, `docs-live`, and `container-publish` run. The container condition requires both gates to succeed and every push-only need to be skipped. This republishes an existing release image without touching its assets, PyPI, the tap, or the GitHub release state.

Without `container_only`, a dispatch runs only `version-check` and `docs-live`; it is a check-only dry run and publishes nothing. PyPI and Homebrew carry explicit push-only gates.

## Documentation publication

The site publishes from pushes to `main`, not from the release workflow. Every deploy writes `https://biomcp.org/__biomcp_revision__/latest.txt`. `docs-live` retries that pointer for up to ten minutes and passes when the live revision equals the tag commit or is a descendant. A behind or divergent revision blocks every publication path.

## Version metadata

Package versions are committed metadata, not values stamped from tags. `scripts/check-version-sync.sh` checks Cargo, Python, lockfiles, `manifest.json`, both `server.json` fields, `CITATION.cff`, and the concrete Homebrew version. `scripts/check-release-versions.py` adds tag equality and stable-tag enforcement. Commit the complete version change before pushing the tag. The private development candidate is Cargo `0.9.1-dev.1` and Python `0.9.1.dev1`; public metadata stays on the latest published release, v0.9.0, until one reviewed commit moves it.

## Permissions and concurrency

The workflow defaults to no token permissions. Each job receives only its declared permission; PyPI alone receives `id-token: write`, container publication receives `packages: write`, and draft/asset publication receives `contents: write`. Release runs share a tag-keyed concurrency group, so a dispatch cannot race a tag push. Third-party actions are pinned where the repository has an admitted immutable revision; the PyPI publisher remains isolated in the protected `pypi` environment.

## Separate manual directory actions

The release workflow does not publish `server.json` to the official MCP Registry or submit BioMCP to third-party directories. After a release, an operator reviews the committed registry metadata and performs each submission separately. Record acceptance before describing any directory as updated.
