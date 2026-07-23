---
title: "ClinGen ERepo MCP Tool for Expert Assertions | BioMCP"
description: "Retrieve versioned ClinGen Evidence Repository expert assertions by CAid through BioMCP without inferring ACMG/AMP evidence strength."
---

# ClinGen Evidence Repository

ClinGen ERepo provides versioned Variant Curation Expert Panel assertions keyed by
ClinGen Allele identifier (CAid).

BioMCP preserves source facts rather than applying ACMG/AMP rules or treating
defaults and comments as applied criterion strength.

## What BioMCP exposes

- `variant erepo <CAid>` returns zero or more assertion summaries with independent
  met and unmet source criterion lists.
- `variant erepo <CAid> --detail` retrieves one explicitly selected versioned SEPIO
  document, including comments, curator facts, and narrowly located PMIDs.
- Typed MCP `variant_erepo` accepts one `caid` or a bounded `caids` batch.

## Example commands

```bash
biomcp --json variant erepo CA015543
```

Retrieve source-faithful assertion summaries for one CAid.

```bash
biomcp --json variant erepo CA015543 --detail
```

Retrieve one selected versioned SEPIO detail document.

```bash
biomcp --json variant erepo --input caids.json
```

Batch input accepts 1–50 CAids and remains summary-only.

When a summary has multiple assertions, detail requires `--assertion <UUID>`;
`--version` must be an exact source document version.

## API access

No BioMCP API key required.

## Official source

- <https://erepo.clinicalgenome.org>

## Related docs

- [Variant user guide](../user-guide/variant.md)
- [Data sources](../reference/data-sources.md)
- [Source licensing](../reference/source-licensing.md)
