# Drug Approvals and Adverse-Event Summary

This reference covers the new Drugs@FDA approvals section and FAERS summary statistics.

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

## FAERS summary statistics

FAERS search responses include summary metadata above the report table:

```bash
biomcp search adverse-event -d pembrolizumab --limit 10
```

Summary fields:

- total reports from OpenFDA metadata,
- returned report count,
- top reactions with count and percentage.

The same summary appears in:

```bash
biomcp drug adverse-events pembrolizumab --limit 10
```
