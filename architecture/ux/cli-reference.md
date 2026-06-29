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

biomcp get variant "BRAF V600E"              # summary card + cheap CIViC pointer
biomcp get variant "BRAF V600E" civic        # live CIViC evidence + currency caveat
biomcp get variant "BRAF V600E" clinvar population conservation
biomcp get variant NM_004333.6:c.1799T>A
biomcp get article 22663011 tldr
biomcp --json get article <id> assets
biomcp get article <id> asset <name>
biomcp get diagnostic GTR000006692.3 genes conditions
biomcp get diagnostic "ITPW02232- TC40" conditions
biomcp get drug pembrolizumab label targets civic approvals
biomcp get drug --name "tepotinib hydrochloride" label
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get disease --name "chronic myeloid leukemia" survival
biomcp get disease "uterine leiomyoma" clinical_features
biomcp get trial NCT02576665 eligibility locations outcomes
# target rare-disease trial detail contract (ticket 412):
biomcp get trial NCT... locations contacts eligibility
```

The trial detail surface includes a `contacts` section for ClinicalTrials.gov
central contacts and email-bearing site contacts. `locations` and `eligibility`
remain opt-in sections for site detail and criteria text.

The pattern is consistent across the entity command surface: no-section gives
a summary, named sections are additive, and `all` gives the standard default
surface rather than every opt-in section. Article `assets` is JSON-only and provider-labelled (PMC OA first, Figshare
fallback when Semantic Scholar points at supported Figshare metadata, including
same-paper Figshare siblings discovered by DOI/title), while `asset <name>`
streams raw bytes with no conversion for downstream parsers. Asset handles remain
BioMCP commands rather than provider download URLs.
Use `--name` when a multi-word drug or disease name would otherwise be confused with section tokens.
Opt-in sections such as
`clinical_features`, `diagnostics`, `disgenet`, and `funding` still require
explicit naming.

## Article Search Source UX

`biomcp search article` defaults to `--source all` for recall. The federated
article path fans out across PubTator3, Europe PMC, PubMed, and compatible
Semantic Scholar with a 12-second per-source latency bound. LitSense2 is not
part of the default fan-out; use `--source litsense2` explicitly when you want
that source. Explicit single-source calls such as `--source pubmed`, `--source
pubtator`, `--source semanticscholar`, or `--source litsense2` keep their normal source/client behavior.

When a federated source times out or is unreachable, BioMCP returns rows from the
healthy sources and reports the degraded source. Markdown includes source-status
notes, JSON includes `_meta.source_status`, and `--debug-plan` includes the same
per-source status in the article leg.

## Rare-Disease Trial Search Target

Ticket 412 identified a gap in rare-disease trial search: users may know a
syndrome name, a chromosomal-deletion label, a gene handle, a sponsor, or only a
mixed free-text question. The target UX keeps ordinary `search trial` compact,
but adds a shared rare-disease trial plan beneath `discover`, `search trial`,
`gene trials`, and `disease trials`.

Target command flow:

```bash
# orient from mixed free text and suggest typed trial commands
biomcp discover "Phelan-McDermid Syndrome SHANK3 clinical trial"

# search with bounded condition/gene expansion and visible matched labels
biomcp search trial -c "Phelan-McDermid Syndrome" --limit 20
biomcp search trial -c "Phelan-McDermid Syndrome" --no-condition-expand --limit 20
biomcp gene trials SHANK3 --limit 20

# inspect action-critical detail through progressive disclosure
biomcp get trial NCT... locations contacts eligibility

# target opt-in action-summary shape; exact command name will be fixed by the build ticket
biomcp search trial -c "Phelan-McDermid Syndrome" --action-summary --state MI --limit 20
```

UX invariants for this target:

- expansion is bounded and visible; output shows which condition/gene label matched a trial;
- strict/literal search remains available through an opt-out flag;
- `discover` may suggest a rare-disease trial plan, but it does not become the trial engine;
- compact search remains the default; site/contact/eligibility detail stays opt-in;
- action summaries rank and caveat only listed ClinicalTrials.gov data, and never imply unlisted pending sites.

See `architecture/functional/rare-disease-trial-workflow.md` for target module boundaries, plan types, and fixture contracts.

## Cross-Entity Pivot Pattern

Pivot helpers allow moving between related entities without rebuilding filters:

```bash
# Variant pivots
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E"
biomcp variant structure "BRAF V600E"
biomcp variant normalize <service> <transcript_hgvs>
biomcp variant normalize all NM_000248.3:c.135del
biomcp variant normalize all NM_004448.2:c.829G>T

# Drug pivots
biomcp drug adverse-events pembrolizumab
biomcp drug adverse-events osimertinib --count patient.reaction.reactionmeddrapt.exact
biomcp drug trials pembrolizumab

# Disease pivots
biomcp disease trials melanoma
biomcp disease trials melanoma --limit 50
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
biomcp get pathway P21964-2        # hints to use `biomcp get protein P21964-2`
biomcp get pathway ENSG00000157764 # hints to use `biomcp get gene ENSG00000157764`
biomcp get pathway BRAF            # hints to use `biomcp get gene BRAF`
biomcp get pathway rs113488022     # hints to use `biomcp get variant rs113488022`

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

# 2. Get variant population data; default cards carry only a cheap CIViC pointer,
#    while full CIViC evidence stays opt-in via `get variant <id> civic`.
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
   produces valid JSON — scripts and agents depend on this. `biomcp --json list`
   and `biomcp --json list <entity>` provide structured command-reference data:
   the root object carries `entities`, `commands`, and `patterns`, while entity
   pages carry `entity` and `commands`. `biomcp cache path` is the documented
   operator-command exception: it stays plain text even under `--json`, while
   `biomcp cache stats` and `biomcp cache clean` keep their normal JSON contracts
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

Legacy helper JSON shapes are documented, not silently normalized in this
release. `article batch --json` remains a bare array of compact article cards;
helper-specific JSON such as `drug interactions --json` and `drug
adverse-events --json` keeps helper-owned fields plus optional `_meta`
follow-ups. `drug adverse-events <name>` accepts the same advertised FAERS
filters as the search footer, including `--count <field>` for server-side
aggregate rankings. `biomcp --json list` and `biomcp --json list <entity>` are command
reference payloads, not query result envelopes.

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
