# Docker Image

BioMCP's release image is assembled only from the two Linux executables the
release publishes. The container build does not compile source or download a
different BioMCP executable.

## Runtime Image Is Bounded And Non-Root

The pinned runtime layer checks the staged executable, retains HTTPS trust
roots, creates private state directories, declares no service port, and runs as
the dedicated non-root account.

```bash
cat ../../Dockerfile | mustmatch like 'debian:bookworm-slim@sha256:
ca-certificates
sha256sum -c
USER 65532:65532
ENTRYPOINT ["biomcp"]'
! rg -n '^(EXPOSE|FROM rust|FROM quay.io/pypa)' ../../Dockerfile
```

## Build Context Contains Only Staged Inputs

```bash
cat ../../.dockerignore | mustmatch like '**
!Dockerfile
!dist/container/**'
```

## The Release Publishes Both Linux Architectures

The release workflow verifies the release's Linux tarballs against their
published sidecars, stages them, pushes one image index for both architectures
under the release's version tag, smokes each platform from the registry, and
moves `latest` only after both smokes pass and only when the tag is the
repository's latest release.

```bash
cat ../../.github/workflows/release.yml | mustmatch like 'container-publish:
concurrency:
group: container-publish-
platforms: linux/amd64,linux/arm64
sha256sum -c biomcp-linux-x86_64.tar.gz.sha256
gh release view
--jq .tagName
org.opencontainers.image.revision
Smoke the linux/amd64 image from the registry
Smoke the linux/arm64 image from the registry
docker buildx imagetools create'
```

## Documentation Shows CLI And Stdio MCP Use

```bash
cat ../../README.md ../../docs/getting-started/installation.md ../../docs/reference/mcp-server.md | mustmatch like 'docker run --rm ghcr.io/genomoncology/biomcp --version
docker run --rm ghcr.io/genomoncology/biomcp list
docker run --rm -i ghcr.io/genomoncology/biomcp serve'
```
