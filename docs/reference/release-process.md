# Release Process

BioMCP publishes from `.github/workflows/release.yml`. A push of a stable tag matching `v*` starts a release. The workflow creates the GitHub release as a draft, validates and publishes the artifacts, then makes the release public last. A manual dispatch can validate an existing tag, republish its container image, or rehearse an untagged candidate without publishing.

## Tag-push release

Prepare and commit every public version file and the versioned CHANGELOG section before pushing the tag. Pre-release tags are not supported.

The workflow runs these jobs:

1. `version-check` checks out the tag with full history. `scripts/check-release-versions.py` requires a stable `v`-prefixed tag, compares it with Cargo and Python package versions, and runs `scripts/check-version-sync.sh`. `scripts/check-changelog-coverage.py` derives tickets from `Land NNNN:` landing subjects, old-pattern `Merge ... tickets/NNNN-` subjects, ticket-numbered `sdlc/records/` files, and ticket files whose status reads complete at the tag, all added in the range and counted once by number, and requires a descriptive version-section bullet or explicit internal-only bullet for each. The `qualification` job invokes the reusable CI workflow on that exact revision, including all shipped-feature, long and platform lanes. No prior branch or main push run is required. Everyday gates run through build jobs; GitHub qualification runs only for release work.
2. `create-draft` creates the GitHub release as a draft. The draft gives asset upload a target without exposing a partial release.
3. `release-gate` runs the release-gate marker set (`make release-gate-markers`, the `-m release_gate` pytest selection whose centerpiece is the packaged-crate cold compile) before any archive is built. `build` and `pypi-build` wait on this job (2125), so no tarball, wheel or zip is built before the marker set passes.
4. `build` compiles the five tarball/zip targets, writes SHA-256 sidecars, and retains them as internal run artifacts. A separate push-only `publish-assets` job uploads them to the draft with `--clobber`, so builders need only read permissions and a failed-job rerun replaces partial assets. The two Linux tarball legs build inside `quay.io/pypa/manylinux_2_28` containers (the ARM64 leg on GitHub's free `ubuntu-24.04-arm` runners, native, no cross toolchain), and the pre-tar binary passes the same glibc 2.28 floor scan the wheels run, so one documented floor covers every Linux artifact: RHEL 8, Debian 10, and Ubuntu 20.04 onward. Linux older than that needs `cargo install`; there is no sdist on purpose (a pip sdist install would require a user-side Rust toolchain).
5. `pypi-build` builds five release-profile wheels: Linux x86_64, Linux ARM64, macOS arm64, macOS x86_64, and Windows x86_64. The Linux wheels build inside `quay.io/pypa/manylinux_2_28` containers — the glibc 2.28 floor is true by construction, maturin pins the `manylinux_2_28` platform tag and the job asserts the produced filename carries it, and a symbol scan enforces the floor against the shipped artifact. The container legs install rustup from the versioned `rustup/archive/1.28.2` URLs with checksum verification; nothing pipes a remote script into a shell.
6. `wheel-smoke` installs every wheel on its native runner and, for Linux, also installs and runs the wheel inside the matching manylinux_2_28 container to prove the floor at runtime: the version banner plus the offline `cache stats --json` deep command, which must print a JSON object. The deep trial and drug commands must exit 0; the absent-FAERS case must return its exact code and text; exit 101, a signal exit, stack-overflow text, malformed JSON, and missing skill assets fail the job.
7. A tag release invokes the reusable documentation publisher for the tagged revision before `docs-live` requires the live documentation revision to equal or descend from the tag commit. It retries tag resolution and the public revision pointer.
8. `pypi-publish` and `container-publish` run only after their declared validation gates. PyPI uses the protected `pypi` environment and trusted publishing. The tap writes with `HOMEBREW_TAP_TOKEN` and waits for `publish-release`, so the formula can only point at a public release.
9. `publish-release` runs its Docker work first — the GHCR `latest` move, decided by `scripts/should-move-latest.sh` against the published releases only (exit 3 means a legitimate stay) — and only then makes the draft public with `gh release edit --draft=false`. A failed Docker step therefore leaves a deletable draft, and `latest` never moves after the release is public.

The GitHub release, its assets, and the container `latest` pointer stay non-public until every check passes. PyPI can briefly precede the GitHub release if a later publisher fails; the Homebrew tap cannot — it runs only after the release is public. Do not re-run the whole workflow after PyPI succeeds; re-run failed jobs only because version collisions fail loudly.

## Container publication

`container-publish` downloads the two Linux archives and their published sidecars from the draft or existing release, verifies them, and builds one image index for `linux/amd64` and `linux/arm64`. It smokes both registry platforms, checks the non-root user and revision label, and leaves the versioned image in place. `publish-release` moves `latest` only after those smokes and only for the newest public release.

## Container-only dispatch

A manual run takes `tag` and `container_only`. `TAG` comes from the input on dispatch and from `github.ref_name` on a tag push.

With `container_only: true`, `version-check`, direct `qualification`, `docs-live`, and `container-publish` run. The container condition requires both gates to succeed and every push-only need to be skipped. This republishes an existing release image without touching its assets, PyPI, the tap, or the GitHub release state.

With only `tag` and without `container_only`, a dispatch runs `version-check`, direct `qualification` and `docs-live`; it is a check-only dry run and publishes nothing. PyPI and Homebrew carry explicit push-only gates.

## Full untagged rehearsal

Complete local release checks and commit stable release metadata first. Choose the final candidate's full forty-character commit SHA and intended stable version. Dispatch `release.yml` **from that exact commit**, with `candidate_sha` set to that SHA and `rehearsal_version` set to the stable version (for example, `0.9.3`). Leave `tag` empty and `container_only` false. The admission helper refuses shortened SHAs, a workflow definition from another revision, mismatched committed versions, prerelease versions, and conflicting modes. It creates no tag.

The rehearsal runs direct qualification, the packaged-crate marker gate, all five production archive and wheel builds, installed-wheel smokes, strict documentation generation with exact revision witnesses, both container platforms through the production Dockerfile with local smokes, release-note rendering, wheel metadata checks, archive checksums, and Homebrew formula rendering and syntax validation. Internal run artifacts retain the archives, wheels, generated site, local container images and publication payloads. `rehearsal-complete` passes only after every required lane succeeds.

Every rehearsal job has read-only or empty permissions. Publisher jobs cannot run in this mode, and no publication credential or trusted-publishing identity is granted. GitHub release creation/upload/publication, PyPI trusted upload, registry authentication/pushes, Pages deployment and live-site verification, the Homebrew push, and moving `latest` remain unexercised. A rehearsal validates their prepared payloads and build paths; actual publication still requires its own successful run.

Run this complete manual rehearsal on the final candidate after every release ticket and final measurement record has landed. Record its run URL and SHA before asking Ian to tag. A later source or workflow change requires another full rehearsal of the new candidate.

## Documentation publication

The site publishes during a tag release through reusable `docs-edge.yml`, before live revision verification. Its manual main dispatch remains available for an explicit publication request. Branch and main pushes do not publish. The workflow checks out the selected exact revision, strictly builds it, preserves Pages history, and serializes all publication runs. It reads the existing revision witness and refuses divergent history. If the existing site is newer, it retains that site; an older release cannot overwrite newer documentation. Every actual deploy writes `https://biomcp.org/__biomcp_revision__/latest.txt`. `docs-live` retries that pointer for up to ten minutes and passes when the live revision equals the tag commit or is a descendant. A behind or divergent revision blocks every publication path.

## Version metadata

Package versions are committed metadata, not values stamped from tags. `scripts/check-version-sync.sh` checks Cargo, Python, lockfiles, `manifest.json`, both `server.json` fields, `CITATION.cff`, and the concrete Homebrew version. `scripts/check-release-versions.py` adds tag equality and stable-tag enforcement, so the release workflow refuses a tag whose own name carries `-dev` and refuses a stable tag while any committed version still does. Commit the complete version change before pushing the tag. Main carries the next development version between releases — Cargo `0.9.3-dev` and Python `0.9.3.dev0` while 0.9.3 work lands — and a build from main reports `0.9.3-dev+g<sha>` instead of the last released number; the release commit drops the dev suffix and commits the released number everywhere, while registry, citation, and Homebrew metadata stay on the latest public release until their own reviewed commit moves them.

## Permissions and concurrency

The workflow defaults to no token permissions. Each job receives only its declared permission; PyPI alone receives `id-token: write`, container publication receives `packages: write`, and draft/asset publication receives `contents: write`. Release runs share a tag- or candidate-keyed concurrency group, so a dispatch cannot race a tag push. Third-party actions are pinned where the repository has an admitted immutable revision; the PyPI publisher remains isolated in the protected `pypi` environment.

## Separate manual directory actions

The release workflow does not publish `server.json` to the official MCP Registry or submit BioMCP to third-party directories. After a release, an operator reviews the committed registry metadata and performs each submission separately. Record acceptance before describing any directory as updated.
