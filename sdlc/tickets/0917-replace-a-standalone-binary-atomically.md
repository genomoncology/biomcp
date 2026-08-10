---
flow: build
priority: 9
deps: ["0910", "0916"]
---
# Replace a standalone binary atomically

The updater writes a predictable `.biomcp.new` path with `File::create`, which
can follow a pre-planted symlink. It flushes user-space buffers but does not
sync the new file and containing directory before reporting success. Ticket
0916 establishes that the installation is owned; this ticket makes the owned
replacement durable and race-safe.

## Replacement contract

- Create a unique new file in the executable's directory with create-new and
  no-follow semantics. A predictable name is forbidden.
- Write only the checksum-verified extracted binary, set executable
  permissions where applicable, sync the file, and then perform the platform's
  atomic replacement step.
- On Unix, rename the new file over the current executable and sync the parent
  directory before success.
- On Windows, use a unique backup and new-file swap, restore the original on a
  failed swap, and never delete the only working binary. A leftover backup is
  reported rather than silently ignored.
- Before replacing the binary, atomically write and sync a transaction receipt
  that records the old and new checksums as the only accepted states. After the
  binary replacement and directory sync, atomically finalize the receipt to the
  new version and checksum. On restart, a transaction receipt whose executable
  matches either recorded checksum is recovered deterministically; any other
  checksum fails closed. This makes a crash between the two file renames
  recoverable without pretending two filesystem paths can change atomically.
- Clean up only temporary paths created by this invocation.

## Done when

Filesystem tests in an isolated directory cover success, a pre-planted old
temporary symlink, name collision, short write/sync/rename failures through an
injected file-operation seam, rollback, permission preservation, receipt
agreement, and restart at every transaction boundary. Success is reported only
after the finalized durable replacement boundary. No test replaces the running
development binary.

## Authorized test changes

Design commits may restate the replacement-seam and successful-update tests in
`src/cli/update.rs` and related release/update contract tests. Existing archive
size, extraction, and checksum failures stay fail closed.

The src line ceiling may rise by at most 170 lines.
