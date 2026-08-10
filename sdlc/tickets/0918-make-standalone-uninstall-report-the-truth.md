---
flow: build
priority: 9
deps: ["0916"]
---
# Make standalone uninstall report the truth

`biomcp uninstall` currently turns a file-removal error into a success string
and exit zero. A non-writable test left the binary in place while claiming the
command completed. Ticket 0916 limits this command to an installer-owned
standalone binary; this ticket defines completion for that installation.

## Uninstall contract

An uninstall owns exactly the receipt-proven executable and its adjacent
receipt. It does not scan `PATH`, delete package-manager metadata, or remove an
undocumented second executable.

- On platforms that permit removing the running executable, remove both owned
  files and report success only when neither remains.
- On platforms that require deferred deletion, use a bounded, tested helper
  that carries the exact canonical paths and reports the scheduled state
  honestly. If safe deferred deletion cannot be established, return a nonzero
  typed error with an exact manual command instead of claiming success.
- Any permission, identity, partial-removal, or helper failure is nonzero and
  identifies which owned path remains.
- Repeating the command after a completed uninstall is an ordinary
  not-installed error, not success.

## Done when

Isolated process tests cover complete removal, non-writable paths, a receipt
that changes after validation, one-file partial failure, platform-specific
deferred behavior, and repeated invocation. No test targets the repository,
the active test binary, a virtual environment outside its temporary fixture,
or a real package-manager installation.

## Authorized test changes

Design commits may restate `uninstall_self` tests in
`src/cli/system/dispatch.rs`, CLI outcome tests, and isolated package smoke
tests that currently accept a success message after incomplete removal.
Ticket 0920 will give the resulting typed outcome its global JSON rendering;
this ticket must return enough structured state for that rendering.

The src line ceiling may rise by at most 110 lines.
