# Drug

Use drug commands for medication lookup, interaction review, target-oriented search, and U.S./EU/WHO regulatory context.

## Search drugs

Text query:

```bash
biomcp search drug -q "kinase inhibitor" --limit 5
biomcp search drug Keytruda --limit 5
```

Regional or comparison search:

```bash
biomcp search drug Keytruda --region eu --limit 5
biomcp search drug "influenza vaccine" --region ema --limit 5
biomcp search drug trastuzumab --region who --limit 5
biomcp search drug artesunate --region who --product-type api --limit 5
biomcp search drug BCG --region who --product-type vaccine --limit 5
biomcp search drug Keytruda --region all --limit 5
```

Target-oriented search:

```bash
biomcp search drug --target BRAF --limit 5
```

Indication-oriented search:

```bash
biomcp search drug --indication melanoma --limit 5
biomcp search drug --indication malaria --region who --limit 5
```

`search drug --interactions <drug>` is currently unavailable because the public
data sources BioMCP uses do not expose partner-indexed interaction rows on the
search surface. Use `biomcp drug interactions <name>` or `biomcp get drug
<name> interactions` for the DDInter-backed interaction workflow instead.

Omitting `--region` on a plain name/alias search checks U.S., EU, and WHO data.
If you omit `--region` while using structured filters such as `--target` or
`--indication`, BioMCP stays on the U.S. MyChem path. Explicit `--region who`
filters structured U.S. hits through WHO Prequalification for finished-pharma
and API lookups. `--product-type <finished_pharma|api|vaccine>` is WHO-only
and requires explicit `--region who`. WHO vaccine search is plain name/brand
only, so structured WHO filters reject `--product-type vaccine`, and default
WHO search still excludes vaccines unless you request that product type
explicitly.
Explicit `--region eu` or `--region all` with structured filters still errors.
`ema` is accepted as an input alias for the canonical `eu` region value.
When MyChem cannot resolve a vaccine brand name, omitted `--region` on a plain
name search, explicit `--region eu|all`, and explicit WHO vaccine
name/brand search with `--product-type vaccine` can also use the local CDC
CVX/MVX bundle to expand vaccine aliases. That bridge does not change WHO
finished-pharma/API matching, does not create a WHO vaccine `get drug` detail
surface, and does not run for pure `--region us` searches.

WHO vaccine support in this release is search-only. Use
`search drug <name> --region who --product-type vaccine` for vaccine rows;
`get drug <name> regulatory --region who|all` remains finished-pharma/API only.

## Get a drug record

```bash
biomcp get drug pembrolizumab
```

Default output provides concise identity and mechanism context. Approval-bearing
JSON includes additive `approval_date_raw`, `approval_date_display`, and
`approval_summary` fields, while markdown renders the human-friendly display
date in the base card. Default drug output and the `targets` section keep
generic targets from ChEMBL/Open Targets and may add a separate `Variant
Targets (CIViC): ...` line when CIViC surfaces a variant-specific molecular
profile such as `EGFRvIII`.

## Request drug sections

Supported sections: `label`, `regulatory`, `safety`, `shortage`, `targets`,
`indications`, `interactions`, `civic`, `approvals`, `all`.

FDA label section:

```bash
biomcp get drug vemurafenib label
```

Shortage section:

```bash
biomcp get drug carboplatin shortage
```

Regional regulatory and safety sections:

```bash
biomcp get drug trastuzumab regulatory --region who
biomcp get drug Keytruda regulatory --region eu
biomcp get drug Dupixent regulatory --region ema
biomcp get drug trastuzumab regulatory --region all
biomcp get drug Keytruda regulatory --region all
biomcp get drug Ozempic safety --region eu
biomcp get drug Ozempic shortage --region eu
```

If you omit `--region` on `get drug <name> regulatory`, BioMCP checks U.S. and
EU regulatory data. Other no-flag `get drug` shapes keep the default U.S. path
unless you pass `--region` explicitly.

Targets and indications sections:

```bash
biomcp get drug pembrolizumab targets
biomcp get drug pembrolizumab indications
```

`get drug <name> targets` is a mixed-source workflow:

- Generic targets come from ChEMBL and Open Targets.
- Variant-specific target annotations may be added from CIViC.
- Full CIViC evidence tables remain opt-in via `get drug <name> civic`.

Interactions (DDInter-backed structured report with additive label text when available):

```bash
biomcp get drug warfarin interactions
```

This section now renders DDInter-backed partner rows, source-provided severity
levels, and class rollups from the current local DDInter bundle. When the
current bundle has no matching rows, BioMCP says that truthfully and does not
claim the drug has no clinical interactions.

CIViC evidence and Drugs@FDA approvals:

```bash
biomcp get drug vemurafenib civic
biomcp get drug dabrafenib approvals
```

