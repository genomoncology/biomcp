---
flow: build
priority: 6
deps: ["0937", "0948", "0952", "0953"]
---
# Build a non-root multi-architecture container

The public container is single-architecture and runs as root. It needs the same
source identity and usable local-state contract as native artifacts without
turning BioMCP into a hosted service.

## Container contract

Stage one OCI image index for `linux/amd64` and `linux/arm64`. Each platform
copies the exact verified 0953 Linux executable selected by target and
candidate-manifest SHA-256; the container build performs no Rust compilation
or second source checkout. It runs as a
non-root user, retains CA certificates, and has an owned writable home plus
BioMCP cache/config directories with ticket 0948's private permissions. The
image supports ordinary CLI and local stdio MCP only; it exposes no network
service port and adds no hosted-server default.

Set standard OCI source, revision, version, licenses, and created labels.
Attach an SBOM and build provenance to the versioned digest. A versioned tag and
the multi-platform index are immutable; `latest` is handled only after public
verification under 0957.

## Done when

- On both architectures, an isolated candidate runs version/help, a loopback
  fixture-backed CLI lookup, stdio MCP initialize/tool call, and an actual
  cache write/read as the non-root user.
- The manifest contains exactly both required platforms, identical version and
  revision labels, no root runtime, no unexpected port, and no source,
  fixtures, credentials, or workstation paths.
- Digest-pinned base and build images, QEMU/native runner identity, layer
  contents, SBOM, and provenance are checked deterministically.
- Ticket 0957 requires a successful exact-SHA candidate run and both public
  platform pulls before moving `latest`; this ticket performs no publication.

## Authorized test changes

Design commits may restate `Dockerfile`, buildx/candidate workflow, container
fixture smoke, OCI metadata, SBOM/provenance, and Docker documentation tests.
Do not add remote hosting, authentication, or infrastructure.

The src line ceiling may not rise.
