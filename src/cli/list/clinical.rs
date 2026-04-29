//! Clinical command-reference pages for `biomcp list`.
pub(super) fn list_trial() -> String {
    r#"# trial

## Commands

- `get trial <nct_id>` - protocol card by NCT ID
- `get trial <nct_id> eligibility` - show eligibility criteria inline
- `get trial <nct_id> locations` - site locations section
- `get trial <nct_id> locations --offset <N> --limit <N>` - paged location slice
- `get trial <nct_id> outcomes` - primary/secondary outcomes
- `get trial <nct_id> arms` - arm/intervention details
- `get trial <nct_id> references` - trial publication references
- `get trial <nct_id> all` - include every section
- `search trial [filters]` - search ClinicalTrials.gov (default) or NCI CTS (`--source nci`)

## Useful filters (ctgov)

- `--condition <name>` (or `-c`)
- `--intervention <name>` (or `-i`)
- `--no-alias-expand`
- `--status <status>` (or `-s`)
- `--phase <NA|1|1/2|2|3|4>` (or `-p`)
- `--facility <name>`
- `--age <years>` (decimals accepted, e.g. `0.5`)
- `--sex <female|male|all>`
- `--mutation <text>`
- `--criteria <text>`
- `--biomarker <text>`
- `--sponsor-type <nih|industry|fed|other>`
- `--prior-therapies <text>`
- `--progression-on <drug>`
- `--line-of-therapy <1L|2L|3L+>`
- `--lat <N>` + `--lon <N>` + `--distance <miles>`
- `--results-available`
- `--has-results` (alias)
- `--study-type <interventional|observational|...>`
- `--date-from <YYYY-MM-DD> --date-to <YYYY-MM-DD>`
- `--count-only`
- `--limit <N> --offset <N>`

## CTGov alias expansion

- `--intervention` auto-expands known aliases from the shared drug identity surface on the default CTGov path.
- Expanded rows add `Matched Intervention` in markdown and `matched_intervention_label` in JSON when an alternate alias matched first.
- `--no-alias-expand` forces literal matching.
- `--next-page` is not supported once alias expansion fans out to multiple queries; use `--offset` or `--no-alias-expand`.

## NCI source notes

- `--source nci --condition <name>` first tries to ground the name to an NCI disease ID and falls back to CTS `keyword`; there is no separate NCI keyword flag.
- `--source nci --status <status>` accepts one normalized status at a time and maps it to CTS recruitment or lifecycle filters.
- `--source nci --phase 1/2` maps to CTS `I_II`; `--phase early_phase1` is not supported.
- `--source nci --lat/--lon/--distance` uses direct `sites.org_coordinates_*` CTS filters and serializes distance with the required `mi` suffix.

## JSON Output

- Non-empty `search trial --json` responses include `_meta.next_commands`.
- `get trial --json` can include CTGov source-provided intervention alternate names; See-also and JSON next commands may prefer search/article follow-ups for investigational codes.
- Alias-expanded trial rows may include `matched_intervention_label`.
- The first follow-up drills the top result with `biomcp get trial <nct_id>`.
- `biomcp list trial` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

pub(super) fn list_diagnostic() -> String {
    r#"# diagnostic

## When to use this surface

- Use `search diagnostic` when you need source-native diagnostic inventory from the local GTR and WHO IVD bundles.
- Start with `--gene` for GTR genetic-test questions, or `--disease --source who-ivd` for WHO infectious-disease diagnostics; add `--type` or `--manufacturer` only when narrowing a real result set.
- Use `get diagnostic <id>` for the base summary card, then add `genes`, `conditions`, `methods`, or `regulatory` when you need progressive disclosure.

## Commands

- `get diagnostic <gtr_accession>` - summary card from the local GTR bundle
- `get diagnostic "<who_ivd_product_code>"` - summary card from the local WHO IVD CSV
- `get diagnostic <gtr_accession> genes` - joined gene list from GTR detail data
- `get diagnostic <gtr_accession> conditions` - joined condition list from GTR detail data
- `get diagnostic <gtr_accession> methods` - GTR methods list
- `get diagnostic <id> regulatory` - optional live FDA device 510(k)/PMA overlay matched from source-native diagnostic names
- `get diagnostic "<who_ivd_product_code>" conditions` - WHO target/marker section
- `get diagnostic <id> all` - include every section supported by the resolved source
- `search diagnostic --gene <symbol>` - case-insensitive exact gene match
- `search diagnostic --disease <name> --source who-ivd` - minimum-length word/phrase match over WHO pathogen/disease/marker
- `search diagnostic --disease <name> --source gtr` - minimum-length word/phrase match over GTR condition names
- `search diagnostic --type <test_type> --source <gtr|who-ivd|all>` - case-insensitive exact type filter
- `search diagnostic --manufacturer <name> --source <gtr|who-ivd|all>` - case-insensitive substring over manufacturer/lab labels
- `search diagnostic ... --limit <N> --offset <N>` - offset pagination with `1..=50` result limits

## Search rules

- At least one of `--gene`, `--disease`, `--type`, or `--manufacturer` is required.
- All provided filters are conjunctive.
- `--disease` must contain at least 3 alphanumeric characters and matches full words or phrases at boundaries; short noisy tokens are rejected.
- `--source` accepts `gtr`, `who-ivd`, or `all` (default).
- Explicit `--source who-ivd --gene ...` is invalid; use `--source gtr` or omit `--source` for gene-first workflows.
- Use `--limit` and `--offset` to page broader diagnostic result sets beyond capped disease cards.
- Result ordering is deterministic: normalized test name ascending, then accession ascending.
- `summary` is always part of `get diagnostic`; supported public section tokens are `genes`, `conditions`, `methods`, `regulatory`, and `all`.
- Source-aware section support: GTR supports `genes`, `conditions`, `methods`, and `regulatory`; WHO IVD supports `conditions` and `regulatory`.
- `all` stays source-aware but intentionally excludes `regulatory` because the FDA overlay is live and opt-in.

## JSON Output

- Non-empty `search diagnostic --json` responses include `_meta.next_commands`.
- The first follow-up drills the top result with `biomcp get diagnostic <id>` and quotes WHO product codes that contain spaces.
- Non-empty next commands include `biomcp list diagnostic` so agents can inspect the full filter surface.
- True zero-result `search diagnostic --json` responses keep `count: 0`, `results: []`, and truthful pagination while adding `_meta.suggestions`.
- Zero-result suggestions include `biomcp list diagnostic` so agents can inspect source-aware diagnostic filters and local GTR/WHO IVD usage.
- `get diagnostic --json` keeps section-aware follow-ups and `_meta.section_sources`.
- `get diagnostic --json ... regulatory` adds a top-level `regulatory` field; omitting the section omits the field, and no FDA match serializes `regulatory: []`.

## Local data

- BioMCP auto-downloads GTR local data on first diagnostic use into `BIOMCP_GTR_DIR` or the default platform data directory.
- BioMCP auto-downloads WHO IVD local data on first WHO diagnostic use into `BIOMCP_WHO_IVD_DIR` or the default platform data directory.
- Full `biomcp health` reports `GTR local data (<resolved_root>)` and `WHO IVD local data (<resolved_root>)`; `biomcp health --apis-only` intentionally omits them.
- Use `biomcp gtr sync` to force-refresh the local GTR bundle.
- Use `biomcp who-ivd sync` to force-refresh the local WHO IVD CSV.
"#
    .to_string()
}

pub(super) fn list_drug() -> String {
    r#"# drug

## When to use this surface

- Use the positional name lookup when you already know the drug or brand name.
- Use `--indication`, `--target`, or `--mechanism` when the question is structured.
- Use `get drug <name>` for label, regulatory, safety, target, or indication detail after you have the normalized drug name.

## Commands

- `get drug <name>` - get by name (MyChem.info aggregation)
- `get drug <name> label [--raw]` - compact FDA approved-indications summary by default; add `--raw` for the truncated FDA label text
- `get drug <name> regulatory [--region <us|eu|who|all>]` - regional regulatory summary (Drugs@FDA, EMA, and/or WHO Prequalification)
- `get drug <name> safety [--region <us|eu|all>]` - regional safety context (OpenFDA and/or EMA)
- `get drug <name> shortage [--region <us|eu|all>]` - query current shortage status
- `get drug <name> targets` - generic targets from ChEMBL/OpenTargets plus additive CIViC variant-target annotations when available
- `get drug <name> indications` - enrich with OpenTargets indications
- `get drug <name> interactions` - DDInter-backed structured interaction rows plus class rollups; empty states stay scoped to the current DDInter download bundle
- `get drug <name> civic` - CIViC therapy evidence/assertion summary
- `get drug <name> approvals` - Drugs@FDA approval/application details (US-only legacy section)
- `get drug <name> all [--region <us|eu|who|all>]` - include all sections

## Search

- `search drug <query>`
- `search drug -q <query>`
- `search drug <query> --region <us|eu|who|all>`
- `search drug <query> --region who --product-type <finished_pharma|api|vaccine>`
- `search drug --target <gene>`
- `search drug --indication <disease>`
- `search drug --indication <disease> --region who --product-type <finished_pharma|api>`
- `search drug --mechanism <text>`
- `search drug --atc <code>`
- `search drug --pharm-class <class>`
- `search drug --interactions <drug>` - unavailable from current public data sources; use `drug interactions <name>` for structured DDI lookup
- `search drug ... --limit <N> --offset <N>`

## Helpers

- `drug trials <name> [--no-alias-expand]`
- `drug interactions <name>` - DDInter-backed structured drug-drug interactions with partner rows, class summaries, and helper-specific JSON follow-ups
- `drug adverse-events <name>` - checks FAERS first, distinguishes FAERS 404 from FAERS 200+empty results, and falls back to ClinicalTrials.gov trial-reported adverse events only on FAERS 404

## JSON Output

- `search drug --json` responses use a region-aware envelope: top-level `region`, top-level `regions`, and optional top-level `_meta`.
- Single-region searches expose one bucket under `regions.us`, `regions.eu`, or `regions.who`.
- Omitted `--region` on plain name/alias lookup and explicit `--region all` expose `regions.us`, `regions.eu`, and `regions.who`.
- Each region bucket keeps `pagination`, `count`, and `results`.
- Non-empty `search drug --json` responses include `_meta.next_commands`.
- Structured indication searches with matching results can also include `_meta.workflow` and `_meta.ladder[]` for the `treatment-lookup` workflow.
- Non-vaccine searches keep `biomcp get drug <name>` as the preferred follow-up; WHO vaccine-only results stay search-only and omit broken `get drug` guidance.
- `biomcp list drug` is always included so agents can inspect the full filter surface.
- `biomcp --json drug interactions <name>` returns the canonical anchor drug, interaction rows, class summaries, and helper-specific `_meta.next_commands` for `biomcp get drug <canonical> safety` plus `biomcp search article --drug <canonical> --limit 5`.
- `biomcp --json drug adverse-events <name>` keeps the FAERS `summary` / `results` / `count` fields, adds `faers_not_found`, and includes `trial_adverse_events` only when the ClinicalTrials.gov fallback returns posted trial adverse-event terms.

## Notes

- Omitting `--region` searches U.S., EU, and WHO data for plain name/alias lookups.
- Structured filters remain U.S.-only when `--region` is omitted.
- Explicit `--region who` filters structured U.S. hits through WHO prequalification.
- `--product-type <finished_pharma|api|vaccine>` is WHO-only and requires explicit `--region who`.
- WHO vaccine search is plain name/brand only; structured WHO filters reject `--product-type vaccine`.
- Default WHO search excludes vaccines unless you explicitly request `--product-type vaccine`.
- Explicit `--region eu|all` is still invalid with structured filters.
- `ema` is accepted as an input alias for the canonical `eu` drug region value.
- Omitting `--region` on `get drug <name> regulatory` is the one implicit combined-region get path; other no-flag `get drug` shapes stay on the default U.S. path.
- WHO vaccine support in this ticket is search-only; `get drug <name> regulatory --region who|all` remains finished-pharma/API only.
- `drug trials <name>` inherits CTGov intervention alias expansion, adds `Matched Intervention` / `matched_intervention_label` when an alternate alias matched first, and accepts `--no-alias-expand` for literal matching.
- Drug interaction commands auto-download the DDInter CSV bundle into `BIOMCP_DDINTER_DIR` or the default data directory on first use; empty results stay scoped to the current DDInter bundle instead of claiming clinical safety.
- `drug adverse-events <name>` explains when a drug is absent from FAERS versus present with no matching FAERS events; only the FAERS-404 branch queries ClinicalTrials.gov.
- EU regional commands auto-download the EMA human-medicines JSON feeds into `BIOMCP_EMA_DIR` or the default data directory on first use.
- Default/EU vaccine brand lookups and explicit WHO vaccine name/brand searches can also auto-download the CDC CVX/MVX bundle into `BIOMCP_CVX_DIR` or the default data directory on first use.
- WHO regional commands auto-download the WHO finished-pharma, API, and vaccine CSV exports into `BIOMCP_WHO_DIR` or the default data directory on first use (`who_pq.csv`, `who_api.csv`, and `who_vaccines.csv`).
- Run `biomcp ddinter sync`, `biomcp ema sync`, `biomcp cvx sync`, `biomcp who sync`, `biomcp gtr sync`, or `biomcp who-ivd sync` to force-refresh the local runtime data.
"#
    .to_string()
}

pub(super) fn list_disease() -> String {
    r#"# disease

## When to use this surface

- Use `get disease <name_or_id>` when you want the normalized disease card with genes, pathways, and phenotypes.
- Use `get disease <name_or_id> diagnostics` when you need a capped diagnostic-test card from local GTR and WHO IVD data.
- Use `get disease <name_or_id> funding` when the question is about NIH grant support for a disease.
- Use `get disease <name_or_id> survival` when the question is specifically about cancer survival outcomes.
- Use `get disease <name_or_id> phenotypes` for symptom-style questions.
- Use `get disease <name_or_id> clinical_features` only when you need MedlinePlus clinical-summary rows for configured diseases; unsupported diseases omit fabricated rows, and the section stays opt-in.
- Use `search article -d <disease>` when you need broader review literature or want to supplement sparse structured data.

## Commands

- `get disease <name_or_id>` - resolve MONDO/DOID or best match by name with OpenTargets gene scores
- `get disease <name_or_id> genes` - Monarch rows plus additive CIViC/OpenTargets disease-gene associations with merged OpenTargets scores
- `get disease <name_or_id> pathways` - Reactome pathways from associated genes
- `get disease <name_or_id> phenotypes` - HPO phenotypes with resolved names
- `get disease <name_or_id> diagnostics` - up to 10 diagnostic tests for this condition from GTR and WHO IVD
- `get disease <name_or_id> variants` - CIViC disease-associated molecular profiles
- `get disease <name_or_id> models` - Monarch model-organism evidence
- `get disease <name_or_id> prevalence` - OpenTargets prevalence-like evidence
- `get disease <name_or_id> survival` - SEER Explorer 5-year relative survival by sex for mapped cancers
- `get disease <name_or_id> civic` - CIViC disease-context evidence
- `get disease <name_or_id> disgenet` - DisGeNET scored disease-gene associations (requires `DISGENET_API_KEY`)
- `get disease <name_or_id> funding` - NIH Reporter grants for the requested disease phrase, or the resolved canonical name for identifier lookups, over the most recent 5 NIH fiscal years
- `get disease <name_or_id> clinical_features` - MedlinePlus clinical-summary rows for configured diseases; unsupported diseases omit fabricated rows
- `get disease <name_or_id> all` - include all standard disease sections (`diagnostics`, `disgenet`, `funding`, and `clinical_features` stay opt-in)
- `search disease <query>` - positional search by name
- `search disease -q <query>` - search by name
- `search phenotype "<HP terms or symptom phrases>"` - HPO IDs or resolved symptom text to ranked diseases
- `search disease -q <query> --source <mondo|doid|mesh>` - constrain ontology source
- `search disease -q <query> --inheritance <pattern>`
- `search disease -q <query> --phenotype <HP:...>`
- `search disease -q <query> --onset <period>`
- `search disease -q <query> --no-fallback` - skip discover recovery and keep the direct zero-result response
- `search disease ... --limit <N> --offset <N>`

Disease diagnostic cards are capped at 10 rows. When rows exist, the card
prints a `See also:` command such as
`biomcp search diagnostic --disease <query> --source all --limit 50`; continue
with `--offset` on `search diagnostic` for later pages.

## Helpers

- `disease trials <name>`
- `disease articles <name>`
- `disease drugs <name>`

## JSON Output

- Non-empty `search disease --json` responses include `_meta.next_commands`.
- Disease search JSON emits at most one workflow ladder; `mutation-catalog` wins over `trial-recruitment` when both bounded probes match.
- The first follow-up drills the top result with `biomcp get disease <id>`.
- `biomcp list disease` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}

pub(super) fn list_phenotype() -> String {
    r#"# phenotype

## Commands

- `search phenotype "<HP:... HP:...>"` - rank diseases by phenotype similarity
- `search phenotype "<symptom phrase[, symptom phrase]>"` - resolve symptom text to HPO IDs, then rank diseases
- `search phenotype "<HP:...>" --limit <N> --offset <N>` - page ranked disease matches

## Examples

- `search phenotype "HP:0001250 HP:0001263"`
- `search phenotype "HP:0001250" --limit <N> --offset <N>`
- `search phenotype "HP:0001250,HP:0001263" --limit 10`
- `search phenotype "seizure, developmental delay" --limit 10`

## Workflow tips

- Start with 2-5 high-confidence HPO terms when you have them; otherwise use one symptom phrase or comma-separated symptom phrases.
- Use specific neurologic/cancer phenotype terms before broad umbrella terms.
- Run `discover "<symptom text>"` first when you want BioMCP to surface candidate `HP:` terms before ranking diseases.
- Follow with `get disease <id> all` to inspect phenotypes, genes, and pathways.

## Related

- `search disease -q <query> --phenotype <HP:...>`
- `disease trials <name>`
- `disease articles <name>`
"#
    .to_string()
}

pub(super) fn list_adverse_event() -> String {
    r#"# adverse-event

## Commands

- `search adverse-event --drug <name> --source <faers|vaers|all>` - FAERS by default for ordinary drug queries; vaccine-resolved searches can add CDC VAERS
- `search adverse-event <vaccine query> --source vaers` - aggregate CDC WONDER VAERS summary
- `search adverse-event --drug <name> --outcome <death|hospitalization|disability>`
- `search adverse-event --drug <name> --serious <type>`
- `search adverse-event --drug <name> --date-from <YYYY|YYYY-MM-DD> --date-to <YYYY|YYYY-MM-DD>`
- `search adverse-event --drug <name> --suspect-only --sex <m|f> --age-min <N> --age-max <N>`
- `search adverse-event --drug <name> --reporter <type>`
- `search adverse-event --drug <name> --count <field>` - OpenFDA FAERS aggregation mode
- `search adverse-event ... --limit <N> --offset <N>`
- `get adverse-event <report_id>` - retrieve report by ID

## Source behavior

- default `--source all` always runs OpenFDA FAERS and adds CDC VAERS only when the query resolves to a vaccine and the active filters are VAERS-compatible
- `--source vaers` is aggregate-only and supports plain vaccine query text from `--drug` or the positional query
- VAERS intentionally does not support --reaction, --outcome, --serious, --date-from, --date-to, --suspect-only, --sex, --age-min, --age-max, --reporter, --count, or --offset > 0
- `--source` only applies to `--type faers`; recall and device searches keep their existing source-specific paths

## Other query types

- `search adverse-event --type recall --drug <name>` - enforcement/recalls
- `search adverse-event --type device --device <name>` - MAUDE device events
- `search adverse-event --type device --manufacturer <name>` - MAUDE by manufacturer
- `search adverse-event --type device --product-code <code>` - MAUDE by product code

## JSON Output

- Non-empty `search adverse-event --json` responses include `_meta.next_commands`.
- `--source all` keeps the FAERS envelope and adds a truthful `vaers` status block.
- `--source vaers` returns a VAERS-first envelope with `source`, `query`, `vaers`, and `_meta`.
- FAERS and device searches drill the top result with `biomcp get adverse-event <report_id>`.
- Recall searches currently return `biomcp list adverse-event` without a recall-specific `get` command.
- `biomcp list adverse-event` is always included so agents can inspect the full filter surface.
"#
    .to_string()
}
