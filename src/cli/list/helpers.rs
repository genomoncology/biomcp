//! Shared helper and root command-reference pages for `biomcp list`.

const LIST_REFERENCE: &str = include_str!("../list_reference.md");
pub(super) fn list_all() -> String {
    let has_oncokb = std::env::var("ONCOKB_TOKEN")
        .ok()
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);

    let mut out = LIST_REFERENCE.replace(
        "{{ENTITY_CATALOG}}",
        &super::catalog::render_entity_inventory(),
    );

    if has_oncokb {
        out = out.replace(
            "- `variant articles <id> [--strategy <union|annotation|lexical>]`\n",
            "- `variant articles <id> [--strategy <union|annotation|lexical>]`\n- `variant oncokb <id>`\n",
        );
    }
    out
}

pub(super) fn list_discover() -> String {
    r#"# discover

## Commands

- `discover <query>` - resolve a trimmed free-text biomedical phrase of at most 4,096 UTF-8 bytes into a primary concept and suggested BioMCP follow-up commands
- `--json discover <query>` - emit structured concepts plus discover-specific `_meta` metadata for agents

## Options

- `--limit <N>` - maximum concepts returned; default 5, must be between 1 and 25
- `--offset <N>` - checked zero-based index into the stable ranked concepts
- `--full` - expand the bounded synonym and cross-reference previews

## Output bounds

Compact mode keeps at most 3 synonyms and 5 cross-references per concept, with at most 256 UTF-8 bytes per value and a 32 KiB structured-output budget.

`--full` keeps at most 50 synonyms and 100 cross-references per concept, with at most 512 UTF-8 bytes per value and a 256 KiB structured-output budget.

## When to use this surface

- Use `discover` when you only have free text and need BioMCP to resolve the first entity or alias before choosing the next typed command.
- Discover is primarily a single-entity resolver for aliases, brands, symptoms, and close concept names.
- Prefer the first suggested command when the query clearly implies treatment, symptoms, safety, trials, or gene+disease orientation.
- Existing routed exceptions remain supported for symptom-of-disease prompts, HPO symptom bridging, treatment prompts,
  gene+disease orientation, and unambiguous gene-plus-topic follow-ups.
- Relational or multi-entity questions may redirect to `biomcp search all --keyword "<query>"`.
- Unambiguous gene-plus-topic queries can also surface `biomcp search article -g <symbol> -k <topic> --limit 5` when the remaining topic is meaningful.
- If no biomedical entities resolve, discover suggests `biomcp search article -k <query> --type review --limit 5`.
- If only low-confidence concepts resolve, discover adds a broader-results article-search hint.
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

- `--sections <s1,s2,...>` - request specific sections on each entity; adverse-event batches do not support `--sections`
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

JSON always includes `unresolved_genes`, including an empty array when every input resolves. Markdown prints `Unresolved genes:` before the result table or empty-result message.

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
