# CLI Reference

BioMCP provides one command family with entity-oriented subcommands.

## Global options

- `--json`: return structured JSON output
- `--no-cache`: bypass HTTP cache for the current command

`--json` normally returns structured output, including JSON `error` objects on stdout for BioMCP command errors while preserving nonzero exit codes. Parse/usage errors under `--json` also exit 2 with a JSON `invalid_argument` error on stdout. `biomcp cache path` is a plain-text exception. `biomcp cache stats`, `biomcp cache clean`, and `biomcp cache clear` respect `--json` on success. `biomcp cache clear` still refuses non-TTY destructive runs with plain stderr unless you pass `--yes`.

Once a command is identified, its primary collection path remains present as `[]` on empty success and structured errors (for example, `results`, `concepts`, `edges`, `recommendations`, or a drug region's nested `results`). An empty collection beside `error` means the call failed, not that the biomedical result was negative; scripts must inspect the exit status or `error`. Errors before a command is identified remain keyless. Provider bodies, request URLs, credentials, parser details, and internal local paths are omitted from standard structured errors. Section-shaped `search all`, scalar trial `--count-only`, and VAERS-only aggregate responses keep their existing shapes rather than gaining a false `results` key.

## Core command patterns

```text
biomcp search <entity> [filters]
biomcp get <entity> <id> [section...]
```

Section names are positional trailing arguments after `<id>`. `get article`
also accepts the named `--pdf` modifier, but only together with the `fulltext`
section.

## Evidence metadata

`get` responses include outbound evidence links in markdown output where available.
In JSON mode, links are exposed under `_meta.evidence_urls` and can include
Ensembl, OMIM, NCBI Gene, and UniProt URLs. Section-level provenance is exposed
under `_meta.section_sources`.

## Workflow ladder metadata

Some first-call JSON responses include sidecar-backed workflow ladder metadata:

```json
"_meta": {
  "workflow": "pharmacogene-cumulative",
  "ladder": [
    {
      "step": 1,
      "command": "biomcp search pgx -d warfarin --limit 10",
      "what_it_gives": "CPIC drug-gene rows for known pharmacogenes."
    }
  ]
}
```

`_meta.next_commands` remains the dynamic one-hop HATEOAS follow-up list for the
current response. `_meta.workflow` and `_meta.ladder[]` are static, named
multi-step worked-example paths loaded from installed sidecar JSON files. The
ladder commands are byte-equal to the matching `biomcp skill <slug>` playbook
command block and do not interpolate the user's query.

Examples:

```bash
biomcp search drug --indication "myasthenia gravis" --limit 5 --json
biomcp get drug warfarin --json
biomcp drug interactions warfarin --limit 25 --offset 0 --json
biomcp get drug aspirin --json
```

The warfarin response can emit `pharmacogene-cumulative`; aspirin omits that
workflow ladder when the actionable CPIC A/B pharmacogene threshold is not met.

## Top-level commands

```text
biomcp search ...
biomcp get ...
biomcp discover <query>
biomcp enrich <GENE1,GENE2,...> [--limit N]
biomcp batch <entity> <id1,id2,...> [--sections ...] [--source ...]
biomcp chart [type]
biomcp cache path
biomcp cache stats
biomcp cache clean [--max-age <duration>] [--max-size <size>] [--dry-run]
biomcp cache clear [--yes]
biomcp ddinter sync
biomcp ema sync
biomcp who sync
biomcp cvx sync
biomcp gtr sync
biomcp who-ivd sync
biomcp health [--apis-only]
biomcp list [entity]
biomcp study list
biomcp study download [--list] [<study_id>]
biomcp study filter --study <id> [--mutated <symbol>] [--amplified <symbol>] [--deleted <symbol>] [--expression-above <gene:threshold>] [--expression-below <gene:threshold>] [--cancer-type <type>]
biomcp study query --study <id> --gene <symbol> --type <mutations|cna|expression|sv>
biomcp study cohort --study <id> --gene <symbol>
biomcp study survival --study <id> --gene <symbol> [--endpoint <os|dfs|pfs|dss>]
biomcp study compare --study <id> --gene <symbol> --type <expression|mutations> --target <symbol>
biomcp study co-occurrence --study <id> --genes <g1,g2,...>
biomcp skill
biomcp skill render
biomcp skill install [--force] [dir]
biomcp skill status [dir]
biomcp skill list                 # list embedded worked examples
biomcp mcp
biomcp serve
biomcp mcp-config [--client <client>] [--absolute-path]
biomcp serve-http [--host 127.0.0.1] [--port 8080]
biomcp update [--check] [--allow-missing-checksum]
biomcp uninstall
biomcp version [--verbose]
```

Worked examples are also addressable directly:

```text
biomcp skill 01
biomcp skill article-follow-up
```

`biomcp health --apis-only` is the upstream inventory smoke test. Full
`biomcp health` also reports local readiness rows such as EMA local data,
WHO Prequalification local data, CDC CVX/MVX local data, GTR local data,
WHO IVD local data, cache dir status, and cache-limit warnings when the
managed HTTP cache is over size or below the configured disk-free floor.
With `--json`, the health summary includes numeric `healthy`, `warning`,
`excluded`, `error`, and `total` fields; the four status counts sum to `total`.
Error rows remain report data and do not change the command's current exit
behavior.

`biomcp cache path` is a local-CLI-only operator command. It prints the managed
HTTP cache path as plain text and ignores the global `--json` flag.

`biomcp --json list` emits structured reference data for scripts and agents.
The root object includes `entities`, `commands`, and `patterns`; `biomcp --json list <entity>` emits a per-surface object with `entity` and `commands`.

`biomcp version` prints release identity as plain text by default. With
`--json`, it emits `{ "version": "...", "git_revision": "...",
"build_timestamp": "..." }`; `--verbose` remains the plain-text mode for
executable provenance and PATH diagnostics.

`biomcp cache stats` is the companion local-CLI operator command. It reports the
resolved cache path, total blob inventory, referenced blob bytes used for
enforcement, orphan count, age range, and the resolved cache limits including
`min_disk_free`; under `--json`, it returns the same contract as a JSON object.

`biomcp cache clean [--max-age <duration>] [--max-size <size>] [--dry-run]`
is the targeted maintenance command for the same cache family. It always removes
orphan blobs, can optionally evict entries older than a duration or LRU-evict to
a byte target, and keeps the same structured report under `--json`.

`biomcp cache clear [--yes]` is the destructive sibling for the same managed
HTTP cache tree. It wipes `<resolved cache_root>/http` completely, never touches
the sibling `downloads/` directory, prompts for confirmation when stdin is a
TTY, and refuses non-interactive runs with plain stderr unless you pass
`--yes`. Successful `--json` output uses `{ "bytes_freed": <number|null>,
"entries_removed": <number> }`.

`biomcp update` downloads the matching GitHub release archive and requires
SHA256 checksum verification before replacing the local binary. If a legitimate
release is missing its checksum sidecar, `--allow-missing-checksum` is an
UNSAFE per-invocation override.

## Search command families

## Discover

```bash
biomcp discover ERBB1
biomcp discover "chest pain"
biomcp discover "developmental delay"
biomcp --json discover diabetes
```

Use `discover` when the user starts with free text rather than a known entity
type. It is primarily a single-entity resolver, while keeping the existing
symptom-of-disease, HPO symptom, treatment, gene+disease, and unambiguous
gene-plus-topic routed flows. Relational or multi-entity questions may redirect
to `biomcp search all --keyword "<query>"`. Markdown output groups resolved
concepts by type and suggests concrete follow-up BioMCP commands. JSON adds
`_meta.discovery_sources` alongside the standard `_meta.next_commands` and
`_meta.section_sources` metadata. Symptom-first queries that resolve to HPO
concepts can suggest `biomcp search phenotype "HP:..."` as the first follow-up.

### All (cross-entity)

```bash
biomcp search all --gene BRAF --disease melanoma
biomcp search all --gene BRAF --counts-only
biomcp search all --keyword "immunotherapy resistance" --since 2024-01-01
biomcp search all --gene BRAF --debug-plan
```

See also: [Search All Workflow](../how-to/search-all-workflow.md)

### Gene

```bash
biomcp search gene BRAF --limit 10 --offset 0
```

### Disease

```bash
biomcp search disease -q melanoma --source mondo --limit 10 --offset 0
```

Inheritance accepts named patterns (including broad `dominant`/`recessive`) or HPO
inheritance IDs. Onset accepts antenatal through late onset; `infancy` normalizes
to `infantile`. Unsupported `--inheritance` and `--onset` values fail locally with
`invalid_argument`. Run `biomcp list disease` for the complete catalogs.

### PGx

```bash
biomcp search pgx -g CYP2D6 --limit 10
biomcp search pgx -d warfarin --limit 10
```

`--pgx-testing` accepts `Actionable PGx`, `Informative PGx`, `No Clinical PGx`,
`Testing Recommended`, or `Testing Required`; unsupported values fail locally with
`invalid_argument`. `--evidence` is a best-effort free-text match over guideline
names or CPIC levels.

### Phenotype (Monarch semsim)

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 10
biomcp search phenotype "seizure, developmental delay" --limit 10
```

### GWAS

```bash
biomcp search gwas -g TCF7L2 --limit 10
biomcp search gwas --trait "type 2 diabetes" --limit 10
```

### Article

```bash
biomcp search article -g BRAF -d melanoma --since 2024-01-01 --limit 5 --offset 0
biomcp search article -a "Williams LS" --limit 5
biomcp --json search article -g BRAF --debug-plan --limit 5
biomcp --json search article -k "Oncotype DX review" --session lit-review-1 --limit 5
biomcp --json search article -g BRAF --limit 5 --full
```

JSON article search rows are compact by default and retain available identifiers,
triage fields, source, and tri-state retraction status. `--full` restores
abstracts, complete source provenance, and ranking diagnostics. `--sort date`
replaces relevance ranking and emits an in-band warning in compact JSON, full
JSON, and Markdown. Article tables label mixed PMID/PMCID/DOI/arXiv/Semantic
Scholar values under `Identifier`.

`-a/--author` is an authorship filter. On the default `--source all` route it
limits candidate search to author-capable Europe PMC and, when the other filters
are compatible, PubMed. `--open-access` or `--no-preprints` may narrow further
to Europe PMC. Select either capable source directly when needed; PubTator3,
Semantic Scholar, and LitSense2 reject author filters rather than interpreting
the name as free text.

`-k/--keyword` is provider-neutral text, not raw PubMed or Europe PMC grammar.
Use `--author` or `--journal` for those fields. Recognized provider field
expressions are rejected, while ordinary biomedical bracket and colon notation
remains literal keyword text.

`--session <token>` is article-local and optional. Use it as a short
non-secret local label when a caller may repeat keyword searches for one task;
JSON responses can then add loop-breaker `_meta.suggestions[]` if consecutive
same-session keywords overlap heavily.

### Trial

```bash
biomcp search trial -c melanoma --status recruiting --source ctgov --limit 5 --offset 0
biomcp search trial -c "Rett Syndrome" --limit 20
```

CTGov condition searches send the supplied condition literally. Every
intervention worker is also sent as one quoted literal. Alias expansion uses
plausible trade names and investigational codes while excluding systematic
chemical synonyms. A rejected expanded alias does not discard successful
requested-name results, but leaves the exact total unknown. `--no-alias-expand`
performs one literal request.

For molecular filters, `--mutation <text>` is an exact free-text boolean over
ClinicalTrials.gov title, summary, eligibility, and keyword fields. After broad
discovery, simple mutation text receives a registry eligibility check that removes
exclusion-only matches. Trials where the term is absent remain discoverable, and
boolean expressions are discovery-only. On CTGov,
`--biomarker <text>` is a phrase search over keyword, intervention, and condition
fields; try it for gene-level broadening when a specific `--mutation` search returns
zero rows. Empty filtered
trial searches include broadening follow-ups in `_meta.next_commands` for JSON
callers.

### Variant

```bash
biomcp search variant -g BRAF --hgvsp V600E --limit 5 --offset 0
biomcp search variant -g BRAF --consequence missense_variant --limit 5
biomcp search variant -g BRCA1 --review-status 2 --limit 5
biomcp search variant -g BRAF --has revel --limit 5
biomcp search variant -g BRAF --missing revel --limit 5
```

Consequence, review-status, and `--has`/`--missing` values use the stable
vocabularies printed by `biomcp list variant`. Unknown values fail with a typed
`invalid_argument` error instead of returning a successful empty search.

Protein searches using `--hgvsp` or positional `GENE CHANGE`, plus coding-HGVS and rsID forms, are strict exact searches. Their JSON keeps
the usual top-level `pagination`, `count`, `results`, and `_meta` fields and adds
`requested_variant` plus `resolution`. Each retained row includes its actual
`source_identity` arrays and source-derived `matched_alias`. Exact pagination is
applied after identity filtering: `pagination.total` is the compatible count
when the provider candidate set was examined exhaustively, or `null` when the
1,000-candidate safety cap leaves candidates unexamined. Broad gene-only and
discovery-filter searches keep their existing JSON shape.

### Drug

```bash
biomcp search drug -q "kinase inhibitor" --limit 5 --offset 0
biomcp search drug Keytruda --limit 5
biomcp search drug Keytruda --region eu --limit 5
biomcp search drug "influenza vaccine" --region ema --limit 5
biomcp search drug prevnar --region eu --limit 5
biomcp search drug trastuzumab --region who --limit 5
biomcp search drug BCG --region who --product-type vaccine --limit 5
biomcp search drug --indication malaria --region who --limit 5
```

Drug search JSON is region-aware: the top-level object exposes `region`,
`regions`, and optional `_meta` metadata such as `next_commands`, `workflow`,
and `ladder`. Single-region searches use
`regions.us.results`, `regions.eu.results`, or `regions.who.results`; omitted
`--region` on a plain name lookup and explicit `--region all` expose all three
region buckets, each with `pagination`, `count`, and `results`. Those nested
`results` paths remain present as empty arrays on parsed structured errors; drug
search never gains a false flat top-level `results` key.

For vaccine brand lookups, omitted `--region` on a plain name search and
explicit `--region eu|all` can auto-read the local CDC CVX/MVX bundle after
MyChem identity resolution misses. Explicit WHO vaccine name/brand searches
with `--product-type vaccine` can use the same bridge. The CDC bundle augments
EMA/default vaccine lookups plus explicit WHO vaccine search only; `--region us`
stays U.S.-only and does not touch the CVX root.

### Diagnostic

```bash
biomcp search diagnostic --gene BRCA1 --limit 5 --offset 0
biomcp search diagnostic --disease HIV --source who-ivd --limit 5
biomcp search diagnostic --disease tuberculosis --source all --limit 5
biomcp get gene BRCA1 diagnostics
biomcp get disease tuberculosis diagnostics
biomcp get diagnostic GTR000006692.3 regulatory
biomcp get diagnostic "ITPW02232- TC40" regulatory
```

Diagnostic search is filter-only. At least one of `--gene`, `--disease`,
`--type`, or `--manufacturer` is required, and all provided filters are
conjunctive. `--source` accepts `gtr`, `who-ivd`, or `all` (default). GTR
remains the gene-capable source; WHO IVD supports `--disease`, `--type`, and
`--manufacturer`, and explicit `--source who-ivd --gene ...` fails fast with a
recovery hint. `--disease` requires at least three alphanumeric characters and
matches complete disease words or phrases at boundaries; use `--limit` and
`--offset` for broader diagnostic pages. Diagnostic commands auto-sync the
local GTR bundle into
`BIOMCP_GTR_DIR` and the WHO IVD CSV into `BIOMCP_WHO_IVD_DIR` on first use,
falling back to the default platform data directory when those env vars are
unset.

### Pathway

```bash
biomcp search pathway -q "MAPK signaling" --limit 5 --offset 0
biomcp search pathway -q "Pathways in cancer" --limit 5 --offset 0
```

`search pathway --limit` accepts 1-25; pathway pivot helpers accept 1-50. Out-of-range errors state the range.

### Protein

```bash
biomcp search protein -q kinase --limit 5 --offset 0
biomcp search protein -q kinase --all-species --limit 5
```

### Adverse event

```bash
biomcp search adverse-event --drug pembrolizumab --source faers --serious --limit 5 --offset 0
biomcp search adverse-event --drug osimertinib --count patient.reaction.reactionmeddrapt.exact --limit 10
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5
biomcp search adverse-event "MMR vaccine" --source vaers --limit 5
biomcp search adverse-event --type device --manufacturer Medtronic --limit 5
biomcp search adverse-event --type device --product-code PQP --limit 5
```

FAERS `--count` accepts `reaction` and `reactionmeddrapt` as aliases for
`patient.reaction.reactionmeddrapt.exact`; `patient.reaction.reactionmeddrapt`
and its `.exact` form; `patient.drug.medicinalproduct[.exact]`;
`patient.drug.openfda.generic_name[.exact]`;
`patient.drug.openfda.brand_name[.exact]`; `patient.patientsex`;
`patient.patientonsetage`; `serious`; `seriousnessdeath`;
`seriousnesshospitalization`; `seriousnesslifethreatening`;
`seriousnessdisabling`; `seriousnesscongenitalanomali`; `seriousnessother`;
`patient.reaction.reactionoutcome`; and `primarysource.qualification`. Unsupported
`--count` fields fail locally with `invalid_argument`.

## Get command families

### Gene

```bash
biomcp get gene BRAF
biomcp get gene BRAF pathways ontology diseases protein
biomcp get gene BRAF go interactions civic expression hpa druggability clingen constraint
biomcp get gene BRCA1 diagnostics
biomcp get gene ERBB2 funding
biomcp get gene BRAF all
```

`diagnostics` and `funding` stay opt-in and are not included in
`biomcp get gene <symbol> all`.

### Disease

```bash
biomcp get disease melanoma
biomcp get disease MONDO:0005105 genes phenotypes
biomcp get disease tuberculosis diagnostics
biomcp get disease melanoma clinical_features
biomcp get disease MONDO:0005105 variants models
biomcp get disease MONDO:0005105 pathways prevalence civic survival
biomcp get disease "chronic myeloid leukemia" funding
biomcp get disease "chronic myeloid leukemia" survival
biomcp get disease --name "chronic myeloid leukemia" survival
biomcp get disease MONDO:0005105 all
```

Use `--name` when a multi-word disease name would otherwise be confused with section tokens.
`clinical_features`, `diagnostics`, `disgenet`, and `funding` stay opt-in and
are not included in `biomcp get disease <name_or_id> all`.
Disease diagnostic cards are capped at 10 rows and print a
`search diagnostic --disease <query> --source all --limit 50` follow-up for
broader paged results.

### PGx

```bash
biomcp get pgx CYP2D6
biomcp get pgx codeine recommendations frequencies
biomcp get pgx warfarin annotations
```

### Article

```bash
biomcp get article 22663011
biomcp get article 22663011 indexing
biomcp get article 22663011 all
biomcp get article 22663011 fulltext
biomcp get article 22663011 fulltext --pdf
biomcp --json get article <id> assets
biomcp get article <id> asset <asset-key>
biomcp get article 22663011 tldr
biomcp article batch 22663011 24200969
```

Article detail and batch return every author supplied by the selected source in
source order. JSON carries `authors`, returned `author_count`,
`author_completeness` (`complete`, `source_limited`, or `unavailable`), and
`author_source` (`pubtator` or `europepmc`). Europe PMC display-string lists are
source-limited. Batch keeps its bare-array JSON envelope and request order, and
Markdown cards show authorship plus its status. This compatibility exception is
not converted to an object collection envelope.

`get article <id> indexing` adds PubMed citation authors with nested
source-associated affiliations and optional ORCID plus structured MeSH
headings. Its status distinguishes available-empty from unavailable metadata.
Unavailable JSON and Markdown retain the base article and add a sanitized
`failure.code` plus static message; provider bodies, request URLs, credentials,
and parser internals are never included. PubMed's normal external `DOCTYPE` is
accepted without fetching the DTD, under the existing 8 MiB body bound and a
100,000-node XML bound. The section is opt-in except that `all` includes it, so
ordinary detail, search, and batch avoid the extra PubMed request.

`S2_API_KEY` is optional. With it, BioMCP sends authenticated Semantic Scholar
requests at 1 req/sec for `search article`, `get article`, `get article ... tldr`,
`article batch`, and the explicit `article citations|references|recommendations`
helpers. Without it, those same paths use the shared unauthenticated pool at
1 req/2sec.

For article full text, the default ladder is XML -> PMC HTML. A rung wins only
when its JATS/HTML structure contains article-body content; abstract-only and
metadata-only responses remain healthy partials and later rungs continue. Add
`--pdf` only to `get article <id> fulltext` when you want Semantic Scholar
open-access PDF as the final fallback after XML and HTML do not provide a body.
Requested full-text JSON adds `full_text_coverage` with final
`full_text`/`abstract_only`/`metadata_only`/`none`/`unavailable` coverage and
ordered sanitized attempts (provider, source kind, coverage, outcome, cache
state, and bounded reason). Compatible `full_text_path`, `full_text_source`, and
`full_text_manifest` winner fields remain present only for actual full text;
ordinary article cards omit coverage.

Use `get article <id> assets`
for the JSON article-asset manifest merged from PMC OA, Europe PMC supplementary ZIP,
recognized JATS/PMC HTML links, and eligible Figshare metadata.
Figshare manifests may merge same-paper sibling records discovered by DOI/title;
handles stay as BioMCP commands and provider URLs remain internal. Named linked files
carry typed coverage outcomes. Use `get article <id> asset <asset-key>` to stream
one asset as raw bytes with no conversion. Named-only coverage still returns a
manifest with `assets: []`; an entirely healthy unnamed miss is `not_found`, and
a failed source with no successful fallback is `source_unavailable`.

### Trial

```bash
biomcp get trial NCT02576665
biomcp get trial NCT02576665 contacts eligibility locations
biomcp get trial NCT02576665 all
biomcp --json get trial NCT03361748 documents
biomcp get trial NCT03361748 document Prot_SAP_000.pdf
```

`eligibility` is registry-supplied text and reports CTGov posted-document
availability. The CTGov-only `documents` form is a standalone JSON manifest;
`document <filename>` accepts an exact advertised name and returns raw,
unconverted bytes up to 32 MiB. Documents may contain additional eligibility
detail but do not guarantee that a criterion is resolved, and they remain
outside ordinary `all`.

### Variant

```bash
biomcp get variant "BRAF V600E"
biomcp get variant 'NM_004333.6:c.1799T>A'
biomcp get variant "BRAF V600E" predictions
biomcp get variant "BRAF V600E" predict
biomcp get variant rs7903146 gwas
biomcp variant structure "BRAF V600E"
```

The cached `predictions` section can include REVEL, AlphaMissense, ClinPred,
SIFT, MetaRNN, `BayesDel add-AF`, and `BayesDel no-AF`.
The BayesDel entries are separate source scores; BioMCP does not assign a
clinical threshold or pathogenicity classification to either flavor. The
`predict` section is the separate, credentialed AlphaGenome integration.

Default `get variant` output includes a one-line CIViC actionability pointer from
cached MyVariant data when present, or a `get variant <id> civic` next-command
when not. It does not make the live CIViC GraphQL call unless the `civic` section
or `all` is requested.

`variant structure <variant>` is opt-in. It joins the exact variant to selected
residue, overlapping InterPro domain ranges, UniProt PDB/AlphaFold structures,
Cancerhotspots recurrence, warnings, and `_meta.next_commands`; default
`get variant` output stays compact.

### Drug

```bash
biomcp get drug pembrolizumab
biomcp search drug artesunate --region who --product-type api
biomcp search drug BCG --region who --product-type vaccine
biomcp get drug trastuzumab regulatory --region who
biomcp get drug Keytruda regulatory --region eu
biomcp get drug Dupixent regulatory --region ema
biomcp get drug Ozempic safety --region eu
biomcp get drug carboplatin shortage
biomcp get drug --name "tepotinib hydrochloride" label
```

Use `--name` when a multi-word drug name would otherwise be confused with section tokens.
Omitting `--region` on a plain name/alias `search drug` checks U.S., EU, and
WHO data. If you omit `--region` while using structured filters such as
`--target` or `--indication`, BioMCP stays on the U.S. MyChem path. Explicit
`--region who` filters structured U.S. hits through WHO Prequalification for
finished-pharma/API searches. WHO-only `--product-type
<finished_pharma|api|vaccine>` requires explicit `--region who`. WHO vaccine
search is plain name/brand only, structured WHO filters reject
`--product-type vaccine`, and default WHO search still excludes vaccines unless
you request that product type explicitly.
Explicit `--region eu` or `--region all` with structured filters still errors.
`ema` is accepted as an input alias for the canonical `eu` region value.
Drug search JSON stays under the same top-level `region` + `regions` envelope
for every region mode, so scripts should navigate `regions.<region>.results`
rather than a flat top-level `results` array.
For `get drug`, use `--region` only with `regulatory`, `safety`, `shortage`, or
`all`; WHO currently supports `regulatory` and `all`, while `approvals` stays
U.S.-only. WHO vaccine support in this ticket is search-only, so
`get drug <name> regulatory --region who|all` remains finished-pharma/API only.
If you omit `--region` on `get drug <name> regulatory`, BioMCP checks U.S. and
EU regulatory data. Other no-flag `get drug` shapes stay on the default U.S.
path unless you pass `--region`.

### Diagnostic

```bash
biomcp get diagnostic GTR000006692.3
biomcp get diagnostic GTR000006692.3 regulatory
biomcp get diagnostic "ITPW02232- TC40"
biomcp get diagnostic "ITPW02232- TC40" conditions
biomcp get diagnostic "ITPW02232- TC40" regulatory
biomcp get diagnostic "ITPW02232- TC40" all
```

`get diagnostic` always renders the summary card first. Supported section names
are `genes`, `conditions`, `methods`, `regulatory`, and `all`, but support is
source-aware: GTR supports `genes`, `conditions`, `methods`, and
`regulatory`, while WHO IVD supports `conditions` and `regulatory`. `all`
expands only to the source-native local sections and intentionally excludes the
live FDA overlay. In JSON mode, unrequested sections are omitted while
requested empty sections remain present as `[]`.

### Pathway

```bash
biomcp get pathway R-HSA-5673001
biomcp get pathway R-HSA-5673001 genes
biomcp get pathway hsa05200
biomcp get pathway hsa05200 genes
biomcp get pathway P21964-2        # hints to use `biomcp get protein P21964-2`
biomcp get pathway ENSG00000157764 # hints to use `biomcp get gene ENSG00000157764`
biomcp get pathway BRAF            # hints to use `biomcp get gene BRAF`
biomcp get pathway rs113488022     # hints to use `biomcp get variant rs113488022`
```

Reactome lookup failures for IDs that look like UniProt accessions, Ensembl IDs, gene symbols,
or dbSNP rsIDs include a redirect hint to the matching `get protein`, `get gene`, or
`get variant` command.

### Protein

```bash
biomcp get protein P15056
biomcp get protein P15056 domains interactions
biomcp get protein P15056 complexes
```

### Adverse event

```bash
biomcp get adverse-event 10222779
biomcp get adverse-event 10222779 reactions outcomes
biomcp get adverse-event 10222779 concomitant guidance all
```

## Enrichment

```bash
biomcp enrich BRAF,KRAS,NRAS --limit 10
biomcp enrich BRAF,KRAS,NRAS --limit 10 --json
```

Enrichment JSON always includes `unresolved_genes`; fully resolved input returns
`[]`. Markdown prints `Unresolved genes:` before the result table or empty-result
message so failed symbols are not mistaken for confident empty evidence.

## Batch mode

Batch accepts up to 10 IDs per call and each call must use a single entity type.

```bash
biomcp batch article 22663011,24200969
biomcp batch gene BRAF,TP53
biomcp batch gene BRAF,TP53 --sections pathways,interactions
biomcp batch trial NCT02576665,NCT03715933 --source nci
biomcp batch variant "BRAF V600E","KRAS G12D" --json
```

## MCP mode

- `biomcp serve` runs the stdio MCP server.
- `biomcp mcp-config --client <codex|claude-desktop|claude-code|cursor|cline|vscode|json>` prints copy-paste local stdio MCP client config using `biomcp serve`; add `--absolute-path` when the client cannot see your shell `PATH`.
- `biomcp serve-http` runs the MCP Streamable HTTP server.
- Streamable HTTP clients connect to `/mcp`.
- Probe routes: `/health`, `/readyz`, and `/`.
- `biomcp serve-sse` remains available only as a hidden compatibility command that points users back to `biomcp serve-http`.

See also: `docs/reference/mcp-server.md`.

## Helper command families

```bash
biomcp variant trials "BRAF V600E"
biomcp variant articles "BRAF V600E"
biomcp variant articles "BRAF V600E" --strategy annotation
biomcp variant articles "BRAF V600E" --strategy lexical
biomcp --json variant articles --input variants.json --debug-plan
cat variants.json | biomcp --json variant articles --input -
biomcp variant structure "BRAF V600E"
biomcp variant oncokb "BRAF V600E"
biomcp variant normalize <service> <transcript_hgvs>
biomcp variant normalize all NM_000248.3:c.135del
biomcp variant normalize all 'NM_004448.2:c.829G>T'
```

`variant articles` defaults to a bounded union of exact annotation, normalized
alias, and source-citation routes. It merges route provenance before ranking and
applies pagination once. The `annotation` and `lexical` strategies isolate one
exact route for diagnosis; unresolved default results are explicitly
best-effort. JSON reports route status and marks incomplete acquisition with an
unknown total. Structured `--input <path|->` accepts a JSON array of 1-10 variant
objects and returns ordered compact `items`; it cannot be combined with the
positional ID and requires JSON output. `--debug-plan` is also JSON-only and adds normalized aliases, strict and
discovery provider requests (with exact query/template version), provider/call/page
facts, ranking inputs, and fixed item and request budgets. `--verify-identity`
adds captured-evidence identity facts without changing retrieval recall;
`--confirmed-only` requires verification and filters before rank/pagination.
Query aliases remain retrieval provenance rather than observed evidence, while
identity observations retain source, locator, linked gene, observed alias, and
captured-content hash. Verification artifacts record post-response verifier and
provider-template versions, not retrieval-cache inputs. The typed `variant_articles`
MCP tool accepts the same item fields and equivalent `verify_identity`/`confirmed_only`
controls in memory.

Structured assembly-aware items accept `genomic: "NC_...:g...."` plus `build`,
or `accession`, `position`, `ref`, and `alt` plus `build`. Versioned RefSeq
requires explicit `GRCh37` or `GRCh38`; existing `chrN` identities remain
compatible. RefSeq exact routes contain only caller-present transcript/coding,
gene/coding, and genomic aliases. There is no liftover, accession-to-`chr`
conversion, strand flip, transcript selection, or inferred coordinate alias.

`caller_supplied` means BioMCP accepted the supplied fields as one caller
assertion; it validated syntax but did not establish cross-coordinate
equivalence. `resolution.basis` is `caller_supplied`, `provider_confirmed`, or
`null`. MyVariant `provider_validation` is `confirmed`, `not_found`,
`indeterminate`, `contradictory`, or `unavailable`; `matched_alias` is non-null
only for `confirmed`, and `contradictory_field` only for `contradictory`.
Invalid batch items keep `resolution: null`.

| Validation | Result |
|---|---|
| confirmed | resolved/provider-confirmed; exact routes and source citation |
| not found | RefSeq resolved/caller-supplied; exact routes; citation skipped without degradation |
| indeterminate | RefSeq resolved/caller-supplied; exact routes but incomplete/unknown total |
| contradictory | unresolved/null; no exact route; optional labelled fallback only |
| unavailable | RefSeq resolved/caller-supplied; exact routes but incomplete/truncated |

`biomcp variant normalize ... --json` always writes parseable JSON on exit 0. If no provider returns a normalized form, the payload uses `status: "no_result"`, an empty `results` list, a clear `message`, per-service details, and `_meta.next_commands`.

```bash
biomcp drug interactions warfarin --limit 25 --offset 25
biomcp drug adverse-events pembrolizumab
biomcp drug adverse-events osimertinib --count patient.reaction.reactionmeddrapt.exact
biomcp drug trials pembrolizumab
biomcp disease trials melanoma
biomcp disease trials melanoma --limit 50
biomcp disease drugs melanoma
biomcp disease articles "Lynch syndrome"
biomcp gene trials BRAF
biomcp gene drugs BRAF
biomcp gene articles BRCA1
biomcp gene pathways BRAF
biomcp pathway drugs R-HSA-5673001
biomcp pathway drugs hsa05200
biomcp pathway articles R-HSA-5673001
biomcp pathway trials R-HSA-5673001
biomcp protein structures P15056
biomcp article entities 22663011
biomcp article citations 22663011 --limit 3
biomcp article references 22663011 --limit 3
biomcp article recommendations 22663011 --limit 3
```

## Chart reference

Use `biomcp chart` to list chart families and `biomcp chart <type>` for the
embedded help page for one chart type.

```bash
biomcp chart
biomcp chart violin
```

## Local study analytics

`study` is BioMCP's local cBioPortal analytics family for downloaded
cBioPortal-style datasets.
Unlike the public entity surface, `study` operates on files in your local study
root instead of querying remote APIs for each request.

Use `BIOMCP_STUDY_DIR` when you want an explicit study root for reproducible
downloads and examples; if it is unset, BioMCP falls back to its default study
root. `biomcp study list` is the authoritative local cohort list for the current
snapshot. `biomcp study download --list` shows downloadable IDs, and
`biomcp study download <study_id>` installs a study into that local root. If a
query names a study that is not local, BioMCP returns `not_in_local_cohorts` and
points to `biomcp study download <study_id>` instead of returning an empty cohort
as if it had been analyzed.

| Use this | When |
|----------|------|
| `biomcp search/get/<entity>` | You want discovery or detail across the public entity surface |
| `biomcp study download` | You need to fetch a cBioPortal-style study dataset into your local study root |
| `biomcp study ...` analytics commands | You already have local study files and want cohort, query, survival, compare, or co-occurrence analysis |

### Study command examples

```bash
biomcp study list
biomcp study download --list
biomcp study download msk_impact_2017
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations
biomcp study query --study msk_impact_2017 --gene RET --type sv
biomcp study query --study msk_impact_2017 --gene TP53 --type mutations --chart bar --theme dark --palette wong -o docs/blog/images/tp53-mutation-bar.svg
biomcp study filter --study brca_tcga_pan_can_atlas_2018 --mutated TP53 --amplified ERBB2 --expression-above ERBB2:1.5
biomcp study cohort --study brca_tcga_pan_can_atlas_2018 --gene TP53
biomcp study survival --study brca_tcga_pan_can_atlas_2018 --gene TP53 --endpoint os
biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type expression --target ERBB2
biomcp study compare --study brca_tcga_pan_can_atlas_2018 --gene TP53 --type mutations --target PIK3CA
biomcp study co-occurrence --study msk_impact_2017 --genes TP53,KRAS
```

### Dataset requirements

- `study list` shows locally available studies; this is the local cohort list for coverage decisions.
- `study download` fetches remote datasets into the local study root.
- `study filter` intersects mutation, CNA, expression, and clinical filters.
- `study query` supports `mutations`, `cna`, `expression`, and structural variants/fusions (`sv`, alias `fusion`) per-gene summaries; off-snapshot study IDs return `not_in_local_cohorts` with a `study download` hint.
- Mutation query and top-mutated outputs are mutation-only; when `data_sv.txt` is present, they say fusions/SV are excluded and point to `--type sv`.
- `study cohort`, `study survival`, and `study compare` require `data_mutations.txt` and `data_clinical_sample.txt`.
- `study survival` also requires `data_clinical_patient.txt` with canonical `{ENDPOINT}_STATUS` and `{ENDPOINT}_MONTHS` columns.
- Expression workflows require a supported expression matrix file.

## Author search and detail

```bash
biomcp search author -q "Louis Williams" --source semanticscholar --limit 5 --offset 0
biomcp get author semanticscholar:1716151
```

Author identities are exact Semantic Scholar provider records, not BioMCP-global people. `--affiliation`, PubMed/ORCID author lookup, publications, coauthors, and topics are not available in this release.
