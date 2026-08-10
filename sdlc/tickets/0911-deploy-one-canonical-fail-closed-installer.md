---
flow: build
priority: 10
deps: ["0951"]
---
# Deploy one canonical fail-closed installer

Archived ticket 0620 made the root `install.sh` fail closed, but the separately
maintained `docs/install.sh` did not receive that behavior. The script served at
`https://biomcp.org/install.sh` matches the stale docs copy: it continues when
no SHA-256 tool exists and when the checksum sidecar is missing. The public
installation guide says the opposite.

## Source and deployment contract

The repository-root `install.sh` is the sole authored installer. The docs-site
copy is generated from it and must be byte-for-byte identical. The docs and
release workflows fail before deployment if any packaged copy differs.

The canonical installer exits nonzero without installing or replacing a file
when:

- none of `sha256sum`, `shasum -a 256`, or `openssl dgst -sha256` is available;
- the checksum sidecar cannot be fetched;
- the sidecar is empty or malformed; or
- the computed checksum differs.

There is no warning-and-continue path and no unsafe override in the public
installer.

## Done when

- `tests/test_public_installer_checksum.py` exercises every failure above and
  one successful local installation without public network.
- A packaging contract proves every shipped installer copy is identical to the
  root file.
- `.github/workflows/ci.yml` and `.github/workflows/release.yml` build docs only
  after that identity contract passes.
- Release verification compares the deployed public bytes with the canonical
  file and fails if they differ.
- `docs/getting-started/installation.md` describes the behavior actually
  shipped.

This ticket repairs a regression of completed ticket 0620; it does not rewrite
that completion record.

## Authorized test changes

Design commits may restate installer checksum tests, public-installation docs
contracts, MkDocs packaging checks, and release workflow assertions in
`tests/test_public_installer_checksum.py`,
`tests/test_public_install_docs_contract.py`,
`tests/test_release_smoke_script.py`, and related release-contract tests. The
root installer's existing successful platform selection behavior stays intact.

The src line ceiling may not rise.
