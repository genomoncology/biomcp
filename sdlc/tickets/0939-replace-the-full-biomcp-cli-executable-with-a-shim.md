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

Keep the PyPI distribution name `biomcp-cli` and retain the `biomcp-cli`
command indefinitely as a supported, small forwarding alias. It locates the
sibling `biomcp`, passes every argument and standard stream through, and
returns the same exit status. It writes no routine warning, so stdout and
stderr remain byte-for-byte the real command output, including JSON, errors,
and binary redirection. There is no scheduled removal ticket or deprecation
promise.

The shim must not link the BioMCP application library or contain a second copy
of its providers, templates, or embedded assets. Its stripped installed size
is at most 1 MiB on every supported platform. A missing or non-executable
sibling fails clearly without searching arbitrary PATH entries.

## Done when

- Fresh Linux, macOS, and Windows wheel inspections contain one full `biomcp`
  and a shim no larger than 1 MiB, not two full executables.
- Version, help, JSON success/error, nonzero exit, signal/interruption where
  supported, and a local binary-output redirection behave exactly like direct
  `biomcp`.
- Updater/uninstaller ownership from 0916 recognizes the package-managed shim
  without attempting to self-mutate it.
- Public installation docs distinguish the `biomcp-cli` package name from its
  supported compatibility command and use `biomcp` in primary examples.
- Release and wheel-size contracts fail if the full duplicate returns.

## Authorized test changes

Design commits may restate Cargo binary targets, the shim entrypoint, maturin
wheel tests, package metadata, updater ownership tests, and installation docs.
Existing `biomcp` behavior and the PyPI distribution identity remain covered.

The src line ceiling may rise by at most 60 lines.
