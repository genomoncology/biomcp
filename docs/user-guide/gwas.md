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

Key flags: `-g/--gene` for a gene symbol, `--trait` for phenotype text,
`--region` for genomic intervals like `chr:start-end`, and `--p-value` for a
finite significance threshold greater than 0 and at most 1. Use `--limit` and
`--offset` for bounded paging.

## Get records

GWAS is search-only. There is no `get gwas` subcommand.

## Request sections

GWAS search rows do not expose extra section names. Use `biomcp get variant
<id> gwas` when you need GWAS evidence attached to a known variant card.

## Helper commands

GWAS is search-only. Start with `search gwas` for genes, traits, or regions,
then pivot into `get variant <id> gwas` if a specific association needs deeper
context.

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
