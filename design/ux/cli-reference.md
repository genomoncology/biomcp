# BioMCP CLI Reference (UX Analysis)

This document captures stable CLI ergonomic patterns, demo workflows, and MCP
configuration references. It is a durable UX reference for future design,
documentation, and verification work — not a user manual.

## Command Grammar

```
biomcp search <entity> [filters]      → discovery queries
biomcp get <entity> <id> [sections]   → focused detail
biomcp <entity> <helper> <id>         → cross-entity pivot
biomcp enrich <GENE1,GENE2,...>        → gene-set enrichment
biomcp batch <entity> <id1,id2,...>    → parallel gets
biomcp search all [slot filters]      → unified fan-out
```

Ops commands:
```
biomcp health [--apis-only]   → inspect per-source connectivity and excluded key-gated rows
biomcp version                → show version and build info
biomcp update [--check]       → self-update or check for updates
biomcp list [entity]          → show entities, commands, and filters
biomcp skill                  → show the embedded BioMCP agent guide
biomcp skill install <dir>    → install the BioMCP guide into an agent directory
biomcp skill list             → legacy alias; currently reports no embedded catalog
biomcp serve-http            → run the MCP Streamable HTTP server at `/mcp`
biomcp serve-sse             → removed compatibility command; use `biomcp serve-http`
```

## Progressive Disclosure Pattern

Every `get` command returns a summary card by default. Sections extend output:

```bash
biomcp get gene BRAF                      # summary card only
biomcp get gene BRAF pathways             # + pathway section
biomcp get gene BRAF civic interactions   # + multiple sections
biomcp get gene BRAF all                  # everything

biomcp get variant "BRAF V600E" clinvar population conservation
biomcp get article 22663011 tldr
biomcp get drug pembrolizumab label targets civic approvals
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get trial NCT02576665 eligibility locations outcomes
```

The pattern is consistent across all 12 entity types: no-section gives a
summary, named sections are additive, `all` gives the full record.

## Cross-Entity Pivot Pattern

Pivot helpers allow moving between related entities without rebuilding filters:

```bash
# Variant pivots
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E"

# Drug pivots
biomcp drug adverse-events pembrolizumab
biomcp drug trials pembrolizumab

# Disease pivots
biomcp disease trials melanoma
biomcp disease drugs melanoma
biomcp disease articles "Lynch syndrome"

# Gene pivots
biomcp gene trials BRAF
biomcp gene drugs BRAF
biomcp gene articles BRCA1
biomcp gene pathways BRAF

# Pathway pivots
biomcp pathway drugs R-HSA-5673001
biomcp pathway articles R-HSA-5673001
biomcp pathway trials R-HSA-5673001

# Protein pivots
biomcp protein structures P15056

# Article pivots
biomcp article entities 22663011
biomcp article citations 22663011 --limit 3
biomcp article references 22663011 --limit 3
biomcp article recommendations 22663011 --limit 3
```

## `search all` Contract

`search all` is typed slots first. The durable contract is to express intent
through named slots, with the positional form retained only as a keyword alias.

Primary examples:

```bash
biomcp search all --gene BRAF --disease melanoma
biomcp search all --gene BRAF --counts-only
biomcp search all --keyword "checkpoint inhibitor"
```

Spec shorthand uses the equivalent short flags:

```bash
biomcp search all -g BRAF -d melanoma
biomcp search all -k "checkpoint inhibitor"
```

Secondary positional alias:

```bash
biomcp search all BRAF
```

Fans out in parallel across genes, variants, diseases, drugs, trials,
articles, pathways, PGx, GWAS, and adverse events. Use typed slots in docs,
demos, and help text; treat the positional alias as compatibility syntax rather
than the primary teaching path. Federated totals are approximate.

## Unified Search

The `search all` response is a counts-first orientation card for exploratory
work. A single slot still returns a multi-entity summary, while `--counts-only`
suppresses row bodies for lower-noise planning.

## Demo Workflows

### GeneGPT Demo: Variant → Trial → Article Evidence Walk

Source: `scripts/genegpt-demo.sh`

```bash
# 1. Get gene summary
biomcp --json get gene BRAF

# 2. Get variant population data
biomcp --json get variant "BRAF V600E" population

# 3. Find trials for the variant
biomcp --json variant trials "BRAF V600E" --limit 3

# 4. Find supporting literature
biomcp --json search article -g BRAF -d melanoma --limit 3
```

Scoring: evidence_score = trial_count + article_count. Non-zero score confirms
the core variant-evidence pipeline is working.

### GeneAgent Demo: Variant → Pathway → Drug → Protein Walk

Source: `scripts/geneagent-demo.sh`

```bash
# 1. Get variant ClinVar annotation
biomcp --json get variant "BRAF V600E" clinvar

# 2. Get pathway gene members
biomcp --json get pathway R-HSA-5673001 genes

# 3. Find drugs in pathway
biomcp --json pathway drugs R-HSA-5673001 --limit 3

# 4. Get protein structures
biomcp --json protein structures P15056
```

Scoring: drug_count from pathway drugs. Non-zero confirms the
variant→pathway→drug→protein pivot chain is working.

These two scripts are the canonical smoke checks for a working BioMCP release.
Run them after any significant change to the entity surface.

## MCP Server Configuration

Standard MCP client config:

```json
{
  "mcpServers": {
    "biomcp": {
      "command": "biomcp",
      "args": ["serve"]
    }
  }
}
```

Multi-worker deployment (shared rate limiter):

```bash
# Start shared Streamable HTTP server
biomcp serve-http --host 0.0.0.0 --port 8080

# Point agent workers at /mcp instead of spawning individual biomcp processes
```

## Key UX Invariants

These properties should be preserved across releases:

1. **`biomcp list`** shows all entities and commands — must not reference
   stale or removed commands
2. **`biomcp list <entity>`** shows entity-specific filters and examples —
   examples must be runnable
3. **JSON output** (`--json` flag) is available on all query commands and
   produces valid JSON — scripts and agents depend on this
4. **`biomcp health`** reports per-source connectivity, cache writability, and
   excluded key-gated sources in one inspection view; partial upstream failures
   stay visible in output even though the command currently exits 0
5. **Error messages** include suggested next steps — suggestions must name
   real commands

## Skills Quick Reference

Overview: `biomcp skill` (prints the embedded `SKILL.md` guide)

Install: `biomcp skill install ~/.claude --force`

List: `biomcp skill list` (currently prints `No skills found` because the old
embedded use-case catalog no longer ships)

Legacy lookup: `biomcp skill 03` or `biomcp skill variant-to-treatment`
returns a clear not-found error rather than stale content

Install output lands in `skills/biomcp/` and currently includes `SKILL.md`
plus `jq-examples.md`. The installer auto-discovers existing config
directories (`.claude`, `.agents/skills/`, etc.) when no directory is passed.
