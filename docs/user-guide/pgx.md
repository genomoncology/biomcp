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

With guideline-name and CPIC level filters:

```bash
biomcp search pgx -g CYP2D6 --cpic-level A --evidence guideline --limit 10
```

Key flags: `-g/--gene` for the gene symbol, `-d/--drug` for the therapy,
`--cpic-level` for CPIC levels `A|B|C|D`, and `--pgx-testing` for `Actionable PGx`,
`Informative PGx`, `No Clinical PGx`, `Testing Recommended`, or `Testing Required`.
`--evidence` remains a best-effort free-text match over guideline names or CPIC
levels. Use `--limit` and `--offset` for bounded paging.

## Get PGX records

```bash
biomcp get pgx CYP2D6
```

The default card returns only the first 10 CPIC interactions. Use `--limit`
(1-50) and `--offset` to page that bounded list.

## Request PGX sections

Retrieve detailed PGX data for a gene-drug pair by section.

Interactions explicitly:

```bash
biomcp get pgx CYP2D6 interactions --limit 10 --offset 10
```

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

All sections at once, capped at 50 rows per section:

```bash
biomcp get pgx CYP2D6 --full
```

### Available sections

| Section | Content |
|---------|---------|
| `interactions` | CPIC gene-drug interactions |
| `recommendations` | CPIC dosing recommendations |
| `frequencies` | Allele frequency data |
| `guidelines` | Published clinical guidelines |
| `annotations` | PharmGKB clinical annotations |
| `all` | All sections combined |

A named section returns only identity, provenance, and that section. Multiple
sections may be requested together at offset zero. To follow a continuation,
request its section alone with the advertised `--offset`; nonzero offsets with
multiple sections or `--full` are rejected before provider work.

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
- Use section-specific `get pgx` calls when you need only interactions, recommendations, frequencies, guidelines, or annotations.
- Keep CPIC level filters tight when you want high-confidence dosing guidance.

## Related guides

- [Gene](gene.md)
- [Drug](drug.md)
- [Variant](variant.md)
