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

On the default CTGov path, `--intervention` auto-expands known drug aliases
from the shared drug identity surface, unions the matching trials, and shows
which alias matched each returned row.

```bash
biomcp search trial -i daraxonrasib --limit 20
biomcp search trial -i daraxonrasib --no-alias-expand --limit 20
```

When an alternate alias wins, markdown adds a `Matched Intervention` column and
JSON adds `matched_intervention_label`. `--no-alias-expand` forces strict
literal matching. If alias expansion fans out to multiple CTGov queries,
`--next-page` is unavailable; use `--offset` or `--no-alias-expand`.

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

Use NCI CTS when you want the shared BioMCP trial CLI to target the NCI trial
catalog instead of ClinicalTrials.gov.

```bash
biomcp search trial -c melanoma --source nci --limit 5
```

`--condition` remains the NCI entry point. BioMCP first tries to ground the
condition through MyDisease and, when the best match has an NCI Thesaurus
cross-reference, sends `diseases.nci_thesaurus_concept_id=<C-code>`. When no
grounded NCI ID is available, BioMCP falls back to CTS `keyword=<text>`.
There is no separate NCI keyword flag in this ticket.

NCI status handling is source-specific. Use one normalized status at a time:

- `recruiting` maps to CTS `sites.recruitment_status=ACTIVE`
- `not yet recruiting`, `enrolling by invitation`, `active, not recruiting`,
  `completed`, `suspended`, `terminated`, and `withdrawn` map to the closest
  documented CTS lifecycle or site-status value
- comma-separated status lists are rejected for `--source nci`

NCI phase handling is also source-specific:

- `1`, `2`, `3`, and `4` map to CTS `I`, `II`, `III`, and `IV`
- `1/2` maps to CTS `I_II`
- `NA` stays `NA`
- `early_phase1` is rejected for `--source nci`

```bash
biomcp search trial -c melanoma --source nci --status recruiting --phase 1/2 --limit 5
```

NCI geographic filtering is direct CTS filtering rather than CTGov's
geo-verify mode. When `--lat`, `--lon`, and `--distance` are all present,
BioMCP sends `sites.org_coordinates_lat`, `sites.org_coordinates_lon`, and
`sites.org_coordinates_dist=<N>mi`.

```bash
biomcp search trial -c melanoma --source nci --lat 42.36 --lon -71.06 --distance 50 --limit 5
```

For higher limits and reliable authenticated access, set `NCI_API_KEY`.

## Get a trial by NCT ID

```bash
biomcp get trial NCT02576665
```

The default response summarizes title, status, condition context, intervention names, and source metadata. CTGov detail can include source-provided intervention alternate names; for investigational codes, follow-ups may use safer search/article routes instead of a brittle drug-card lookup.

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

## Helper commands

There is no direct `trial <helper>` family. Use inbound pivots such as
`biomcp gene trials <gene>`, `biomcp variant trials <id>`,
`biomcp drug trials <name>`, or `biomcp disease trials <name>` when the anchor
entity is already known.

## Downloaded text and cache

Large text blocks (for example, eligibility text) are cached in the BioMCP download area.
This keeps repeated lookups responsive.

## JSON mode

```bash
biomcp --json get trial NCT02576665
biomcp --json search trial -i daraxonrasib --limit 20
```

## Practical tips

- Start broad on condition, then add intervention and biomarker filters.
- Keep limits low while tuning search criteria.
- Use `eligibility` section only when you need raw criteria text.

## Related guides

- [How to find trials](../how-to/find-trials.md)
- [Disease](disease.md)
- [Drug](drug.md)
