# Adverse Event

Use adverse-event commands for safety surveillance across four source-backed
paths:

- OpenFDA FAERS drug adverse-event reports,
- CDC WONDER VAERS aggregate vaccine adverse-event summaries,
- recall notices,
- device events.

Vaccine searches default to combined OpenFDA FAERS + CDC VAERS when the query resolves to a vaccine and the active filters are VAERS-compatible. Non-vaccine searches stay FAERS-only in practice. `--source vaers` is aggregate-only, accepts only the vaccine query and `--limit`, and rejects `--reaction`, `--outcome`, `--serious`, `--date-from`, `--date-to`, `--suspect-only`, `--sex`, `--age-min`, `--age-max`, `--reporter`, `--count`, `--offset`, recall classification, and device fields.

## Search FAERS reports

By drug:

```bash
biomcp search adverse-event --drug pembrolizumab --limit 5
```

Serious reports only:

```bash
biomcp search adverse-event --drug pembrolizumab --serious --limit 5
```

Reaction-focused filter:

```bash
biomcp search adverse-event --drug pembrolizumab --reaction pneumonitis --limit 5
```

FAERS counts are FAERS-only and therefore require `--source faers`; count operations require offset zero:

```bash
biomcp search adverse-event --drug pembrolizumab --source faers --count reaction --limit 10
```

## Search vaccine events with VAERS

Combined FAERS + VAERS for vaccine queries:

```bash
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5
```

VAERS-only aggregate summary:

```bash
biomcp search adverse-event "MMR vaccine" --source vaers --limit 5
```

VAERS summaries are aggregate-only. They surface the matched vaccine display name, CDC WONDER code, CVX code(s) when the query resolves through the CDC CVX/MVX bridge, serious vs non-serious counts, age distribution, and top reaction counts. `--limit` bounds the number of top reaction rows.

`--source` only applies to `--type faers`; recall and device searches keep
their existing source-specific paths.

## Search recall notices

```bash
biomcp search adverse-event --type recall --drug metformin --limit 5
```

Classification filter:

```bash
biomcp search adverse-event --type recall --drug metformin --classification "Class I" --limit 5
```

## Search device events (MAUDE)

```bash
biomcp search adverse-event --type device --device "insulin pump" --limit 5
```

Manufacturer filter:

```bash
biomcp search adverse-event --type device --manufacturer Medtronic --limit 5
```

Product-code filter:

```bash
biomcp search adverse-event --type device --product-code PQP --limit 5
```

`--manufacturer` and `--product-code` are valid only with `--type device`.

Device seriousness is typed: bare `--serious` and `--serious any` select Death or Injury and render as `death_or_injury`, `--serious death` selects Death only, and `--serious injury` selects Injury only. FAERS seriousness values such as `hospitalization` are rejected on the device route.

## Get a report by ID

```bash
biomcp get adverse-event 10222779
```

Report resolution is source-aware and returns the corresponding markdown format.

## Request report sections

| Section | Description |
|---------|-------------|
| `reactions` | Adverse reactions reported |
| `outcomes` | Reaction outcomes (death, hospitalization, etc.) |
| `concomitant` | Concomitant medications |
| `guidance` | Safety guidance and labeling |
| `all` | Include all sections |

```bash
biomcp get adverse-event 10222779 reactions outcomes
biomcp get adverse-event 10222779 all
```

## Helper commands

There is no direct `adverse-event <helper>` family. Use
`biomcp drug adverse-events <name>` when you want the inbound drug pivot into
this safety surface.

## JSON mode

```bash
biomcp --json get adverse-event 10222779
```

## Combined-source outcomes

JSON from `search adverse-event <query> --source all` includes independent `section_outcomes.faers` and `.vaers` entries while preserving `vaers.status`. One branch can be `unavailable` without erasing a healthy result from the other; `empty` means that source completed successfully with no matching evidence. A successful VAERS branch credits both CDC CVX identity resolution and CDC VAERS evidence. When `--source all` includes FAERS-only filters or a nonzero offset, `vaers.status` explains the skip and its section outcome remains `not_requested` because no VAERS fetch ran. Device-search JSON includes the effective query summary, including the selected seriousness meaning.

## Practical tips

- Include drug generic names for better FAERS recall.
- Use plain vaccine names, family names, or common brand names when you want the VAERS path.
- Treat FAERS rows and VAERS aggregate counts as signal, not incidence estimates.
- Validate serious findings through full source documents when needed.

## Related guides

- [Drug](drug.md)
- [FAQ](../reference/faq.md)
- [Troubleshooting](../troubleshooting.md)
