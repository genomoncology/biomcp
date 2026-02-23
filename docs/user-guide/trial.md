# Trial

Use trial commands to search and inspect clinical studies with oncology-focused filters.

## Trial command model

- `search trial` finds candidate studies.
- `get trial <NCT_ID>` retrieves a specific study.
- positional sections expand details.

## Search trials (default source)

ClinicalTrials.gov is the default source.

```bash
biomcp search trial -c melanoma --status recruiting --limit 5
```

Add intervention and phase filters:

```bash
biomcp search trial -c melanoma -i pembrolizumab --phase 3 --limit 5
```

Add biomarker filters:

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
biomcp search trial -c melanoma --biomarker BRAF --limit 5
```

Geographic filtering:

```bash
biomcp search trial -c melanoma --lat 42.36 --lon -71.06 --distance 50 --limit 5
```

When geo filters are set, the search query summary includes `lat`, `lon`, and `distance`.

Prior-therapy filters:

```bash
biomcp search trial -c melanoma --prior-therapies platinum --limit 5
biomcp search trial -c melanoma --line-of-therapy 2L --limit 5
```

## Search trials (NCI source)

Use NCI CTS when needed:

```bash
biomcp search trial -c melanoma --source nci --limit 5
```

For higher limits/reliability, set `NCI_API_KEY`.

## Get a trial by NCT ID

```bash
biomcp get trial NCT02576665
```

The default response summarizes title, status, condition context, and source metadata.

## Request trial sections

Eligibility:

```bash
biomcp get trial NCT02576665 eligibility
```

Locations:

```bash
biomcp get trial NCT02576665 locations
```

Outcomes:

```bash
biomcp get trial NCT02576665 outcomes
```

Arms/interventions:

```bash
biomcp get trial NCT02576665 arms
```

References:

```bash
biomcp get trial NCT02576665 references
```

All sections where supported:

```bash
biomcp get trial NCT02576665 all
```

## Downloaded text and cache

Large text blocks (for example, eligibility text) are cached in the BioMCP download area.
This keeps repeated lookups responsive.

## JSON mode

```bash
biomcp --json get trial NCT02576665
```

## Trial matching tips

- Start broad on condition, then add intervention and biomarker filters.
- Keep limits low while tuning search criteria.
- Use `eligibility` section only when you need raw criteria text.

## Related guides

- [How to find trials](../how-to/find-trials.md)
- [Disease](disease.md)
- [Drug](drug.md)
