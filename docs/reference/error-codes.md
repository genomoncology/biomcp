# Error Codes

BioMCP exposes structured internal error variants through human-readable CLI messages.
This reference maps each `BioMcpError` variant to likely causes and practical recovery steps.

## Error catalog

| Error variant | Meaning | Recovery guidance |
|---------------|---------|-------------------|
| `HttpClientInit` | HTTP client could not initialize | Check TLS/network stack, proxy settings, and local certificate configuration |
| `Http` | HTTP request failed before receiving a successful response | Retry the command and verify network connectivity |
| `HttpMiddleware` | Retry/cache middleware failed | Retry; if persistent, clear cache and re-run with `--no-cache` |
| `Api` | Upstream API returned an error response | Check API status, input values, and any source-specific constraints |
| `ApiJson` | API response shape changed or returned malformed JSON | Retry once; if repeatable, report issue because upstream format may have changed |
| `NotFound` | Requested entity ID was not found | Verify identifier format; run `search` before `get` when unsure |
| `InvalidArgument` | Command arguments are invalid or inconsistent | Re-run with `--help` and correct flag values/section names |
| `ApiKeyRequired` | Source requires an API key that is not set | Export the listed environment variable and retry |
| `SourceUnavailable` | Requested source could not be used | Switch sources if possible or retry later |
| `Template` | Markdown/templating render failed | Report issue (rendering bug) |
| `Json` | Local JSON serialization/deserialization failed | Retry; if persistent, report issue with command and payload context |
| `Io` | File system I/O failed | Check permissions, available disk space, and install/cache paths |

## Key environment variables

| Variable | Used by |
|----------|---------|
| `ALPHAGENOME_API_KEY` | Variant `predict` section |
| `NCI_API_KEY` | Trial source `--source nci` |
| `ONCOKB_TOKEN` | Production OncoKB enrichment |
| `OPENFDA_API_KEY` | Optional OpenFDA quota stability |

## Not-found troubleshooting pattern

When you get a `NotFound` error, validate in this order:

1. Identifier syntax (`rs...`, `NCT...`, `PMID`, `MONDO:...`)
2. Search by keyword or symbol
3. Retry with a broader query

Examples:

```bash
biomcp search gene -q BRAF --limit 5
biomcp search trial -c melanoma --limit 5
biomcp search disease -q melanoma --limit 5
```

## Related docs

- `docs/troubleshooting.md`
- `docs/reference/data-sources.md`
