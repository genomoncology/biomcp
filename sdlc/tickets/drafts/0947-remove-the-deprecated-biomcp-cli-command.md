---
flow: build
priority: 1
deps: ["0939"]
---
# Remove the deprecated biomcp-cli command after one compatibility release

Held as a draft until one public release has shipped ticket 0939's small
forwarding shim. The PyPI distribution remains named `biomcp-cli`; only the
deprecated executable command is removed.

## Removal contract

Remove the `biomcp-cli` Cargo binary target and wheel script after the
compatibility release. Fresh native archives and wheels install only the
`biomcp` executable. Installation, migration, updater ownership, release smoke,
and size tests distinguish package name from command name and contain no claim
that `biomcp-cli` remains executable.

Do not promote this ticket into the same release that first ships the shim.
Release notes must name the prior compatibility release and the supported
`biomcp` replacement.

The src line ceiling must fall by the removed shim.
