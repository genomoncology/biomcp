---
title: "ClinicalTrials.gov MCP Tool for AI Agents | BioMCP"
description: "Search ClinicalTrials.gov from BioMCP for recruiting studies, eligibility criteria, locations, and trial details without learning the native API."
---

# ClinicalTrials.gov

ClinicalTrials.gov is the default public registry for structured trial discovery, which makes it the most practical source when you need to move from a condition or biomarker to live recruiting studies quickly. It is the source most clinicians, coordinators, and patients already recognize, so its identifiers and status labels carry well across teams.

This page covers BioMCP's default trial backend. BioMCP also supports `--source nci` for NCI CTS, but that is a separate boundary with different access rules and should not be conflated with the default ClinicalTrials.gov path.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search trial` | Filtered trial search with condition, status, biomarker, and pagination controls | Default backend is ClinicalTrials.gov v2 |
| `get trial <nct_id>` | Trial summary card for a specific NCT record | Uses the default ClinicalTrials.gov detail path |
| `get trial <nct_id> eligibility` | Inclusion and exclusion criteria text | Section expansion from the same trial record |
| `get trial <nct_id> locations` | Facility and contact rows | Uses site data from the default backend |
| `get trial <nct_id> outcomes` | Primary and secondary outcome measures | Detail-section view |
| `get trial <nct_id> arms` | Study arms and interventions | Detail-section view |
| `get trial <nct_id> references` | Linked publications and citations when present | Detail-section view |

## Example commands

```bash
biomcp search trial -c melanoma --status recruiting --limit 3
```

Returns a trial table with NCT ID, title, status, phase, and condition columns.

```bash
biomcp search trial -c melanoma --mutation "BRAF V600E" --limit 3
```

Returns a filtered trial table with the mutation echoed in the query summary.

```bash
biomcp get trial NCT02576665
```

Returns a trial card with the NCT heading, status, and condition context.

```bash
biomcp get trial NCT02576665 eligibility
```

Returns an eligibility section with inclusion and exclusion criteria text.

```bash
biomcp get trial NCT02576665 --limit 3 locations
```

Returns a locations table with facility, city, country, status, and contact fields.

## API access

No BioMCP API key required.

## Official source

[ClinicalTrials.gov](https://clinicaltrials.gov/) is the official public registry and API surface behind BioMCP's default trial workflow.

## Related docs

- [Trial](../user-guide/trial.md)
- [How to find trials](../how-to/find-trials.md)
- [Data Sources](../reference/data-sources.md)
