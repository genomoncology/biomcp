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

Key flags: `-g/--gene` for the gene symbol, `-d/--drug` for the therapy,
`--cpic-level` for CPIC levels `A|B|C|D`, `--pgx-testing` for testing-related
guidance, and `--evidence` when you want evidence summaries alongside the main
results. Use `--limit` and `--offset` for bounded paging.

## Get PGX records

```bash
biomcp get pgx CYP2D6
```

The base PGX card summarizes affected drugs and the guideline context tied to
the gene or drug you queried.

## Request PGX sections

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

## Helper commands

PGX does not expose a separate helper family. Start with `search pgx` when you
need to find the right anchor, then switch to `get pgx <gene_or_drug>` for the
base card or section-level follow-up.

## JSON mode

```bash
biomcp --json search pgx -g CYP2D6
biomcp --json get pgx CYP2D6 recommendations
```

Requested `frequencies` and `annotations` are also represented in
`section_outcomes`. CPIC frequency aggregation is `degraded` when some additive
lookups fail but usable rows remain, and `unavailable` when no usable rows
survive. PharmGKB annotation failures are `unavailable`; healthy no-results are
`empty`.

## Practical tips

- Start with `search pgx` when you only know the gene or drug and need the matching guideline rows first.
- Use section-specific `get pgx` calls when you need only recommendations, frequencies, guidelines, or annotations.
- Keep CPIC level filters tight when you want high-confidence dosing guidance.

## Related guides

- [Gene](gene.md)
- [Drug](drug.md)
- [Variant](variant.md)
