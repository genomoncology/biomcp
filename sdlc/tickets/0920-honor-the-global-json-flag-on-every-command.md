---
flow: build
priority: 8
deps: ["0917", "0918", "0919", "0923", "0930", "0931"]
---
# Honor the global JSON flag on every command

The global `--json` flag is accepted by commands that still write plain text.
Confirmed examples include skill content, skill list, chart, mcp-config,
`update --check`, and uninstall. A caller that asks for machine-readable output
must receive one JSON document or a clear pre-execution rejection.

## Output contract

- Every finite command that succeeds with `--json` writes exactly one valid
  JSON document to stdout and no prose before or after it.
- Progress and warnings go to stderr and never corrupt stdout.
- Errors use the existing stable JSON error envelope and the command's real
  nonzero exit code.
- Narrative results use typed envelopes: skill content names the skill and
  carries its content; chart and mcp-config expose typed fields; update and
  uninstall expose status, version/path, ownership, and changed-state fields.
- Long-running stdio or HTTP server commands for which a one-document result is
  meaningless reject `--json` before reading stdin, binding a socket, or
  starting work. They do not silently ignore it.

The direct list serialization from ticket 0919 is part of this matrix. This
ticket does not create a generic JSON wrapper around already rendered text.

## Done when

- A process-level table generated from ticket 0919's finite leaf-command
  catalog exercises every leaf with `--json`, using local fixtures and harmless
  temporary paths. The only exclusions are the explicitly enumerated
  long-running server leaves described above; a new leaf without a decision
  fails the table.
- Each allowed success parses as one JSON value and has no trailing bytes.
- Each disallowed combination fails before side effects with the stable JSON
  error envelope. There is no plain-text-error exception once `--json` parses.
- Update and uninstall tests never mutate a package-managed or development
  installation.
- Author-validation errors include the documented empty collection fields.

## Authorized test changes

Design commits may restate the global JSON matrix and exceptions in
`tests/test_cli_surface_contract_ratchet.py`,
`src/cli/response_contract.rs`, `src/cli/outcome.rs`,
`src/cli/system/dispatch.rs`, `src/cli/skill/tests/*.rs`, and list/catalog
tests. Existing entity JSON schemas remain unchanged unless this ticket names
their missing documented error field.

The src line ceiling may rise by at most 260 lines.
