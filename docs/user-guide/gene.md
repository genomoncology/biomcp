# Gene

Use gene commands to retrieve canonical metadata and targeted biological context.

## What the gene guide covers

- symbol-based retrieval,
- lightweight search,
- section expansion,
- JSON output for downstream systems.

## Search genes

Start with search when you are unsure of symbol spelling or aliases.

```bash
biomcp search gene -q BRAF --limit 5
```

Useful fields in search output typically include symbol, Entrez ID, and species.

## Get a gene record

```bash
biomcp get gene BRAF
```

The default gene view is concise and intended for orientation.

## Request deeper sections

BioMCP expands detail via positional sections.

Pathway view:

```bash
biomcp get gene BRAF pathways
```

Disease associations:

```bash
biomcp get gene BRAF diseases
```

Ontology terms:

```bash
biomcp get gene BRAF ontology
```

Protein summary:

```bash
biomcp get gene BRAF protein
```

GO terms and interactions:

```bash
biomcp get gene BRAF go interactions
```

CIViC evidence summary:

```bash
biomcp get gene BRAF civic
```

Multiple sections can be chained:

```bash
biomcp get gene BRAF pathways diseases
```

## Gene helper commands

```bash
biomcp gene trials BRAF --limit 5
biomcp gene drugs BRAF --limit 5
biomcp gene pathways BRAF
biomcp gene articles BRAF
biomcp gene definition BRAF
```

## Common workflows

### Clinical trial pivot

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
```

### Literature pivot

```bash
biomcp search article -g BRAF -d melanoma --limit 5
```

### Variant pivot

```bash
biomcp search variant -g BRAF --limit 5
```

## JSON mode

Use JSON for pipelines or agent post-processing.

```bash
biomcp --json get gene BRAF
```

## Error handling expectations

If a section name is unsupported, BioMCP returns an explicit unknown-section message
with hints about valid section names.

## Practical tips

- Keep section requests narrow for better focus.
- Start with one section, then add another only if needed.
- Use `search` first when symbol ambiguity is possible.

## Related guides

- [Variant](variant.md)
- [Article](article.md)
- [Trial](trial.md)
- [Protein](protein.md)
- [Progressive disclosure](../concepts/progressive-disclosure.md)
