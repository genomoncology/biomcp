# GWAS

Use GWAS commands to search trait-variant associations from the GWAS Catalog.

## Search GWAS

By trait:

```bash
biomcp search gwas --trait "type 2 diabetes" --limit 10
```

By gene:

```bash
biomcp search gwas -g BRAF
```

By genomic region:

```bash
biomcp search gwas --region "chr7:140400000-140500000" --limit 10
```

With p-value threshold:

```bash
biomcp search gwas -g TCF7L2 --p-value 5e-8 --limit 10
```

### Search filters

| Flag | Description |
|------|-------------|
| `-g/--gene` | Gene symbol |
| `--trait` | Trait or phenotype term |
| `--region` | Genomic region (chr:start-end) |
| `--p-value` | P-value significance threshold |
| `-l/--limit` | Max results |
| `--offset` | Pagination offset |

GWAS is search-only. There is no `get` subcommand.

## JSON mode

```bash
biomcp --json search gwas --trait "type 2 diabetes"
```

## Practical tips

- Combine gene and trait filters to narrow broad searches.
- Use `--region` for locus-level queries when you have genomic coordinates.
- GWAS data is also available as a variant section: `biomcp get variant rs7903146 gwas`.

## Related guides

- [Variant](variant.md)
- [Gene](gene.md)
- [Phenotype](phenotype.md)
