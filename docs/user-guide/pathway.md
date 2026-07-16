# Pathway

Use pathway commands to move from pathway names or IDs to genes, events, enrichment, and drug pivots.

Key boundaries:

- Search is multi-source across Reactome, KEGG, and WikiPathways.
- Exact title matches surface first across sources.
- KEGG base cards stay summary-only unless you explicitly request `genes`.
- `events` and pathway `enrichment` are Reactome-only.
- `all` means all sections available for the resolved pathway source.

## Search pathways

`QUERY` is required for normal pathway search. `--top-level` is the only queryless search mode.

```bash
biomcp search pathway "MAPK signaling" --limit 5
biomcp search pathway "MAPK signaling pathway" --limit 5
biomcp search pathway -q "Pathways in cancer" --limit 5
biomcp search pathway --top-level --limit 5
```

Exact KEGG title searches can surface organism-specific KEGG rows such as
`hsa04010` for `MAPK signaling pathway` when the live KEGG source participates.

## Get pathway records

```bash
biomcp get pathway R-HSA-5673001
biomcp get pathway hsa05200
```

## Request pathway sections

Genes:

```bash
biomcp get pathway R-HSA-5673001 genes
biomcp get pathway hsa05200 genes
```

Contained events (Reactome only):

```bash
biomcp get pathway R-HSA-5673001 events
```

Gene-set enrichment (Reactome only):

```bash
biomcp get pathway R-HSA-5673001 enrichment
```

All supported sections for the resolved source:

```bash
biomcp get pathway R-HSA-5673001 all
biomcp get pathway hsa05200 all
```

## Helper commands

```bash
biomcp pathway drugs R-HSA-5673001 --limit 5
biomcp pathway drugs hsa05200 --limit 5
biomcp pathway articles R-HSA-5673001
biomcp pathway trials R-HSA-5673001
```

## JSON mode

```bash
biomcp --json get pathway R-HSA-5673001 genes
biomcp --json get pathway hsa05200 genes
```

## Optional-section outcomes

JSON and MCP pathway records include outcomes for source-supported requested
`genes`, `events`, and `enrichment` sections. Unsupported sections remain an
argument error; `not_requested`, `empty`, and `unavailable` are not substitutes
for unsupported capability. Markdown reports requested unavailable sections
in-band.

## Practical tips

- Use `search pathway` when you know the label but not the stable ID.
- Reach for Reactome IDs when you need `events` or pathway `enrichment`.
- Use the helper commands when the next step is drug, article, or trial context rather than more pathway detail.

## Related guides

- [Gene](gene.md)
- [Drug](drug.md)
