# What Is BioMCP?

BioMCP is a biomedical command-line interface and MCP server designed for practical
research and clinical-informatics workflows.

It combines two ideas:

1. A stable command grammar for entity retrieval and search.
2. A protocol surface for agent runtimes through MCP tools and resources.

## Core goal

The project goal is not to mirror every field from every API.
The goal is to deliver dependable, compact answers that can be expanded on demand.

## The command grammar

BioMCP uses one consistent grammar:

- `search <entity> [filters]`
- `get <entity> <id> [section...]`

This grammar is intentionally shared across entities so users and agents do not
need a different mental model per endpoint.

## Entities

BioMCP covers entities across clinical, research, and regulatory domains:

**Core clinical entities:** gene, variant, trial, article, drug, disease

**Extended entities:** pathway, protein, adverse-event, PGx (pharmacogenomics)

**Discovery entities:** GWAS, phenotype search, biomarker, organization, intervention

## Why this matters for agents

Agent runtimes often fail on data tooling for three reasons:

- command discovery is inconsistent,
- output is too verbose or unstable,
- source provenance is unclear.

BioMCP addresses these with:

- predictable commands,
- compact markdown-first defaults,
- explicit source mapping in docs.

## What BioMCP is not

BioMCP is not a local biomedical warehouse.
It is also not a replacement for formal clinical interpretation systems.

It is a workflow interface that normalizes retrieval from trusted public APIs.

## Progressive detail

BioMCP starts concise, then expands by section.

Examples:

```bash
biomcp get gene BRAF
biomcp get gene BRAF pathways
biomcp get article 22663011
biomcp get article 22663011 fulltext
```

The first command gives orientation.
The second command gives focused depth.

## MCP mode

`biomcp serve` runs a stdio MCP server.

The server advertises:

- tools: one execution tool (`shell`)
- resources: curated markdown help and pattern documents

This keeps client integration simple while preserving discoverability.

## Where BioMCP fits

BioMCP works well for:

- trial searching triage,
- variant-to-literature pivots,
- disease normalization and downstream lookups,
- adverse-event and recall quick checks.

## Design priorities

The design priorities are:

- deterministic command behavior,
- explicit error messages,
- low context overhead,
- documented API provenance,
- CI-backed contract checks.

## Next reading

- Concept: [Progressive disclosure](progressive-disclosure.md)
- User guides: `docs/user-guide/`
- Data provenance: `docs/reference/data-sources.md`
- MCP behavior: `docs/reference/mcp-server.md`
