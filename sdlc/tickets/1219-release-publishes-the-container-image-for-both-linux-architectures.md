---
flow: build
priority: 3
deps: []
---

# 1219: Release publishes the container image for both Linux architectures

## Goal

Every published release pushes `ghcr.io/genomoncology/biomcp:<version>` and moves `latest`, with `linux/amd64` and `linux/arm64` in the same image index, assembled from that release's own Linux executables. Today no release job touches GHCR: `7f365191` deleted `docker-publish` from `.github/workflows/release.yml` on 2026-09-16, so 0.9.0 published no image. GHCR still serves only `0.8.25` and `latest`, both the same single-platform amd64 digest, and `docker pull ghcr.io/genomoncology/biomcp` fails on Apple Silicon with `no matching manifest for linux/arm64/v8` (issue #281). A `workflow_dispatch` with `container_only: true` must rebuild the image for an already-published release, starting with v0.9.0.

## Current Facts

- `.github/workflows/release.yml` has jobs `build`, `pypi-build`, `pypi-publish`, `homebrew-tap` and no container step; the top-level `packages: write` permission is unused (release.yml:13-16).
- `build` stages release assets in the repo root and uploads them to the GitHub release with `actions/upload-release-asset@v1` (release.yml:60-110). That action needs `github.event.release.upload_url`, which is empty on `workflow_dispatch`, so a dispatch cannot rerun `build`.
- `Dockerfile` copies `dist/container/${TARGETARCH}/biomcp` and `dist/container/${TARGETARCH}/biomcp.sha256`, runs `sha256sum -c`, and runs as UID 65532 (Dockerfile:16-18, 31). Buildx supplies `TARGETARCH` as `amd64` or `arm64`.
- `.dockerignore` admits only `Dockerfile` and `dist/container/**`; `spec/surface/docker-image.md` pins both, and its prose says the image is assembled from the two Linux executables and does not compile source.
- The v0.9.0 release carries `biomcp-linux-x86_64.tar.gz` and `biomcp-linux-arm64.tar.gz` with sidecars. The `build` matrix produces both on every release (release.yml:24-40).
- No executable spec block asserts a workflow container job since `37336c22` removed the staged-pipeline block.
- `tests/test_release_workflow_provenance.py::test_no_other_workflow_exposes_release_publication` rejects release routes (`gh release create`, `uv publish`, `skopeo copy`, `git push`) in any other workflow; `release.yml` is the authoritative publication workflow (`sdlc/issues/2026-09-18-release-workflow-tests-describe-a-retired-workflow.md`).
- `docs/reference/release-process.md` describes the retired `stage`/`promote` design and a GHCR `latest` job that does not exist (backlog P4, `sdlc/planning/proposed-backlog-0.9.1-1.0.md:39-46`). Backlog P2 records the dropped docker job.
- Tests pin `types: [published]`, `environment: pypi`, and `homebrew-tap:` in `release.yml` (`tests/test_upstream_planning_analysis_docs.py:684-686`, `:1142`).

## Design

- Add a `container-publish` job to `release.yml`:
  - `needs: [build]`, `runs-on: ubuntu-latest`, `if: always()` so a `workflow_dispatch` run that skips `build` still publishes, and `permissions: contents: read, packages: write`.
  - Check out the release tag, `gh release download` the two Linux tarballs, extract each `biomcp` into `dist/container/{amd64,arm64}/`, and write `biomcp.sha256` beside it.
  - `docker/setup-qemu-action@v3`, `docker/setup-buildx-action@v3`, `docker/login-action@v3` (ghcr, `GITHUB_TOKEN`), then `docker/build-push-action@v6` with `platforms: linux/amd64,linux/arm64`, `push: true`, and the `<version>` tag only. Pass `SOURCE_SHA` (the tag's commit), `VERSION`, and `CREATED` build args so the Dockerfile's revision and version labels are populated.
  - Smoke both platforms by pulling and running `--version` from the registry, arm64 under QEMU, checking the running UID is not 0 and `org.opencontainers.image.revision` equals the tag's commit. Move `latest` with `docker buildx imagetools create` only after both smokes pass.
- Add a `container_only` boolean input to `workflow_dispatch`, default false. Gate `build`, `pypi-build`, `pypi-publish`, and `homebrew-tap` with `if: github.event_name == 'release' || inputs.container_only != true` so a container-only dispatch cannot touch PyPI, the release assets, or the tap.
- Extend `spec/surface/docker-image.md` with an executable block asserting `release.yml` carries the container job, `platforms: linux/amd64,linux/arm64`, the revision label, both platform smokes, and the latest promotion. `make spec` proves it.
- Rewrite `docs/reference/release-process.md` to the single-workflow process that exists, including container publication and the dispatch path. Do not reintroduce the staged pipeline.
- Add a CHANGELOG entry under Unreleased.

## Acceptance

1. `release.yml` has the container job above and a release event still runs every previous job unchanged.
2. `make lint`, `make test`, and `make spec` pass at the pushed SHA on the gate host. `test_no_other_workflow_exposes_release_publication` and the workflow markers pinned in `test_upstream_planning_analysis_docs.py` still pass.
3. A `workflow_dispatch` with `container_only: true` and `tag: v0.9.0` completes with only the container job and its smoke steps started.
4. After the dispatch, GHCR serves `0.9.0` and `latest` at the same digest whose index contains `linux/amd64` and `linux/arm64`, and the revision label equals the v0.9.0 commit. Arm64 resolves without a platform error.
5. The rewritten `release-process.md` describes only jobs and channels the workflow has.

## Out of scope

- Reviving the sealed candidate pipeline or wiring `release/` Python back into the workflow.
- Deleting the now-unused `release/` staged tooling.
- The macOS and ARM64 CI runner gap (backlog P3).
- Editing or replying to GitHub issue #281; that reply is Ian's call.

## Complexity

- Contract score: 1 (publication contract is explicit, the workflow shape is new)
- State and timing score: 1 (ordered jobs and a published registry tag; `latest` moves after smoke)
- Reach score: 1 (one public surface, release.yml, plus spec and docs)
- Proof score: 2 (hosted dispatch plus registry inspection on two platforms, not a local test)
- Cost of error score: 2 (a wrong push publishes a public artifact and can move `latest`)
- Total: 7
- Minimum level floor: none (no concurrency, credential, or durable-state change)
- Final level: 3
- Reasons: public artifact publication with two-platform proof and a mutable `latest` pointer
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer)

## Decisions

Open to Ian's overturn.

1. The image is assembled from the release's published Linux tarballs, not recompiled in the container job. The image matches the executables users download and the spec's prose.
2. The `container_only` dispatch mode lives in the authoritative `release.yml` rather than a second workflow, because a second workflow would fight the provenance test's intent.
3. `latest` moves only after both platform smokes pass; the 0.8.25 job pushed both tags at once.
4. The release-process rewrite replaces the retired stage/promote prose with the workflow that exists, which is backlog item P4.

## Review

- Design review: pending
- Code review: pending
