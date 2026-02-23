# Pathway

Use pathway commands to move from pathway names/IDs to genes, events, enrichment, and drug pivots.

## Search pathways

```bash
biomcp search pathway -q "MAPK signaling" --limit 5
```

## Get pathway records

```bash
biomcp get pathway R-HSA-5673001
```

## Request pathway sections

Genes:

```bash
biomcp get pathway R-HSA-5673001 genes
```

Contained events:

```bash
biomcp get pathway R-HSA-5673001 events
```

Gene-set enrichment:

```bash
biomcp get pathway R-HSA-5673001 enrichment
```

## Helper command

```bash
biomcp pathway drugs R-HSA-5673001 --limit 5
```

## JSON mode

```bash
biomcp --json get pathway R-HSA-5673001 genes
```

## Related guides

- [Gene](gene.md)
- [Drug](drug.md)
