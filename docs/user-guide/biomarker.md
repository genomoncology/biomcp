# Biomarker

Use biomarker commands to search clinical trial biomarker criteria from the NCI Clinical Trials Search API.

## Search biomarkers

By name:

```bash
biomcp search biomarker BRAF
```

By type:

```bash
biomcp search biomarker --type "Antigen" --limit 5
```

With eligibility filter:

```bash
biomcp search biomarker --eligibility inclusion --limit 10
```

By assay purpose:

```bash
biomcp search biomarker --assay-purpose "Eligibility Criterion - Inclusion"
```

By NCI thesaurus code:

```bash
biomcp search biomarker --code C17965 --limit 5
```

### Search filters

| Flag | Description |
|------|-------------|
| `-q/--query` | Biomarker name or keyword |
| `--type` | Biomarker type (e.g., `Antigen`, `Gene`) |
| `--eligibility` | Eligibility context |
| `--assay-purpose` | Assay purpose filter |
| `--code` | NCI thesaurus code |
| `-l/--limit` | Max results |
| `--offset` | Pagination offset |

Biomarker is search-only. There is no `get` subcommand.

## JSON mode

```bash
biomcp --json search biomarker BRAF
```

## Practical tips

- Use biomarker search to discover how a marker is used across trial eligibility criteria.
- Combine with trial and intervention searches for full protocol context.
- NCI thesaurus codes provide exact biomarker lookups.

## Related guides

- [Trial](trial.md)
- [Gene](gene.md)
- [Intervention](intervention.md)
