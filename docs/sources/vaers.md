---
title: "CDC WONDER VAERS MCP Tool for Vaccine Safety Signals | BioMCP"
description: "Use BioMCP to query CDC WONDER VAERS aggregate vaccine adverse-event summaries in BioMCP with reaction counts, seriousness breakdowns, and age distribution."
---

# CDC WONDER VAERS

CDC WONDER VAERS is the right source when you need vaccine-specific safety
signal context rather than general post-marketing drug surveillance. In BioMCP,
it is an aggregate-only branch of `search adverse-event`: `search adverse-event --source vaers`
runs the direct CDC WONDER D8 query path, while the default `--source all`
route keeps OpenFDA FAERS and adds VAERS only when the query resolves to a
vaccine.

Vaccine identity can flow through the local CDC CVX/MVX bridge when the query
needs help resolving to the CDC WONDER D8 code space, so brand-heavy inputs can
still land on the right vaccine family without exposing the raw CDC codes to
the operator. JSON from `--source all` keeps independent
`section_outcomes.faers` and `.vaers` states. CVX or VAERS failure is
`unavailable`; only a healthy CVX classification can produce
`query_not_vaccine` or `unmapped_vaccine` in `vaers.status`.

## What BioMCP exposes

The VAERS-only path supports only the vaccine query text from `--drug` or the
positional argument. FAERS-specific filters such as reaction, seriousness,
date, demographic, reporter, and count filters stay on the OpenFDA FAERS path
in this ticket.

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search adverse-event <vaccine_query> --source vaers` | Aggregate VAERS vaccine summary | Direct CDC WONDER D8 XML POST query |
| `search adverse-event <vaccine_query> --source all` | Combined FAERS + VAERS vaccine search | Keeps the FAERS table and appends CDC VAERS when the filters are compatible |
| `search adverse-event <vaccine_query>` | Default vaccine search behavior | Equivalent to `--source all` for vaccine-resolved queries |
| `biomcp health` | VAERS readiness row | Probes the real CDC WONDER VAERS query path |

## Example commands

```bash
biomcp search adverse-event "MMR vaccine" --source vaers --limit 5
```

Returns an aggregate CDC VAERS summary with matched vaccine identity, CDC WONDER code, CVX code(s) when available, serious vs non-serious counts, age distribution, and top reactions.

```bash
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 5
```

Keeps the OpenFDA FAERS result table and appends a CDC VAERS summary when the query resolves to a vaccine and the active filters are VAERS-compatible.

```bash
biomcp search adverse-event "influenza vaccine" --source all --limit 5
```

Uses the same combined route for a broad vaccine-family query.

```bash
biomcp search adverse-event "COVID-19 vaccine" --source faers --limit 5
```

Forces the legacy FAERS-only path when you explicitly do not want VAERS.

```bash
biomcp health --apis-only
```

Shows the `CDC WONDER VAERS` readiness row in the live upstream inventory.

## API access

No BioMCP API key required. BioMCP calls the CDC WONDER D8 XML POST endpoint and includes the required agreement to the CDC WONDER data use restrictions.

## Official source

[CDC WONDER VAERS](https://wonder.cdc.gov/vaers.html) is the official CDC
WONDER entry point for public VAERS data and documentation.

## Related docs

- [Adverse Event](../user-guide/adverse-event.md)
- [Data Sources](../reference/data-sources.md)
- [Troubleshooting](../troubleshooting.md)
