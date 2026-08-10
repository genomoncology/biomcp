---
flow: build
priority: 7
deps: ["0916", "0940"]
---
# Replace the full biomcp-cli executable with a shim

The PyPI package is correctly named `biomcp-cli`, but the wheel installs both
`biomcp` and an undocumented `biomcp-cli` executable. Each is currently a full
31.8 MiB native program, doubling installed payload and complicating ownership.

## Compatibility contract

Keep the PyPI distribution name `biomcp-cli`. For one compatibility release,
retain its command as a small forwarding executable that locates the sibling
`biomcp`, passes every argument and standard stream through, and returns the
same exit status. It writes one plain deprecation notice to stderr naming
`biomcp`; stdout remains byte-for-byte the real command output, including JSON
and binary redirection.

The shim must not link the BioMCP application library or contain a second copy
of its providers, templates, or embedded assets. Its stripped installed size
is at most 1 MiB on every supported platform. A missing or non-executable
sibling fails clearly without searching arbitrary PATH entries. Draft ticket
0947 owns removal after the compatibility release.

## Done when

- Fresh Linux, macOS, and Windows wheel inspections contain one full `biomcp`
  and a shim no larger than 1 MiB, not two full executables.
- Version, help, JSON success/error, nonzero exit, signal/interruption where
  supported, and a local binary-output redirection behave like direct `biomcp`
  except for the stderr deprecation notice.
- Updater/uninstaller ownership from 0916 recognizes the package-managed shim
  without attempting to self-mutate it.
- Public installation docs distinguish the `biomcp-cli` package name from the
  deprecated command name and direct all examples to `biomcp`.
- Release and wheel-size contracts fail if the full duplicate returns.

## Authorized test changes

Design commits may restate Cargo binary targets, the shim entrypoint, maturin
wheel tests, package metadata, updater ownership tests, and installation docs.
Existing `biomcp` behavior and the PyPI distribution identity remain covered.

The src line ceiling may rise by at most 60 lines.
