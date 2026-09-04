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

Condition searches send the supplied label literally.

```bash
biomcp search trial -c "Rett Syndrome" --limit 20
```

### Pagination termination

For JSON ClinicalTrials.gov trial searches, continue only while
`pagination.has_more` is true and pass the opaque `pagination.next_page_token`
back as `--next-page`. When a
reported total says the returned page reaches the end, BioMCP returns
`has_more: false` and no next-page token, even if the upstream registry supplied
one. This prevents a pagination client from restarting at earlier results.

On the default CTGov path, every `--intervention` worker is sent as one quoted
literal. BioMCP can expand the name with plausible trade names and
investigational codes, excludes systematic chemical synonyms, unions the
matching trials, and shows which alias matched each returned row.

```bash
biomcp search trial -i daraxonrasib --limit 20
biomcp search trial -i daraxonrasib --no-alias-expand --limit 20
```

When an alternate alias wins, markdown adds a `Matched Intervention` column and
JSON adds `matched_intervention_label`. If CTGov rejects only an expanded alias,
BioMCP keeps successful requested-name results and leaves the exact total unknown.
`--no-alias-expand` performs one literal request for the supplied name. If
intervention expansion fans out to multiple CTGov queries, `--next-page` is
unavailable; use `--offset` or `--no-alias-expand`.

JSON search and detail output preserve the complete provider condition array.
Markdown detail lists every condition, while the search table keeps its compact
condition cell; when that cell is abridged, it states the complete condition
count.

Add biomarker filters:

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 5
biomcp search trial -c melanoma --biomarker BRAF --limit 5
```

`--mutation` broadly searches CTGov title, summary, eligibility, and keyword
fields. After broad discovery, simple mutation text receives a registry eligibility
check that removes exclusion-only matches. Trials where the term is absent remain
discoverable, and boolean expressions are discovery-only.

`--age` accepts finite patient ages from 0 through 150 years, including
fractional ages. A registry bound must match the exact numeric grammar
`[0-9]+(?:\.[0-9]+)?`, followed by either no unit or one recognized singular or
plural unit: years, months, weeks, days, hours, or minutes (case-insensitive).
One or more Unicode whitespace characters may surround the numeric/unit tokens
and, when a unit is present, must separate it from the number. Only outer
whitespace is removed from `original`; internal whitespace is retained exactly.
A missing unit means years. Signs, leading or trailing decimal points, exponent
notation, `NaN`, positive infinity spellings (`inf`, `Infinity`, `+inf`,
`+Infinity`), negative infinity spellings (`-inf`, `-Infinity`), punctuation,
trailing tokens, numeric overflow, and unknown units are rejected as malformed.
Filtering compares years, months, weeks, and days; hours, minutes, `N/A`, and
malformed provider text fail open rather than excluding a trial.

Geographic filtering:

```bash
biomcp search trial -c melanoma --lat 42.36 --lon -71.06 --distance 50 --limit 5
```

When geo filters are set, the search query summary includes `lat`, `lon`, and
`distance`. Latitude must be finite from -90 through 90, and longitude must be
finite from -180 through 180.

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

- shared input also accepts `NA`/`N/A` and the early-phase aliases
  `EARLY_PHASE1`, `early_phase1`, and `early1`; matching is case-insensitive
- canonical `PHASE1` through `PHASE4`, numeric `1` through `4`, and Roman `I`
  through `IV` normalize to the same four scalar phases
- `PHASE1/PHASE2`, `1/2`, and `I_II` denote the single combined Phase 1/2
  label and map to CTS `I_II`
- `PHASE2/PHASE3`, `2/3`, and `II_III` denote the single combined Phase 2/3
  label and map to CTS `II_III`
- `NA` and `N/A` become `NA`
- scalar NCI requests stay scalar rather than expanding to overlapping
  combined labels
- `EARLY_PHASE1`, `early_phase1`, and `early1` are accepted shared inputs but
  rejected for `--source nci`; CTGov accepts them

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

NCI accepts one quoted value total across `--biomarker`, `--mutation`, and
`--criteria`, sending it once as the CTS `biomarkers` field. Repeated values or
combining those flags is rejected. NCI also rejects the CTGov-only
`--study-type`, `--sponsor`, `--date-from`, and `--date-to` filters before any
request rather than silently ignoring them.

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

CTGov eligibility is registry-supplied text. JSON eligibility output identifies
that provenance and reports whether posted trial documents are available. When
documents exist, Markdown offers a cautious follow-up because they may contain
additional eligibility detail; BioMCP does not claim that a protocol resolves
any criterion.

Age bounds in JSON are objects, not strings. For example, `6 Months` is
`{"number":6.0,"unit":"months","original":"6 Months"}`. All three members
are always present. A retained no-limit or malformed bound has null `number`
and `unit` while preserving its nonblank `original`; absent or blank bounds are
omitted. Human-readable age ranges retain the provider notation.

Posted CTGov documents use standalone manifest and retrieval forms:

```bash
biomcp --json get trial NCT03361748 documents
biomcp get trial NCT03361748 document Prot_SAP_000.pdf > protocol.pdf
```

Use only an exact filename advertised by the current manifest. Retrieval returns
raw bytes without PDF parsing or conversion and rejects bodies larger than 32
MiB. Document forms are unavailable with `--source nci` and are not included in
ordinary `all`.

Contacts:

```bash
biomcp get trial NCT02576665 contacts
```

Locations:

```bash
biomcp get trial NCT02576665 locations
biomcp get trial NCT02576665 --offset 20 --limit 10 contacts locations
```

Locations use a 20-site page by default. `--offset` and `--limit` select an
explicit page, and Markdown renders that full selected page with a footer that
reports its shown count, total, offset, and limit. When `contacts` and
`locations` are combined, top-level site contacts are scoped to the returned
sites; central contacts remain visible even when the page is empty.

A contacts-only response remains complete. Standalone `all` JSON and batch
JSON are also complete and unpaginated. Unpaginated `all` and batch Markdown
show at most 20 sites, disclose that display cap when it applies, and show only
the top-level site contacts belonging to those visible sites.

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
- Use `eligibility` for registry-supplied criteria text, provenance, and structured sex/age facts.
- Use `contacts` when you need CTGov central or site contact details.

## Related guides

- [How to find trials](../how-to/find-trials.md)
- [Disease](disease.md)
- [Drug](drug.md)
