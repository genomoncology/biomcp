# Docker Image

BioMCP's release image is assembled only from the two Linux executables already
registered in the sealed candidate. The container build does not compile source
or download a different BioMCP executable.

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

## Stage And Promotion Stay Separate

The private stage creates and smokes a two-platform OCI archive without a push.
Only the protected promotion jobs can write the versioned GHCR tag. Both public
architectures are then pulled, checked for the full revision label, run as
non-root, and reconciled before `latest` moves.

```bash
cat ../../.github/workflows/release.yml | mustmatch like 'mode:
container-artifact:
--output type=oci,dest=dist/oci/biomcp.oci.tar
publish-versioned:
public-container-smoke:
org.opencontainers.image.revision
advance-mutable-pointers:'
```

## Documentation Shows CLI And Stdio MCP Use

```bash
cat ../../README.md ../../docs/getting-started/installation.md ../../docs/reference/mcp-server.md | mustmatch like 'docker run --rm ghcr.io/genomoncology/biomcp --version
docker run --rm ghcr.io/genomoncology/biomcp list
docker run --rm -i ghcr.io/genomoncology/biomcp serve'
```
