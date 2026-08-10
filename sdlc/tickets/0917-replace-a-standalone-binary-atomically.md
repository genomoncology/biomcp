---
flow: build
priority: 9
deps: ["0910", "0916"]
---
# Replace an owned Unix standalone binary atomically

The updater writes a predictable `.biomcp.new` path with `File::create`, which
can follow a pre-planted symlink. It flushes user-space buffers but does not
sync the new file and containing directory before reporting success. Ticket
0916 establishes that the installation is owned; this ticket makes the owned
Unix replacement durable and race-safe. A running Windows executable needs a
separate helper transaction; draft ticket 0945 owns that design.

## Replacement contract

- Create a unique new file in the executable's directory with create-new and
  no-follow semantics. A predictable name is forbidden.
- Write only the checksum-verified extracted binary, set executable
  permissions, execute the staged file's `--version` smoke, sync the file, and
  then perform the Unix atomic replacement step.
- Immediately before mutation, reopen without following symlinks and
  revalidate the current executable identity, checksum, and installer receipt.
  A path or receipt changed during download fails closed and leaves it intact.
- Rename the new file over the current executable and sync the parent directory
  before success.
- On Windows, `biomcp update` fails before download or mutation with a typed
  unsupported message directing the user to the verified installer. Do not
  attempt a backup/swap from the running process and do not report a scheduled
  replacement as completed.
- Before replacing the binary, atomically write and sync a transaction receipt
  that records the old and new checksums as the only accepted states. After the
  binary replacement and directory sync, atomically finalize the receipt to the
  new version and checksum. On restart, a transaction receipt whose executable
  matches either recorded checksum is recovered deterministically; any other
  checksum fails closed. This makes a crash between the two file renames
  recoverable without pretending two filesystem paths can change atomically.
- Clean up only temporary paths created by this invocation.

## Done when

Filesystem tests in an isolated directory cover success, staged-version smoke
failure, a pre-planted old temporary symlink, name collision, current-path and
receipt replacement during download, short write/sync/rename failures through
an injected file-operation seam, permission preservation, receipt agreement,
and restart at every transaction boundary. Windows tests prove rejection
occurs before transport and filesystem mutation. Success is reported only
after the finalized durable replacement boundary. No test replaces the running
development binary.

## Authorized test changes

Design commits may restate the replacement-seam and successful-update tests in
`src/cli/update.rs` and related release/update contract tests. Existing archive
size, extraction, and checksum failures stay fail closed. They may restate
Windows update expectations and public update documentation to say that
self-update is unavailable there until draft 0945 is deliberately promoted
and completed.

The src line ceiling may rise by at most 240 lines.
