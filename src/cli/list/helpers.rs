//! Shared helper and root command-reference pages for `biomcp list`.

const LIST_REFERENCE: &str = include_str!("../list_reference.md");
pub(super) fn list_all() -> String {
    let has_oncokb = std::env::var("ONCOKB_TOKEN")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let mut out = LIST_REFERENCE.to_string();

    if has_oncokb {
        out = out.replace(
            "- `variant articles <id>`\n",
            "- `variant articles <id>`\n- `variant oncokb <id>`\n",
        );
    }
    out
}

pub(super) fn list_discover() -> String {
    r#"# discover

## Commands

- `discover <query>` - resolve a free-text biomedical phrase into a primary concept and suggested BioMCP follow-up commands
- `--json discover <query>` - emit structured concepts plus discover-specific `_meta` metadata for agents

## When to use this surface

- Use `discover` when you only have free text and need BioMCP to resolve the first entity or alias before choosing the next typed command.
- Discover is primarily a single-entity resolver for aliases, brands, symptoms, and close concept names.
- Prefer the first suggested command when the query clearly implies treatment, symptoms, safety, trials, or gene+disease orientation.
- Existing routed exceptions remain supported for symptom-of-disease prompts, HPO symptom bridging, treatment prompts, gene+disease orientation, and unambiguous gene-plus-topic follow-ups.
- Relational or multi-entity questions may redirect to `biomcp search all --keyword "<query>"`.
- Unambiguous gene-plus-topic queries can also surface `biomcp search article -g <symbol> -k <topic> --limit 5` when the remaining topic is meaningful.
- If no biomedical entities resolve, discover suggests `biomcp search article -k <query> --type review --limit 5`.
- If only low-confidence concepts resolve, discover adds a broader-results article-search hint.
"#
    .to_string()
}

pub(super) fn list_suggest() -> String {
    r#"# suggest

## Commands

- `suggest <question>` - route a biomedical question to one shipped BioMCP worked-example playbook
- `--json suggest <question>` - emit the four-field response for agents

## Output fields

- `matched_skill` - playbook slug, or no match
- `summary` - short routing explanation
- `first_commands` - exactly two starter commands on match; none on no-match
- `full_skill` - `biomcp skill <slug>` for the full playbook, or none

## When to use this surface

- Use `suggest` when you know the biomedical question but not the first command sequence.
- Matched responses are offline and deterministic; they do not call upstream APIs.
- No-match stays successful and reports `No confident BioMCP skill match`.
- Use `discover "<question>"` when you need entity resolution rather than playbook selection.
"#
    .to_string()
}

pub(super) fn list_batch() -> String {
    r#"# batch

## When to use this surface

- Use batch when you already have a short list of IDs and want the same `get` call repeated consistently.
- Batch is better than sequential `get` calls when you are comparing a few entities side by side.

## Command

- `batch <entity> <id1,id2,...>` - parallel `get` operations for up to 10 IDs

## Options

- `--sections <s1,s2,...>` - request specific sections on each entity
- `--source <ctgov|nci>` - trial source when `entity=trial` (default: `ctgov`)

## Supported entities

- `gene`, `variant`, `article`, `trial`, `drug`, `disease`, `pgx`, `pathway`, `protein`, `adverse-event`

## Examples

- `batch gene BRAF,TP53 --sections pathways,ontology`
- `batch trial NCT04280705,NCT04639219 --source nci --sections locations`
"#
    .to_string()
}

pub(super) fn list_enrich() -> String {
    r#"# enrich

## When to use this surface

- Use enrich when you already have a gene set and need pathways, GO terms, or broader functional categories.
- Start using enrichment once you have 3 or more genes; smaller lists are often better handled by direct `get gene` review.

## Command

- `enrich <GENE1,GENE2,...>` - gene-set enrichment using g:Profiler

## Options

- `--limit <N>` - max number of returned terms (must be 1-50; default 10)

## Examples

- `enrich BRAF,KRAS,NRAS`
- `enrich EGFR,ALK,ROS1 --limit 20`
"#
    .to_string()
}

pub(super) fn list_search_all() -> String {
    r#"# search-all

## Command

- `search all` - cross-entity summary card with curated section fan-out

## Slots

- `--gene` (or `-g`)
- `--variant` (or `-v`)
- `--disease` (or `-d`)
- `--drug`
- `--keyword` (or `-k`)

## Output controls

- `--since <YYYY|YYYY-MM|YYYY-MM-DD>` - applies to date-capable sections
- `--limit <N>` - rows per section (default: 3)
- `--counts-only` - markdown keeps section counts and follow-up links without row tables; `--json` omits per-section results and links
- `--debug-plan` - include executed leg/routing metadata in markdown or JSON
- `--json` - machine-readable sections; in `--counts-only` mode sections carry metadata and counts only

## Notes

- At least one typed slot is required.
- Unanchored keyword-only dispatch is article-only.
- Keyword is pushed into drug search only when `--gene` and/or `--disease` is present.

## Understanding the Output

- Section order follows anchor priority: gene, disease, drug, variant, then keyword-only.
- `get.top` links open the top row as a detailed card.
- `cross.*` links pivot to a related entity search.
- `filter.hint` links show useful next filters for narrowing.
- `search.retry` links appear when a section errors or times out.
- In `--json --counts-only`, per-section follow-up links are omitted; markdown counts-only keeps them.
- Typical workflow: `search all` -> `search <entity>` -> `get <entity> <id>` -> helper commands.
"#
    .to_string()
}
