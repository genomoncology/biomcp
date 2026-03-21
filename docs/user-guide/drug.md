# Drug

Use drug commands for medication lookup, target-oriented search, and safety context.

## Search drugs

Text query:

```bash
biomcp search drug -q "kinase inhibitor" --limit 5
```

Target-oriented search:

```bash
biomcp search drug --target BRAF --limit 5
```

Indication-oriented search:

```bash
biomcp search drug --indication melanoma --limit 5
```

`search drug --interactions <drug>` is currently unavailable because the public data sources BioMCP uses do not expose partner-indexed interaction rows.

## Get a drug record

```bash
biomcp get drug pembrolizumab
```

Default output provides concise identity and mechanism context. Approval-bearing
JSON now includes additive `approval_date_raw`, `approval_date_display`, and
`approval_summary` fields, while markdown renders the human-friendly display
date in the base card.

## Request drug sections

Supported sections: `label`, `shortage`, `targets`, `indications`,
`interactions`, `civic`, `approvals`, `all`.

FDA label section:

```bash
biomcp get drug vemurafenib label
```

Shortage section:

```bash
biomcp get drug carboplatin shortage
```

Targets and indications sections:

```bash
biomcp get drug pembrolizumab targets
biomcp get drug pembrolizumab indications
```

Interactions (OpenFDA label text when public interaction details are available; otherwise a truthful fallback):

```bash
biomcp get drug warfarin interactions
```

CIViC evidence and Drugs@FDA approvals:

```bash
biomcp get drug vemurafenib civic
biomcp get drug dabrafenib approvals
```

## Cross-entity helpers

Trial pivot:

```bash
biomcp drug trials pembrolizumab --limit 5
```

Safety pivot:

```bash
biomcp drug adverse-events pembrolizumab --limit 5
```

## JSON mode

```bash
biomcp --json get drug pembrolizumab
```

## Practical tips

- Start with base `get` before requesting heavy sections.
- Use target filters to narrow crowded drug classes.
- Pair drug lookups with trial filters for protocol matching workflows.

## Related guides

- [Adverse event](adverse-event.md)
- [Trial](trial.md)
- [Data sources](../reference/data-sources.md)
