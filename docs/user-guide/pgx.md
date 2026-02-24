# PGX

Use PGX commands to query pharmacogenomic guidelines and annotations from CPIC and PharmGKB.

## Search PGX

By gene:

```bash
biomcp search pgx -g CYP2D6
```

By drug:

```bash
biomcp search pgx -d codeine
```

With evidence and CPIC level filters:

```bash
biomcp search pgx -g CYP2D6 --cpic-level A --evidence --limit 10
```

### Search filters

| Flag | Description |
|------|-------------|
| `-g/--gene` | Gene symbol |
| `-d/--drug` | Drug name |
| `--cpic-level` | CPIC level (A, B, C, D) |
| `--pgx-testing` | Filter by PGx testing recommendation |
| `--evidence` | Include evidence summaries |
| `-l/--limit` | Max results |
| `--offset` | Pagination offset |

## Get PGX sections

Retrieve detailed PGX data for a gene-drug pair by section.

Dosing recommendations:

```bash
biomcp get pgx CYP2D6 recommendations
```

Allele frequency data:

```bash
biomcp get pgx CYP2D6 frequencies
```

Clinical guidelines:

```bash
biomcp get pgx CYP2D6 guidelines
```

PharmGKB annotations:

```bash
biomcp get pgx CYP2D6 annotations
```

All sections at once:

```bash
biomcp get pgx CYP2D6 all
```

### Available sections

| Section | Content |
|---------|---------|
| `recommendations` | CPIC dosing recommendations |
| `frequencies` | Allele frequency data |
| `guidelines` | Published clinical guidelines |
| `annotations` | PharmGKB clinical annotations |
| `all` | All sections combined |

## JSON mode

```bash
biomcp --json search pgx -g CYP2D6
biomcp --json get pgx CYP2D6 recommendations
```

## Related guides

- [Gene](gene.md)
- [Drug](drug.md)
- [Variant](variant.md)
