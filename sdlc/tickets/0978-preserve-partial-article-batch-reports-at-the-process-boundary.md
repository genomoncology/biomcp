---
flow: build
priority: 9
---
# Preserve partial article-batch reports at the process boundary

Ticket 0973 established that a partially failed batch is a completed report:
the report belongs on stdout, the process exits nonzero, and stderr remains
empty. The general `batch` command now honors that contract, but `article
batch` falls through a legacy string-returning dispatch path. That path turns
its correct nonzero `CommandOutcome` back into an `Error:` blob on stderr and
leaves stdout empty.

Route article batch through the same outcome-preserving process boundary as
other structured commands. JSON and Markdown must behave alike, and MCP must
retain the complete text report rather than treating partial settlement as a
tool transport failure. Update the CLI reference, which still describes the
old bare-array article-batch JSON shape.

## Done when

- A deterministic article batch with one success and one failure writes the
  complete report to stdout, writes nothing to stderr, and exits nonzero in
  JSON and Markdown modes.
- The JSON report remains parseable and uses the shipped `{summary, items}`
  envelope.
- Raw and typed MCP execution retain the complete partial report within their
  existing text/structured boundaries.
- A process-level regression test observes stdout, stderr, and exit status
  together, and the CLI reference agrees with the executable contract.

## Authorized test changes

Design may restate outcome dispatch assertions in `src/cli/outcome.rs`, article
batch assertions in `src/cli/article/dispatch.rs`, process-boundary assertions
in `tests/json_error_contract.rs` or a new focused Rust integration test, and
the article batch contract in `docs/user-guide/cli-reference.md` and its
existing documentation contract tests.
