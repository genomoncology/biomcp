---
title: "Cellosaurus MCP Tool for Cell Line Lookup | BioMCP"
description: "Use BioMCP to turn any common spelling of a cell line into its Cellosaurus accession, then read species, disease, cross-references, and curated variants."
---

# Cellosaurus

Cellosaurus matters when a cell line arrives under four spellings and you need one stable identifier. MOLM13, MOLM-13, MV4-11 and MV4;11 all resolve to a CVCL accession, and that accession is the key every other cell line source joins on.

In BioMCP, Cellosaurus is the identity source for cell lines. Search returns ranked candidates with the match kind that separates a real line from a look-alike, and the detail card carries species, disease, category, sex, and age. Cross-references and curated sequence variations are explicit sections, so the base card stays small. BioMCP paces Cellosaurus at 3 requests / second because the provider publishes no rate limit.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search cell-line <name>` | Ranked candidate lines with accession, species, category, disease, and match kind | Identifier matches sort before synonym-only matches, and human lines first |
| `get cell-line <accession>` | Identity card: name, synonyms, species, disease, category, sex, age, RRID | The base card never fetches cross-references |
| `get cell-line <accession> xrefs` | DepMap, COSMIC, ChEMBL, Cell Model Passports, GDSC, PharmacoDB, and LINCS join keys | Every key is printed; a missing link is an empty list |
| `get cell-line <accession> variants` | Curated sequence variations with HGVS description, HGNC link, zygosity, and PubMed sources | Published as Cellosaurus wrote them; BioMCP adds no interpretation |
| `get cell-line <source-id>` | Reverse lookup from a DepMap, Cell Model Passports, ChEMBL, or PharmacoDB ID | One search request resolves the ID to its accession |

## Example commands

```bash
biomcp search cell-line MOLM13
```

Returns CVCL_2119 as an exact match. A hyphenated query such as `MOLM-13` searches both spellings and merges the windows by accession.

```bash
biomcp search cell-line "MV4;11"
```

Returns CVCL_0064, whose identifier is `MV4-11`. The punctuation does not change the answer.

```bash
biomcp get cell-line CVCL_2119
```

Returns the MOLM-13 identity card with the RRID and the next commands for the two sections.

```bash
biomcp get cell-line CVCL_2119 xrefs
```

Returns the join keys, including DepMap `ACH-000362`.

```bash
biomcp get cell-line CVCL_1844 variants
```

Returns the curated DNMT3A, NPM1, and NRAS rows for OCI-AML-3.

## Release and attribution

Every cell line output names the Cellosaurus release it came from, read at runtime from the provider's own release endpoint. When that read fails, the output names the retrieval time instead. Cellosaurus data is CC BY 4.0, and every output carries the attribution line and the requested citation.

## API access

No BioMCP API key required. Set `BIOMCP_CELLOSAURUS_BASE` to point at a different host.

## Official source

[Cellosaurus](https://www.cellosaurus.org/) is the cell line knowledge resource behind BioMCP's cell line search rows and detail cards. It is also the RRID authority for cell lines.

## Related docs

- [Cell Line](../user-guide/cell-line.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing](../reference/source-licensing.md)
