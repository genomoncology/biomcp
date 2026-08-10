---
flow: build
priority: 10
---
# Let self-update download current release archives

`src/cli/update.rs` declares a 256 MiB release-archive limit, but the download
uses both the shared response middleware and the default body reader at their
8 MiB limits. Every v0.8.25 release archive is larger than 8 MiB. A published
v0.8.24 binary therefore fails to update with `Response body exceeded 8388608
bytes` before archive verification begins.

## Done when

The release download applies `MAX_RELEASE_ARCHIVE_BYTES` at both independent
boundaries:

- `with_response_body_limit` on the request; and
- `read_limited_body_with_limit` on the response.

Changing only one boundary is not complete. Release metadata and checksum
sidecars keep their smaller normal limits. The GitHub release base and asset
transport are injectable in tests; routine tests never call GitHub.

A local HTTP fixture proves:

- declared-content-length and chunked archives larger than 8 MiB and no larger
  than 256 MiB reach checksum and extraction;
- declared and chunked bodies over 256 MiB fail before installation;
- a truncated archive, missing checksum, malformed checksum, and mismatch all
  fail closed; and
- a successful verified archive invokes the replacement seam exactly once.

The release ticket owns a post-publication smoke in which the previous
published CLI checks and downloads the new published asset. This ticket owns
the deterministic local contract.

## Authorized test changes

Design commits may restate and extend the inline tests in
`src/cli/update.rs`, `tests/test_update_command_docs_contract.py`, and the
release smoke contract that currently assumes the updater can consume a
published asset. Existing 256 MiB, extracted-binary, and checksum safety
assertions remain in force.

The src line ceiling may rise by at most 110 lines.
