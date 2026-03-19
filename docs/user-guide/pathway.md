# Pathway

Use pathway commands to move from pathway names/IDs to genes, events, enrichment, and drug pivots. Search returns source-labelled Reactome and KEGG rows, and exact title matches are shown first across sources. KEGG base cards stay summary-only unless you explicitly request `genes`. `events` and pathway `enrichment` are Reactome-only. `all` means all sections available for the resolved pathway source.

## Search pathways

```bash
biomcp search pathway -q "MAPK signaling" --limit 5
biomcp search pathway -q "Pathways in cancer" --limit 5
```

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

## Related guides

- [Gene](gene.md)
- [Drug](drug.md)
