---
flow: build
priority: 9
deps: ["0911"]
---
# Install a verified binary with one atomic rename

The standalone installer verifies an archive in its temporary directory, then
moves the binary directly over the installed path and executes it afterward.
Across filesystems that move can become a copy, and a failed smoke leaves no
previous working binary.

## Installer transaction

After archive and checksum verification, create a unique no-follow staging file
inside the destination directory. Copy only the verified binary into it, set
the final permissions, sync it, and execute that staged path with `--version`.
The version must match the requested release before any installed path changes.

On Unix, rename the staged file over the destination as the only installation
commit and sync the parent directory. A crash or error before rename preserves
the prior binary byte-for-byte; after rename the already-smoked binary is the
only visible version. On Windows-compatible shells, use a same-directory
replacement primitive that either commits the verified file or preserves the
old destination; if that cannot be guaranteed, fail before changing it.

Write the installer ownership receipt only after the executable commit and
directory sync. Clean up only invocation-owned staging paths. Never resolve a
destination symlink or use a predictable temporary name.

## Done when

- Shell-level filesystem tests cover new install, upgrade, cross-filesystem
  download temp, pre-planted symlink/name collision, short copy, chmod, staged
  smoke, rename, sync, and receipt failures.
- Every pre-commit failure leaves an existing sentinel binary and receipt
  unchanged; success exposes the new binary and matching receipt together.
- The executed smoke path is the staged destination-directory file, not the
  archive temp or already replaced destination.
- Root installer tests remain the sole deployed installer contract after 0911;
  no second implementation regains authority.
- Installation and troubleshooting documentation describe the transaction.

## Authorized test changes

Design commits may restate canonical `install.sh`, installer checksum/ownership
tests, release artifact smoke, and installation documentation. Existing
platform selection, archive bounds, checksum failure, and installer identity
assertions remain covered. No product Rust source change belongs here.

The src line ceiling may not rise.
