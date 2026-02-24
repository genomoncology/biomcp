# Phenotype

Use phenotype commands to search HPO terms and phenotype-gene associations from the Monarch Initiative.

## Search phenotypes

By HPO identifiers (space-separated):

```bash
biomcp search phenotype "HP:0001250 HP:0001263"
```

By name or keyword:

```bash
biomcp search phenotype seizure
```

Multiple terms with limit:

```bash
biomcp search phenotype "HP:0001250 HP:0001263" --limit 20
```

### Search filters

| Flag | Description |
|------|-------------|
| `terms` | Positional: HPO IDs or keywords (space/comma separated) |
| `-l/--limit` | Max results |
| `--offset` | Pagination offset |

Phenotype is search-only. There is no `get` subcommand.

## JSON mode

```bash
biomcp --json search phenotype "HP:0001250"
```

## Practical tips

- Use HPO IDs for precise lookups when you know the exact term.
- Use plain-text keywords for exploratory searches across phenotype names.
- Combine multiple HPO IDs in a single query to retrieve a phenotype set.

## Related guides

- [Gene](gene.md)
- [Disease](disease.md)
- [GWAS](gwas.md)