`approvals` remains a legacy U.S.-only section. Use `regulatory` for the region-aware regulatory view.

## EMA local data setup

EU regional commands read EMA local data from `BIOMCP_EMA_DIR` first, then the
platform data directory (`~/.local/share/biomcp/ema` on typical Linux systems).
On first use, BioMCP auto-downloads the six EMA human-medicines JSON feeds
into that root and refreshes stale files after 72 hours. Use `biomcp ema sync`
to force a refresh at any time. `--region ema` is accepted anywhere BioMCP
documents the canonical `eu` region value.

Manual preseed still works. If you need an offline or pre-populated root, place
these files in the target directory:

- `medicines.json`
- `post_authorisation.json`
- `referrals.json`
- `psusas.json`
- `dhpcs.json`
- `shortages.json`

Confirm local EMA readiness with full health output:

```bash
biomcp health
```

Force-refresh EMA local data manually:

```bash
biomcp ema sync
```

EMA row meanings:

- `configured`: `BIOMCP_EMA_DIR` is set and complete
- `configured (stale)`: `BIOMCP_EMA_DIR` is set and complete, but one or more EMA files are older than the 72-hour refresh window
- `available (default path)`: the default platform data directory contains a complete EMA batch
- `available (default path, stale)`: the default platform data directory contains a complete EMA batch, but one or more EMA files are older than the 72-hour refresh window
- `not configured`: no EMA batch is installed at the default path yet
- `error (missing: ...)`: the EMA directory exists but is missing one or more required files

## WHO Prequalification local data setup

WHO regional searches and regulatory commands read the WHO Prequalification
exports from `BIOMCP_WHO_DIR` first, then the platform data directory
(`~/.local/share/biomcp/who-pq` on typical Linux systems). On first use,
BioMCP auto-downloads the finished-pharmaceutical-products CSV, the
active-pharmaceutical-ingredients CSV, and the vaccine export into that root
and refreshes stale files after 72 hours. Use `biomcp who sync` to force a
refresh at any time.

Manual preseed still works. If you need an offline or pre-populated root,
place these files in the target directory:

- `who_pq.csv`
- `who_api.csv`
- `who_vaccines.csv`

Confirm local WHO readiness with full health output:

```bash
biomcp health
```

Force-refresh WHO local data manually:

```bash
biomcp who sync
```

WHO row meanings:

- `configured`: `BIOMCP_WHO_DIR` is set and complete
- `configured (stale)`: `BIOMCP_WHO_DIR` is set and complete, but at least one WHO export is older than the 72-hour refresh window
- `available (default path)`: the default platform data directory contains a complete WHO root with all three exports
- `available (default path, stale)`: the default platform data directory contains all three WHO exports, but at least one is older than the 72-hour refresh window
- `not configured`: no complete WHO root is installed at the default path yet
- `error (missing: ...)`: the WHO directory exists but is missing one of the required files

## CDC CVX/MVX local data setup

Default plain-name vaccine searches that include the EU path can use local CDC
CVX/MVX data as a vaccine identity bridge when MyChem does not recognize the
brand name directly. BioMCP reads `BIOMCP_CVX_DIR` first, then the platform
data directory (`~/.local/share/biomcp/cvx` on typical Linux systems). On
first use, BioMCP auto-downloads `cvx.txt`, `TRADENAME.txt`, and `mvx.txt`
into that root and refreshes stale files after 30 days. Use `biomcp cvx sync`
to force a refresh at any time.

The CDC bundle only augments omitted `--region` plain-name vaccine searches and
explicit `search drug <name> --region eu|all` vaccine searches after MyChem
identity resolution misses. It also augments explicit WHO vaccine name/brand
searches when you pass `--region who --product-type vaccine`. It does not
inject aliases into WHO finished-pharma/API lookups or `get drug`, and
`--region us` does not create or read the CVX root.

Manual preseed still works. If you need an offline or pre-populated root,
place these files in the target directory:

- `cvx.txt`
- `TRADENAME.txt`
- `mvx.txt`

Confirm local CDC readiness with full health output:

```bash
biomcp health
```

Force-refresh CDC CVX/MVX local data manually:

```bash
biomcp cvx sync
```

CDC row meanings:

- `configured`: `BIOMCP_CVX_DIR` is set and the full CDC bundle is present
- `configured (stale)`: `BIOMCP_CVX_DIR` is set and complete, but at least one CDC file is older than the 30-day refresh window
- `available (default path)`: the default platform data directory contains a complete CDC CVX/MVX bundle
- `available (default path, stale)`: the default platform data directory contains a complete CDC bundle, but at least one file is older than the 30-day refresh window
- `not configured`: no complete CDC CVX/MVX root is installed at the default path yet
- `error (missing: ...)`: the CDC directory exists but is missing one or more required files

## DDInter local data setup

