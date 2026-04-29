# BioMCP CLI Reference (UX Analysis)

This document captures stable CLI ergonomic patterns, demo workflows, and MCP
configuration references. It is a durable UX reference for future design,
documentation, and verification work — not a user manual.

## Command Grammar

```
biomcp search <entity> [filters]      → discovery queries
biomcp get <entity> <id> [sections]   → focused detail
biomcp <entity> <helper> <id>         → cross-entity pivot
biomcp suggest <question>             → offline question-to-playbook routing
biomcp discover <query>               → single-entity free-text resolution into typed follow-up commands
biomcp enrich <GENE1,GENE2,...>        → gene-set enrichment
biomcp batch <entity> <id1,id2,...>    → parallel gets
biomcp search all [slot filters]      → unified fan-out
```

Ops commands:
```
biomcp health [--apis-only]   → inspect per-source connectivity and excluded key-gated rows
biomcp version                → show version and build info
biomcp update [--check] [--allow-missing-checksum] → self-update or check for updates
biomcp list [entity]          → show entities, commands, and filters
biomcp skill                  → show the embedded BioMCP agent guide
biomcp skill render           → print the canonical agent prompt
biomcp skill install <dir>    → install the BioMCP guide into an agent directory
biomcp skill list             → list embedded worked examples
biomcp suggest "<question>"   → select a worked-example playbook and two starter commands
biomcp cache path             → print the managed HTTP cache path (plain text; ignores `--json`)
biomcp cache stats            → show HTTP cache statistics (JSON supported)
biomcp cache clean            → remove orphan blobs and optionally age- or size-evict the HTTP cache (JSON supported)
biomcp cache clear [--yes]    → destructively wipe the managed HTTP cache tree (JSON success; TTY or `--yes` required)
biomcp gtr sync               → force-refresh the local GTR diagnostic bundle
biomcp who-ivd sync           → force-refresh the local WHO IVD diagnostic CSV
biomcp serve-http            → run the MCP Streamable HTTP server at `/mcp`
```

Compatibility note: `biomcp serve-sse` remains available only as a hidden compatibility command that points users to `biomcp serve-http`.

`discover` is primarily the single-entity resolver in this command grammar.
When relational or multi-entity free text only weakly matches a single concept,
the UX contract is to redirect through `biomcp search all --keyword "<query>"`
via discover `notes` and `_meta.next_commands` rather than surfacing noisy
stopword-collocation residue. Existing symptom, treatment, gene+disease, and
gene-plus-topic discover exceptions remain intact.

## Progressive Disclosure Pattern

Every `get` command returns a summary card by default. Sections extend output:

```bash
biomcp get gene BRAF                      # summary card only
biomcp get gene BRAF pathways             # + pathway section
biomcp get gene BRAF civic interactions   # + multiple sections
biomcp get gene BRAF all                  # everything

biomcp get variant "BRAF V600E" clinvar population conservation
biomcp get article 22663011 tldr
biomcp get diagnostic GTR000006692.3 genes conditions
biomcp get diagnostic "ITPW02232- TC40" conditions
biomcp get drug pembrolizumab label targets civic approvals
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get disease "uterine leiomyoma" clinical_features
biomcp get trial NCT02576665 eligibility locations outcomes
```

The pattern is consistent across the entity command surface: no-section gives
a summary, named sections are additive, and `all` gives the standard default
surface rather than every opt-in section. Opt-in sections such as
`clinical_features`, `diagnostics`, `disgenet`, and `funding` still require
explicit naming.

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

1. **`biomcp list`** shows all entities and top-level commands, including
   `discover` and the major command families — it must not reference stale or
   removed commands, but runtime-generated per-record `next_commands` remain
   outside the static list contract
2. **`biomcp list <entity>`** shows entity-specific filters and examples —
   examples must be runnable
3. **JSON output** (`--json` flag) is available on all query commands and
   produces valid JSON — scripts and agents depend on this. `biomcp cache path`
   is the documented operator-command exception: it stays plain text even under
   `--json`, while `biomcp cache stats` and `biomcp cache clean` keep their
   normal JSON contracts
4. **`biomcp health`** reports per-source connectivity, cache writability, and
   excluded key-gated sources in one inspection view; partial upstream failures
   stay visible in output even though the command currently exits 0
5. **Error messages** include suggested next steps — suggestions must name
   real commands

JSON is the default script contract for query commands, with a documented
plain-text exception for `biomcp cache path`. `biomcp cache stats`,
`biomcp cache clean`, and `biomcp cache clear` support `--json` on success,
while `cache clear` still refuses non-TTY destructive runs unless `--yes` is
present. The cache family remains CLI-only because revealing workstation-local
filesystem paths over MCP would cross the runtime security boundary.

`biomcp version` also supports `--json` for release identity (`version`,
`git_revision`, `build_timestamp`), while `--verbose` remains the plain-text
executable provenance/PATH diagnostic mode.

`search drug --json` is the region-aware exception inside the otherwise flat
search-wrapper family. Drug search has heterogeneous U.S./EU/WHO row schemas,
so its stable contract uses top-level `region`, top-level `regions`, and
per-region `pagination` / `count` / `results` buckets instead of one shared
top-level `results` array.

## See Also and Next Commands

BioMCP uses result-local guidance to teach the next executable step directly
from the current output.

- Entity-card markdown uses `related_*()` helpers in
  `src/render/markdown/related.rs` plus `format_related_block()` to render
  `See also:` follow-up commands at the bottom of `get` cards.
- Structured output carries the same follow-up contract in
  `_meta.next_commands` from `src/render/json.rs` for agent and script
  consumers.
- Workflow ladders are a separate JSON-only contract: `_meta.workflow` names one
  sidecar-backed workflow and `_meta.ladder[]` carries the static multi-step
  worked-example path loaded from `skills/use-cases/<slug>.ladder.json`.
- Ladder commands are byte-equal to the matching `biomcp skill <slug>` fenced
  bash commands. They are not templated; runtime code chooses a workflow slug
  and loads commands from the sidecar.
- Zero-result disease and drug searches use `discover_try_line()` to emit
  `Try: biomcp discover ...` routing. That is related next-step guidance, but
  it is not the same renderer as the entity-card `See also:` block.
- The contract is executability, not string presence: output should teach the
  next executable step; degrade by omission, not by emitting dead commands.
- This guidance is data-driven. `related_*()` helpers only emit commands when
  the supporting data or capability exists for the current record and runtime.
- Representative proof lives in `spec/entity/gene.md`,
  `spec/entity/variant.md`, `spec/entity/article.md`, `spec/entity/trial.md`,
  `spec/entity/drug.md`, `spec/entity/disease.md`, and
  `spec/entity/protein.md`, plus the sidecar contract tests in
  `tests/test_skill_prompt_contract.py` and the parser-level
  `next_commands_validity` tests in `src/cli/tests/`.

## Skills Quick Reference

Overview: `biomcp skill` (prints the embedded `SKILL.md` guide)

Render: `biomcp skill render`

Install: `biomcp skill install ~/.claude --force`

List: `biomcp skill list`

Open: `biomcp skill 01` or `biomcp skill article-follow-up`

Install output lands in `skills/biomcp/` and currently includes `SKILL.md`,
`use-cases/`, `jq-examples.md`, `examples/`, and `schemas/`. The installer auto-discovers existing config
directories (`.claude`, `.agents/skills/`, etc.) when no directory is passed.

MCP resources include `biomcp://skill/<slug>` for each embedded worked example.
