# Reversed search grammar suggests an unrelated command

Severity: nice-to-have

Two natural command guesses failed during a researcher knowledge-base exercise:

```text
$ biomcp article search
error: unrecognized subcommand 'search'
tip: a similar subcommand exists: 'batch'

$ biomcp trial search
error: unrecognized subcommand 'trial'
tip: a similar subcommand exists: 'article'
```

The runnable commands are `biomcp search article` and `biomcp search trial`. The current diagnostics point in the wrong direction. An agent can recover through `biomcp list`, but the failed command already contains enough information to print the canonical form.

The parser delegates these errors to Clap in `src/cli/shared.rs::try_parse_cli`. Clap compares the unexpected word with sibling subcommands at the current grammar level. It cannot infer that the user reversed the entity and operation. `parse_cli_from_env` then renders that diagnostic without a BioMCP-specific correction. This behavior belongs to BioMCP's CLI error layer rather than an upstream data source.

Detect the `<entity> search` shape for searchable entities and replace the unrelated similarity tip with `Use biomcp search <entity> ...`. BioMCP does not need to accept the reversed form as an alias. A correction keeps one canonical grammar and makes recovery direct. Tests should cover human and JSON diagnostics.

Both reproductions were verified with `biomcp 0.9.0-dev.6` on 2026-09-04.
