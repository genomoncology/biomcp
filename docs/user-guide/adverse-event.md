# Adverse Event

Use adverse-event commands for FDA safety surveillance across three datasets:

- FAERS reports,
- recall notices,
- device events.

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

## Get a report by ID

```bash
biomcp get adverse-event 10222779
```

Report resolution is source-aware and returns the corresponding markdown format.

## JSON mode

```bash
biomcp --json get adverse-event 10222779
```

## Practical guidance

- Include drug generic names for better FAERS recall.
- Treat counts as signal, not incidence estimates.
- Validate serious findings through full source documents when needed.

## Related guides

- [Drug](drug.md)
- [FAQ](../reference/faq.md)
- [Troubleshooting](../troubleshooting.md)
