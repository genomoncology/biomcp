# Intervention

Use intervention commands to search clinical trial interventions from the NCI Clinical Trials Search API.

## Search interventions

By name:

```bash
biomcp search intervention pembrolizumab
```

By type:

```bash
biomcp search intervention --type drug --limit 10
```

By type and category:

```bash
biomcp search intervention --type drug --category "checkpoint inhibitor" --limit 10
```

By NCI thesaurus code:

```bash
biomcp search intervention --code C1447 --limit 5
```

### Search filters

| Flag | Description |
|------|-------------|
| `-q/--query` | Intervention name or keyword |
| `--type` | Intervention type (`drug`, `device`, `procedure`) |
| `--category` | Intervention category |
| `--code` | NCI thesaurus code |
| `-l/--limit` | Max results |
| `--offset` | Pagination offset |

Intervention is search-only. There is no `get` subcommand.

## JSON mode

```bash
biomcp --json search intervention pembrolizumab
```

## Practical tips

- Use `--type drug` to exclude procedures and devices from results.
- Pair intervention searches with trial searches to find active protocols.
- NCI thesaurus codes provide precise lookups when available.

## Related guides

- [Trial](trial.md)
- [Drug](drug.md)
- [Organization](organization.md)
