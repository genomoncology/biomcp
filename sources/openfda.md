---
title: "OpenFDA MCP Tool for Drug Safety Workflows | BioMCP"
description: "Use BioMCP to query OpenFDA adverse events, recalls, device reports, labels, and approval context for drug safety and surveillance workflows."
---

# OpenFDA

OpenFDA is where drug-safety work turns from abstract concern into concrete public records. It matters because FAERS, MAUDE, recalls, and label documents are the source material behind many real-world surveillance, safety-review, and regulatory triage workflows.

In BioMCP, OpenFDA covers adverse events, recalls, device-event reporting,
labels, shortages, U.S. approval context, and the optional diagnostic
`regulatory` overlay. Approval views are Drugs@FDA-derived inside BioMCP, not a
separate direct client, so the OpenFDA page is the right mental model for
those U.S. safety and approval workflows.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search adverse-event --drug <name>` | FAERS report search by drug | OpenFDA adverse-event path |
| `search adverse-event --type recall --drug <name>` | Drug recall search results | OpenFDA recall path |
| `search adverse-event --type device --device <name>` | MAUDE device-event search results | OpenFDA device-event path |
| `get adverse-event <report_id>` | Source-aware adverse-event detail card | Resolves the report against the relevant OpenFDA-backed dataset |
| `get drug <name> label` | FDA public label text and sections | OpenFDA label path |
| `get drug <name> shortage` | Current U.S. shortage status and availability context | Default shortage path is OpenFDA-backed |
| `get drug <name> approvals` | U.S. approval and application details | Drugs@FDA-derived approval context surfaced through BioMCP |
| `get drug <name> interactions` | Public interaction text when labels expose it | Uses label-backed interaction content or a truthful fallback |
| `get drug <name> safety --region us` | U.S. safety summary and recall context | OpenFDA-backed U.S. safety workflow |
| `get diagnostic <id> regulatory` | FDA device 510(k)/PMA status overlay | Exact-name-first live lookup over device regulatory records |

## Example commands

```bash
biomcp search adverse-event --drug pembrolizumab --limit 3
```

Returns an adverse-event summary with report totals and a compact result table.

```bash
biomcp search adverse-event --type recall --drug metformin --limit 3
```

Returns a recall results table with recall number, classification, and product fields.

```bash
biomcp search adverse-event --type device --device "insulin pump" --limit 3
```

Returns a device-event table for MAUDE-backed reports.

```bash
biomcp get drug vemurafenib label
```

Returns an FDA label section with public labeling text.

```bash
biomcp get drug dabrafenib approvals
```

Returns U.S. approval and application details from the Drugs@FDA-derived path.

## API access

Optional `OPENFDA_API_KEY` for higher quota headroom. Configure it with the
[API Keys](../getting-started/api-keys.md) guide and request one from the
[OpenFDA authentication page](https://open.fda.gov/apis/authentication/).

## Official source

[OpenFDA](https://open.fda.gov/) is the official FDA developer surface for
adverse events, recalls, labels, device records, and related public regulatory
data.

## Related docs

- [Adverse Event](../user-guide/adverse-event.md)
- [Drug](../user-guide/drug.md)
- [API Keys](../getting-started/api-keys.md)
