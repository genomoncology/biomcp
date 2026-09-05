# Drug Approvals and Adverse-Event Summary

This reference covers the Drugs@FDA approvals section plus the adverse-event
search summary contract for OpenFDA FAERS and vaccine-capable CDC WONDER VAERS
searches.

## Drug approvals (Drugs@FDA)

Use the `approvals` section on drug entities:

```bash
biomcp get drug dabrafenib approvals
```

The section includes:

- application number (NDA/BLA),
- sponsor,
- key product rows (brand/dosage form/route/status),
- submission timeline rows (type/number/status/date).

Base drug cards and JSON also expose compact approval fields derived from the
existing normalized approval date:

- `approval_date_raw` keeps the stable ISO form,
- `approval_date_display` provides a human-friendly month-name rendering,
- `approval_summary` provides a one-line `"FDA approved on <date>"` summary.

## Adverse-event summary statistics

OpenFDA FAERS search responses include summary metadata above the report table:

```bash
biomcp search adverse-event -d pembrolizumab --limit 10
```

OpenFDA FAERS summary fields:

- total reports from OpenFDA FAERS metadata,
- returned report count,
- top reactions with count and a denominator-specific report share,
- `percentage_context` in JSON whenever reaction percentage rows are present.

Ordinary search calculates each reaction's **share of returned reports** from
the returned page, which is only a sample of all matching reports. The drug
helper calculates each reaction's **share of matching reports** from OpenFDA's
aggregate count response and the total matching report count. The structured
context names the denominator as `returned_reports` or `all_matching_reports`,
records the exact divisor in `denominator_count`, and marks both
`is_incidence` and `establishes_causality` false. It is omitted when there are
no reaction percentage rows and remains additive to all existing summary keys.

FAERS contains spontaneous reports, not a denominator of treated or exposed
patients. These percentages are not incidence estimates and do not establish
causality.

The same summary appears in:

```bash
biomcp drug adverse-events pembrolizumab --limit 10
```

For vaccine queries, `search adverse-event` also supports CDC WONDER VAERS:

```bash
biomcp search adverse-event "COVID-19 vaccine" --source all --limit 10
biomcp search adverse-event "MMR vaccine" --source vaers --limit 10
```

`--source all` preserves the FAERS table and adds a CDC VAERS aggregate summary
when the query resolves to a vaccine and the active filters are VAERS-compatible.
`--source vaers` is aggregate-only and surfaces matched vaccine identity, CDC
WONDER code, CVX code(s) when available, serious vs non-serious counts, age
distribution, and top reactions.
