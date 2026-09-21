---
base: 3111e783
head: 919adb22
---

Published `ghcr.io/genomoncology/biomcp` for `linux/amd64` and `linux/arm64`
from the release workflow, and documented and mechanically pinned the process.

`container-publish` in `.github/workflows/release.yml` runs on
`release: published` and on a `container_only` dispatch. It checks out the
packaging ref, verifies the release's Linux tarballs against their published
sidecars, stages `dist/container/{amd64,arm64}/`, builds one index with buildx,
smokes each platform from the registry, and moves `latest` with
`docker buildx imagetools create` only after both smokes pass and only when
`gh release view` reports the tag as the repository's latest release. The
Dockerfile runtime moved from pinned `debian:bookworm-slim` to pinned
`debian:trixie-slim` (glibc 2.41) because the v0.9.0 executables require
GLIBC 2.39, and the two Linux build legs are pinned to `ubuntu-24.04` so the
binary glibc floor is explicit.

`spec/surface/docker-image.md` pins the release markers, and
`tests/test_release_workflow_provenance.py` pins the dispatch gate, the
packaging ref, the tag revision, both platforms, and the latest guard.
`docs/reference/release-process.md` describes the single workflow and the
container paths, `sdlc/planning/release-0.9-runbook.md` is marked historical,
`AGENTS.md` carries a `## Releases` section, and
`architecture/technical/overview.md` states v0.9.0 and the container job.

Three dispatches proved the path: 35645761982 published the index but failed
the amd64 smoke on GLIBC 2.39 with the bookworm runtime; 35649850201 failed the
same way because the job built with the v0.9.0 tag's bookworm Dockerfile;
35652310359 passed both smokes and moved `latest`. GHCR then served `0.9.0`
and `latest` with identical per-platform manifest digests, `linux/amd64` and
`linux/arm64`, user 65532:65532, version 0.9.0, and revision `a450303872b7`,
the v0.9.0 commit.

Gate at decb67cc on yellow: `make lint` OK, `make spec` OK, `make test` ran
3755/3755 Rust tests green on retry and left only the two pre-existing
`tests/test_source_package_boundary.py` failures from the stale
`MAX_PACKAGE_FILES`, reproduced at 3b1d6452 before this ticket. Gate at the
final SHA 15ac8b6f: `make lint` OK, `make spec` OK, focused provenance tests
5 passed. One Rust nextest flake
(`cache::migration::tests::async_io_crossing_expiry_settles_without_admitting_a_mutation`)
failed once in the first decb67cc run and passed on rerun and 20/20 in
isolation.

Residual: the `release: published` path has not yet run for a real release; the
pre-existing package-boundary failures need their own fix; `release/container.py`
still names the retired bookworm base and is inert. GitHub issue #281 is
answered by the published image; the reply is Ian's.
