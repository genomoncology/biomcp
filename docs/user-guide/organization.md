# Organization

Use organization commands to search research institutions and trial sites from the NCI Clinical Trials Search API.

## Search organizations

By name:

```bash
biomcp search organization "Dana-Farber"
```

By state:

```bash
biomcp search organization --state Massachusetts --limit 10
```

By type and city:

```bash
biomcp search organization --type academic --city Boston --limit 10
```

Industry organizations:

```bash
biomcp search organization --type industry --limit 10
```

### Search filters

| Flag | Description |
|------|-------------|
| `-q/--query` | Organization name or keyword |
| `--type` | Organization type (`academic`, `industry`) |
| `--city` | City name |
| `--state` | State or province |
| `-l/--limit` | Max results |
| `--offset` | Pagination offset |

Organization is search-only. There is no `get` subcommand.

## JSON mode

```bash
biomcp --json search organization "Dana-Farber"
```

## Practical tips

- Use `--state` to narrow results to a geographic area.
- Combine `--type` with a query to distinguish academic medical centers from sponsors.
- Pair organization results with trial searches to find site-specific enrollment.

## Related guides

- [Trial](trial.md)
- [Intervention](intervention.md)
- [Biomarker](biomarker.md)
