# How to: orient with `search all`

`search all` is the counts-first orientation command for cross-entity work.
Use it when you know the biomedical concept you care about but do not yet know
which BioMCP entity is the best next stop.

It fans out across relevant sections based on the slots you provide, then helps
you choose a narrower follow-up command.

## Start with typed slots

Named slots are the primary contract because they make your intent explicit:

```bash
biomcp search all --gene BRAF --disease melanoma
biomcp search all --drug pembrolizumab
biomcp search all --variant "BRAF V600E"
biomcp search all --keyword "checkpoint inhibitor"
```

`--keyword` is the orientation leg. `search all` does not spray that term into
every typed query. Trial and drug legs stay driven by their typed slots, while
the article leg keeps the broader keyword context.

Short flags are equivalent where supported:

```bash
biomcp search all -g BRAF -d melanoma
biomcp search all -v "BRAF V600E"
biomcp search all -k "checkpoint inhibitor"
```

## Use `--counts-only` for a low-noise orientation pass

`--counts-only` is the low-noise orientation pass.

In markdown output, it keeps the section totals and follow-up links while
suppressing row tables. In `--json --counts-only`, each section shrinks to
metadata plus orientation counts only, so per-section `results` and `links`
are omitted.

```bash
biomcp search all --gene BRAF --counts-only
biomcp search all --gene BRAF --disease melanoma --counts-only
biomcp search all --drug pembrolizumab --counts-only
biomcp --json search all --gene BRAF --counts-only
```

In markdown output, each section keeps its heading and shows the stable marker
``Rows omitted (`--counts-only`).``

## Use `--debug-plan` to see the executed leg routing

When you need to understand which typed legs ran, which fallback path was used,
or which upstreams fed the result, add `--debug-plan`.

```bash
biomcp search all --gene BRAF --debug-plan
biomcp --json search all --gene BRAF --disease melanoma --debug-plan
```

Markdown prepends a `## Debug plan` fenced JSON block. JSON mode adds the same
payload under `debug_plan`.

When the same normalized token appears in both `--disease` and `--keyword`,
`--debug-plan` marks that the article/orientation leg kept the shared token as
controlled fallback instead of duplicating it across downstream commands.

## Narrow the next command intentionally

After the orientation pass, move to the entity that best matches your question:

```bash
# Trial follow-up after gene+disease orientation
biomcp search trial -c melanoma --biomarker BRAF --status recruiting --limit 5

# Article follow-up after orientation reveals literature volume
biomcp search article -g BRAF -d melanoma --since 2024-01-01 --limit 5

# Helper pivots when you already know the anchor entity
biomcp gene trials BRAF
biomcp variant trials "BRAF V600E" --limit 5
```

## Positional compatibility syntax

The positional form is supported only as a compatibility alias for `--keyword`:

```bash
biomcp search all BRAF
biomcp search all --keyword BRAF
```

Prefer named slots in new docs, scripts, and demos. They are the primary
teaching path and avoid ambiguity about whether a term is a gene, disease,
drug, variant, or general keyword.

## Related

- [CLI Reference - All (cross-entity)](../user-guide/cli-reference.md#all-cross-entity)
- [Quick Reference](../reference/quick-reference.md)
- [Find Trials](find-trials.md)
- [Find Articles](find-articles.md)