Drug interaction commands read DDInter local data from `BIOMCP_DDINTER_DIR`
first, then the platform data directory (`~/.local/share/biomcp/ddinter` on
typical Linux systems). On first use, BioMCP auto-downloads the eight public
DDInter CSV files into that root and refreshes stale files after 72 hours. Use
`biomcp ddinter sync` to force a refresh at any time. DDInter's own terms warn
that absence from the database does not prove no interaction exists, so BioMCP
keeps empty interaction results scoped to the current local bundle.

Manual preseed still works. If you need an offline or pre-populated root, place
these files in the target directory:

- `ddinter_downloads_code_A.csv`
- `ddinter_downloads_code_B.csv`
- `ddinter_downloads_code_D.csv`
- `ddinter_downloads_code_H.csv`
- `ddinter_downloads_code_L.csv`
- `ddinter_downloads_code_P.csv`
- `ddinter_downloads_code_R.csv`
- `ddinter_downloads_code_V.csv`

Confirm local DDInter readiness with full health output:

```bash
biomcp health
```

Force-refresh DDInter local data manually:

```bash
biomcp ddinter sync
```

DDInter row meanings:

- `configured`: `BIOMCP_DDINTER_DIR` is set and the full DDInter bundle is present
- `configured (stale)`: `BIOMCP_DDINTER_DIR` is set and complete, but at least one DDInter file is older than the 72-hour refresh window
- `available (default path)`: the default platform data directory contains a complete DDInter bundle
- `available (default path, stale)`: the default platform data directory contains a complete DDInter bundle, but at least one file is older than the 72-hour refresh window
- `not configured`: no complete DDInter root is installed at the default path yet
- `error (missing: ...)`: the DDInter directory exists but is missing one or more required files

## Helper commands

Interaction pivot:

```bash
biomcp drug interactions warfarin
biomcp drug interactions imatinib
biomcp --json drug interactions warfarin | jq '._meta.next_commands'
```

`drug interactions <name>` resolves the anchor drug first, reports DDInter-backed
partner rows plus class summaries, and uses helper-specific next commands for
`biomcp get drug <canonical> safety` and
`biomcp search article --drug <canonical> --limit 5`. `get drug <name>
interactions` renders the same interaction report inside the standard drug card.
When the current DDInter download bundle has no matching rows, BioMCP says so
without turning that source empty into a safety claim.

Trial pivot:

```bash
biomcp drug trials pembrolizumab --limit 5
biomcp drug trials daraxonrasib --limit 20
biomcp drug trials daraxonrasib --no-alias-expand --limit 20
```

On `--source ctgov`, `drug trials <name>` inherits the shared trial
intervention alias expansion. Expanded results surface `Matched Intervention`
in markdown and `matched_intervention_label` in JSON when an alternate alias
matched first. Use `--no-alias-expand` to force literal matching.

Safety pivot:

```bash
biomcp drug adverse-events pembrolizumab --limit 5
```

## JSON mode

```bash
biomcp --json get drug pembrolizumab
biomcp --json drug interactions warfarin | jq '._meta.next_commands'
biomcp --json search drug Keytruda --region eu --limit 3 | jq '.regions.eu.results[0].ema_product_number'
biomcp --json search drug Keytruda --region all --limit 3 | jq '.regions | keys'
```

`search drug --json` always returns the same top-level shape: `region`,
`regions`, and optional `_meta.next_commands`. Each region bucket keeps the
single-region wrapper fields `pagination`, `count`, and `results`.

- Use `regions.us.results` for U.S. search rows.
- Use `regions.eu.results` for EMA rows.
- Use `regions.who.results` for WHO Prequalification rows.
- Omitted `--region` on a plain name/alias search and explicit `--region all`
  include all three buckets under `regions`.
- `biomcp --json drug interactions <name>` returns one canonical-anchor
  interaction report with `interactions`, `class_summaries`, and the standard
  `_meta.evidence_urls`, `_meta.section_sources`, and helper-specific
  `_meta.next_commands`.

## Practical tips

- Start with base `get` before requesting heavy sections.
- Use `drug interactions <name>` when the question is explicitly about
  interacting drugs or interaction classes for a known medication.
- Use target filters to narrow crowded drug classes.
- Use `regulatory` with `--region who|all` when you need WHO Prequalification context.
- Use `regulatory`, `safety`, or `shortage` with `--region eu|all` when you need EMA context; `ema` is accepted as an input alias for `eu`.
- Omit `--region` on `get drug <name> regulatory` when you want the default combined U.S. and EU regulatory view.
- Pair drug lookups with trial filters for protocol matching workflows.

## Related guides

- [Adverse event](adverse-event.md)
- [Trial](trial.md)
- [DDInter](../sources/ddinter.md)
- [Data sources](../reference/data-sources.md)
