# FAQ

## Does BioMCP require API keys?

No for most workflows.

Optional keys improve access for specific enrichments and source variants.
See [API keys](../getting-started/api-keys.md).

## Where are cached files stored?

BioMCP uses platform cache directories.

Typical locations:

- Linux: `~/.cache/biomcp/`
- macOS: `~/Library/Caches/biomcp/`

The cache generally contains:

- `http-cacache/` for HTTP responses,
- `downloads/` for retrieved text artifacts.

## Why do invalid dates fail immediately?

Date validation is performed before network calls.

Examples of invalid inputs:

- `2024-13-01` (invalid month)
- `2024-02-30` (invalid day)
- `2023-02-29` (non-leap-year day)

This behavior provides immediate feedback and avoids unnecessary API requests.

## How do I request full article text now?

Use positional sections:

```bash
biomcp get article 22663011 fulltext
```

The same model applies to other entities:

```bash
biomcp get trial NCT02576665 eligibility
biomcp get gene BRAF pathways
biomcp get variant "BRAF V600E" predict
```

## How do I use BioMCP with Claude Desktop?

Configure an MCP server entry using `biomcp serve`.
See [Claude Desktop setup](../getting-started/claude-desktop.md).

## Why are some fields missing?

BioMCP returns concise defaults and depends on upstream source completeness.

When you need more detail:

1. request a section,
2. switch to JSON,
3. verify source availability with `biomcp health --apis-only`.

## How do I inspect available commands quickly?

```bash
biomcp list
biomcp list trial
biomcp list variant
```

## Is BioMCP suitable for final clinical decisions?

BioMCP is a retrieval and workflow tool.
Clinical decisions require domain-expert review and institutional validation processes.

## How do I report reproducible issues?

Include:

- exact command,
- expected vs actual behavior,
- `biomcp --version`,
- relevant environment details,
- whether the issue reproduces with `--json`.
