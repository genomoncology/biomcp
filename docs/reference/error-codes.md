# Error Codes

BioMCP exposes structured internal error variants through human-readable CLI messages.
This reference maps each `BioMcpError` variant to likely causes and practical recovery steps.
JSON output renders variant names as stable snake-case `error.code` values; for example,
missing credentials use `api_key_required`, while configured credentials rejected by a
provider use `api_key_rejected`.

Hard remote-source failures can also include additive `error.source` and
`error.recovery` fields. `source` is a canonical allowlisted provider label
(maximum 80 bytes), and `recovery` is a bounded action (maximum 160 bytes).
Both fields are omitted for unwrapped transport errors when BioMCP no longer
knows which source failed. Legacy source-shaped errors with an unknown name use
the safe fallback described below. The human diagnostic uses the same source
and recovery policy; neither output
includes request destinations, credentials, provider bodies, parser details,
or local paths.

## Process exit codes

BioMCP uses process exit codes to distinguish invalid usage from command
execution failures:

- exit `2`: `clap` rejected the command before BioMCP command execution started.
  With `--json`/`-j`, these usage errors emit the standard JSON error envelope
  on stdout with `error.code: "invalid_argument"`. Example: `biomcp search pathway --badflag`
- exit `2`: the command parsed, then BioMCP returned
  `BioMcpError::InvalidArgument` for invalid or inconsistent usage.
  Examples: `biomcp search pathway`, `biomcp get pathway hsa05200 events`
- exit `1`: runtime, upstream, configuration, not-found, and other execution
  failures unless an explicit command outcome says otherwise.
- exit `1`: alias fallback guidance for `get gene` / `get drug` still counts as a
  not-found miss even when BioMCP can suggest a canonical retry command.
  Example: `biomcp get gene ERBB1`

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
| `ApiKeyRejected` | Provider rejected the configured API key or the account lacks access | Check the credential is valid and that the account has provider access |
| `SourceUnavailable` | Requested source could not be used | Review source configuration and retry |
| `Template` | Markdown/templating render failed | Report issue (rendering bug) |
| `Json` | Local JSON serialization/deserialization failed | Retry; if persistent, report issue with command and payload context |
| `Io` | File system I/O failed | Check permissions, available disk space, and install/cache paths |

## Structured source recovery

The three stable recovery meanings are:

- **Retry the remote source** — a transport, status, or decode failure may be transient.
- **Review source configuration and retry** — check required credentials or source setup first.
- **Narrow the request and retry** — reduce the requested result/body size.

Legacy source errors with an unknown or unsafe provider name use the conservative
label `BioMCP source` and configuration guidance instead of copying that name.
The existing `error.code`, `_meta.not_found`, envelope, output stream, and exit
status remain unchanged when source context is present.

## Key environment variables

| Variable | Used by |
|----------|---------|
| `ALPHAGENOME_API_KEY` | Variant `predict` section |
| `DISGENET_API_KEY` | Scored DisGeNET sections on `get gene` and `get disease` |
| `NCBI_API_KEY` | Higher-throughput PubTator, PubMed/efetch, PMC OA, and NCBI ID converter requests |
| `S2_API_KEY` | Optional authenticated Semantic Scholar requests for article search/get/helpers |
| `NCI_API_KEY` | Trial source `--source nci` |
| `ONCOKB_TOKEN` | Production OncoKB enrichment |
| `OPENFDA_API_KEY` | Optional OpenFDA quota stability |
| `UMLS_API_KEY` | Optional `discover` clinical crosswalk enrichment |

## Not-found troubleshooting pattern

When you get a `NotFound` error, validate in this order:

1. Identifier syntax (`rs...`, `NCT...`, `PMID`, `MONDO:...`)
2. Search by keyword or symbol
3. Retry with a broader query
4. If BioMCP prints `Did you mean: ...`, re-run the suggested canonical `get`
   command. In JSON mode, the same guidance is printed to stdout under
   `_meta.alias_resolution` and `_meta.next_commands` while the process still
   exits `1`.

Examples:

```bash
biomcp search gene -q BRAF --limit 5
biomcp search trial -c melanoma --limit 5
biomcp search disease -q melanoma --limit 5
```

## Related docs

- [Troubleshooting](../troubleshooting.md)
- [Data Sources](data-sources.md)
