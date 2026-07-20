# BioMCP CLI Reference (UX Analysis)

This document captures stable CLI ergonomic patterns, demo workflows, and MCP
configuration references. It is a durable UX reference for future design,
documentation, and verification work — not a user manual.

## Command Grammar

```
biomcp search <entity> [filters]      → discovery queries
biomcp get <entity> <id> [sections]   → focused detail
biomcp <entity> <helper> <id>         → cross-entity pivot
biomcp skill list                     → worked-example catalog
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
biomcp cache path             → print the managed HTTP cache path (plain text; ignores `--json`)
biomcp cache stats            → show HTTP cache statistics (JSON supported)
biomcp cache clean            → remove orphan blobs and optionally age- or size-evict the HTTP cache (JSON supported)
biomcp cache clear [--yes]    → destructively wipe the managed HTTP cache tree (JSON success; TTY or `--yes` required)
biomcp gtr sync               → force-refresh the local GTR diagnostic bundle
biomcp who-ivd sync           → force-refresh the local WHO IVD diagnostic CSV
biomcp mcp-config [--client <client>] [--absolute-path] → print local stdio MCP client config
biomcp serve-http            → run the MCP Streamable HTTP server at `/mcp`
```

`biomcp mcp-config --client <codex|claude-desktop|claude-code|cursor|cline|vscode|json>` emits copy-paste local stdio config using `biomcp serve` by default; `--absolute-path` embeds the resolved executable path for clients that cannot see the shell `PATH`.

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
biomcp get variant "BRAF V600E" clinvar population conservation predictions
biomcp get variant 'NM_004333.6:c.1799T>A'
biomcp get article 22663011 tldr
biomcp get article 22663011 indexing
biomcp --json get article <id> assets
biomcp get article <id> asset <name>
biomcp get diagnostic GTR000006692.3 genes conditions
biomcp get diagnostic "ITPW02232- TC40" conditions
biomcp get drug pembrolizumab label targets civic approvals
biomcp get drug --name "tepotinib hydrochloride" label
biomcp get disease "Lynch syndrome" genes phenotypes variants
biomcp get disease --name "chronic myeloid leukemia" survival
biomcp get disease melanoma clinical_features
biomcp get trial NCT02576665 eligibility locations outcomes
biomcp --json get trial NCT03361748 documents
biomcp get trial NCT03361748 document Prot_SAP_000.pdf
```

Variant `predictions` can expose REVEL, AlphaMissense, ClinPred, SIFT,
MetaRNN, `BayesDel add-AF`, and `BayesDel no-AF` from the MyVariant payload.
The two BayesDel flavors remain separate source scores;
BioMCP does not apply clinical thresholds or classify pathogenicity from them.

The trial detail surface includes a `contacts` section for ClinicalTrials.gov
central contacts and email-bearing site contacts. `locations` and `eligibility`
remain opt-in sections for site detail and registry-supplied criteria text.
Eligibility reports posted-document availability without attributing registry
text to a PDF. CTGov `documents` is a standalone JSON manifest, while `document
<filename>` accepts an exact advertised name and streams raw bytes without
conversion, capped at 32 MiB. Posted documents may contain additional detail but
do not guarantee criterion resolution; they stay outside ordinary `all` and are
unavailable for NCI.

The pattern is consistent across the entity command surface: no-section gives
a summary, named sections are additive, and `all` gives the standard default
surface rather than every opt-in section. Article `indexing` is opt-in on an
ordinary detail request because it adds PubMed citation XML retrieval, but it is
included by article `all`; its availability status distinguishes an empty
PubMed record from unavailable metadata. Unavailable indexing preserves the
base article and exposes only a stable failure code and static message in JSON
and Markdown, never raw provider or parser details. Article `assets` is JSON-only
and provider-labelled (PMC OA first, Europe PMC second, then Figshare when
Semantic Scholar points at supported metadata), while `asset <name>` streams raw
bytes with no conversion for downstream parsers. A failed source with no later
winner is unavailable rather than a confirmed miss. Asset handles remain BioMCP
commands rather than provider download URLs.

Article `fulltext` tries XML and then PMC HTML. A source wins only when its
JATS/HTML structure contains article-body content; abstract-only and metadata-only
responses remain partials and later eligible rungs continue. `fulltext --pdf`
opts in to Semantic Scholar PDF only after XML and HTML do not provide a body.
Requested JSON adds `full_text_coverage` with final structural coverage and
ordered sanitized attempts, while actual winners retain the compatible
`full_text_path`, `full_text_source`, and `full_text_manifest` fields.

Use `--name` when a multi-word drug or disease name would otherwise be confused with section tokens.
Opt-in sections such as
`clinical_features`, `diagnostics`, `disgenet`, and `funding` still require
explicit naming.

## Author Identity UX

BioMCP currently ships the Semantic Scholar-only provider-exact slice:

```bash
biomcp search author -q "Louis S Williams" --source semanticscholar
biomcp get author semanticscholar:2269573451
```

Affiliation filtering, PubMed name-only candidates, and helper commands remain
the additive target:

```bash
biomcp author publications semanticscholar:2269573451 --limit 20
biomcp author coauthors semanticscholar:2269573451 --max-publications 100 --offset 0
```

Current author IDs use the `semanticscholar:` provider qualifier, and BioMCP
does not mint a global person ID. Citation-supplied ORCID values remain identity
evidence rather than direct-source author IDs. Future search and detail
distinguish evidence-linked, ambiguous, and name-only results. Publication output is grouped into
independently paged provider corpora, and coauthor/topic output names its bounded
supporting publication set. See
`architecture/functional/author-identity.md` for exact JSON shapes, evidence
rules, source degradation, privacy boundaries, and the incremental build order.

## Article Search Source UX

`biomcp search article` defaults to `--source all` for recall. The compatible
federated article path fans out across PubTator3, Europe PMC, PubMed, and
Semantic Scholar with a 12-second per-source latency bound. The capable Europe
PMC + PubMed fan-out used by default author and publication-type filtering
shares that bound and the visible partial-coverage contract. `-a/--author` is a
capability constraint: it limits default candidate search to Europe PMC and,
when the other selected filters are compatible, PubMed. `--open-access` or
`--no-preprints` can narrow further to Europe PMC. Explicit PubTator3, Semantic
Scholar, and LitSense2 author searches fail before network work rather than
degrading the name to free text. Direct Europe PMC and PubMed author searches
remain available. LitSense2 is not part of the default fan-out; use `--source
litsense2` explicitly for provider-neutral keyword searches.

`-k/--keyword` is provider-neutral text, not raw backend grammar. Recognized
PubMed and Europe PMC author, journal, and affiliation field forms are rejected
with typed-filter or unfielded-keyword guidance; unrelated biomedical brackets
and colons remain literal.

When a federated source times out or is unreachable, BioMCP returns rows from the
healthy sources and reports the degraded source. Markdown includes source-status
notes, JSON includes `_meta.source_status`, and `--debug-plan` includes the same
per-source status in the article leg.

Top-level article-search JSON uses compact rows by default while preserving
identifiers, shortlist fields, retraction state, pagination, warnings, source
status, and executable follow-ups. `--full` restores detailed rows without
changing search or ordering. `--sort date` replaces relevance ranking and is
announced in compact JSON, full JSON, and Markdown. Human-readable search and
related-paper tables use an `Identifier` heading with typed values.

## Trial Search

Trial condition filters are literal. Every CTGov intervention worker is sent as
one quoted literal. Expansion uses plausible trade names and investigational
codes while excluding systematic chemical synonyms. A rejected expanded alias
preserves successful requested-name results and makes the exact total unknown;
`--no-alias-expand` performs one literal request for the supplied name.
`--mutation <text>` remains an exact free-text boolean over title, summary,
eligibility, and keyword text. After broad discovery, simple mutation text receives
a registry eligibility check that removes exclusion-only matches. Trials where the
term is absent remain discoverable, and boolean expressions are discovery-only.
`--biomarker <text>` is the gene-level broadening lever when mutation wording is too
specific; zero-result filtered trial searches do not auto-broaden; markdown and JSON
`_meta.next_commands` suggest which filters to relax. Trial details such as contacts,
locations, and eligibility remain opt-in through `get trial` sections.

## Variant Search Filters

Variant consequence, ClinVar review-status, and field-presence filters use stable
public vocabularies discoverable through `biomcp list variant`. `--consequence`,
`--review-status`, `--has`, and `--missing` reject unsupported values with a typed
`invalid_argument` response rather than treating a typo as a successful empty search.
Provider-specific field paths and review phrases remain behind that public vocabulary.

Protein, coding-HGVS, and rsID exact searches preserve the supplied identity in
`requested_variant` and expose a structured `resolution`. Rows survive only when
their provider facts prove the requested identity; each retained row includes its
complete `source_identity` arrays and source-derived `matched_alias`. Gene-only,
residue-alias, and discovery-filter searches remain broad and omit this exact-match
metadata.

## Cross-Entity Pivot Pattern

Pivot helpers allow moving between related entities without rebuilding filters:

```bash
# Variant pivots
biomcp variant trials "BRAF V600E" --limit 5
biomcp variant articles "BRAF V600E"
biomcp variant structure "BRAF V600E"
biomcp variant normalize <service> <transcript_hgvs>
biomcp variant normalize all NM_000248.3:c.135del
biomcp variant normalize all 'NM_004448.2:c.829G>T'
```

`biomcp variant normalize ... --json` always writes parseable JSON on exit 0. If no provider returns a normalized form, the payload uses `status: "no_result"`, an empty `results` list, a clear `message`, per-service details, and `_meta.next_commands`.

```bash
# Drug pivots
biomcp drug adverse-events pembrolizumab
biomcp drug adverse-events osimertinib --count patient.reaction.reactionmeddrapt.exact
biomcp drug interactions warfarin --limit 25 --offset 25
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
   produces valid JSON — scripts and agents depend on this. Parse/usage errors
   under `--json` exit 2 with a JSON `invalid_argument` envelope on stdout.
   `biomcp --json list` and `biomcp --json list <entity>` provide structured
   command-reference data: the root object carries `entities`, `commands`, and
   `patterns`, while entity pages carry `entity` and `commands`. `biomcp cache
   path` is the documented operator-command exception: it stays plain text even
   under `--json`, while `biomcp cache stats` and `biomcp cache clean` keep their
   normal JSON contracts. After command identification, primary collection paths
   remain iterable as `[]` on empty success and structured errors; `error` and a
   nonzero exit still distinguish failure from a biomedical empty result. Early
   clap failures remain command-agnostic and keyless. Section-shaped `search
   all`, scalar trial `--count-only`, and VAERS-only aggregate responses retain
   their existing non-collection shapes
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
top-level `results` array. Drug `--region ema` is a public alias for the
canonical `--region eu` value on search and get drug regional sections. Parsed
errors retain the applicable nested region result path or all three paths and do
not gain a false flat `results` key.

Legacy helper JSON shapes are documented, not silently normalized in this
release. `article batch --json` remains a bare array of compact article cards in
request order; the generic legacy batch success shape also remains a bare array.
Neither is migrated to an object envelope in this release. Article detail and
batch cards carry every author supplied by the
selected source plus returned count, completeness, and source; Europe PMC
display-string authorship is explicitly source-limited. Helper-specific JSON
such as `drug interactions --json` and `drug adverse-events --json` keeps
helper-owned fields plus optional `_meta` follow-ups. `drug adverse-events <name>` accepts the same advertised FAERS
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


## Author (current provider-exact slice)

- `search author -q <name> [--source semanticscholar] [--limit N] [--offset N]`
- `get author semanticscholar:<id>`
- Output labels the identity as exact-provider and states that BioMCP has not established an ORCID link in this release.
- Affiliation filtering, PubMed name-only candidates, publications, coauthors, and topics remain future work; ORCID remains citation-supplied identity evidence.
