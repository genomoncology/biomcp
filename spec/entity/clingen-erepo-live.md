# Live ClinGen ERepo diagnostic

This diagnostic calls the real ClinGen Evidence Repository through the shipped
ERepo client. It complements the frozen routine contract: upstream availability
and records can change, but source state and provenance must remain explicit.

## Real ERepo response keeps its source state

A real CAid request reports an ERepo provider and a complete source state. The
routine fixture owns the detailed evidence semantics; this live check also runs
`--detail` so it detects ERepo identity or response-shape drift without using a
simulated provider.

```bash
biomcp --json variant erepo CA015543 | mustmatch like '"provider"
"source_status"'
biomcp --json variant erepo CA015543 --detail | mustmatch like '"detail"
"source_url"'
```
