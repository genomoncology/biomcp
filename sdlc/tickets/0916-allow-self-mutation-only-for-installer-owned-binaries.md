---
flow: build
priority: 9
deps: ["0911"]
---
# Allow self-mutation only for installer-owned binaries

`update` and `uninstall` currently mutate `current_exe` without knowing who
owns the installation. In an isolated PyPI wheel, uninstall removed one entry
point while the `biomcp-cli` executable and distribution metadata remained.
A repaired updater would likewise split package state. Homebrew and other
package managers need the manager, not BioMCP, to own replacement.

## Ownership contract

The canonical standalone installer writes an adjacent
`biomcp.install.json` receipt atomically after the verified binary is in place.
The receipt contains a schema version, installer identity, canonical executable
path, installed BioMCP version, and SHA-256 of the installed executable.

`update` and `uninstall` may mutate files only when:

- the executable and receipt are regular files in the same directory;
- neither path is reached through a symbolic link;
- the receipt names the canonical current executable; and
- the recorded executable checksum matches the file being asked to mutate.

Without that proof, both commands leave every file unchanged and return a typed
`package_managed_install` error. Detect pipx/uv/pip virtual environments and
Homebrew paths when possible and show the exact manager command. An unknown
owner gets generic guidance to use its package manager or reinstall with the
canonical standalone installer. Path guessing alone never grants mutation
authority.

## Done when

Temporary-install tests cover a valid standalone receipt, missing/malformed/
mismatched receipts, a symlinked executable or receipt, an isolated wheel with
both entry points, a pipx-style path, and a Homebrew-style path. Refused cases
make no file or metadata change. Human and typed error details name the safe
next action without claiming an uninstall or update occurred.

## Authorized test changes

Design commits may restate ownership and installer tests in `install.sh`,
`tests/test_public_installer_checksum.py`, `src/cli/update.rs`,
`src/cli/system/dispatch.rs`, and their existing CLI/system tests. Existing
checksum validation and package installation smoke assertions remain.

The src line ceiling may rise by at most 170 lines.
