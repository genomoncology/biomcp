# BioMCP Overview

BioMCP is a single-binary CLI for querying biomedical sources with one command grammar. This overview confirms the binary identity, upstream API reachability, and high-level command map. The checks in this file focus on stable interface markers rather than volatile data payloads.

| Section | Command focus | Why it matters |
|---|---|---|
| Version | `biomcp version` | Confirms binary identity and semantic versioning |
| Health check | `biomcp health --apis-only` | Confirms per-source connectivity and excluded key-gated sources |
| Command reference | `biomcp list` | Confirms core entities are discoverable |
| Entity help | `biomcp list gene` | Confirms contextual filter/helper guidance |

## Version

Version output is the fastest smoke test because it exercises local binary startup without touching network sources. The assertion checks both product name and a semantic version pattern.

```bash
out="$(biomcp version)"
echo "$out" | mustmatch like "biomcp"
echo "$out" | mustmatch '/[0-9]+\.[0-9]+\.[0-9]+/'
```

## Health Check

The API-only health command reports one row per live upstream provider plus explicit excluded rows for key-gated sources. We assert on the table header and the explicit status summary, which are stable formatting markers.

```bash
out="$(biomcp health --apis-only)"
echo "$out" | mustmatch like "| API | Status | Latency |"
echo "$out" | mustmatch like "Status:"
```

## Command Reference

The command index is the human entry point for discovery. This check asserts that the reference heading renders and that representative entities remain listed.

```bash
out="$(biomcp list)"
echo "$out" | mustmatch like "# BioMCP Command Reference"
echo "$out" | mustmatch like "- `discover <query>`"
echo "$out" | mustmatch like "- variant"
echo "$out" | mustmatch like "- trial"
```

## Entity Help

Entity-specific help should expose both filter syntax and cross-entity helpers. These cues are important for users who need to move from orientation to targeted execution quickly.

```bash
out="$(biomcp list gene)"
echo "$out" | mustmatch like "## Search filters"
echo "$out" | mustmatch like "## Helpers"
```
