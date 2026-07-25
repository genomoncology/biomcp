---
title: "ClinGen CSpec source documents | BioMCP"
description: "Retrieve versioned ClinGen Criteria Specification Registry source documents with preserved captures."
---

# ClinGen Criteria Specification Registry

ClinGen CSpec publishes versioned VCEP criteria specifications. BioMCP preserves
source facts and exact stored provider captures; it does not evaluate ACMG criteria
or produce classifications.

Use the full resource IRI returned by a manifest when selecting a document; its display version is a separate provider fact.

## What BioMCP exposes

- `gene cspec <gene>` lists full resource IRIs returned by the provider.
- `gene cspec <gene> --version <full-IRI>` retrieves one exact source document and binds its normalized gene, resource IRI, and specification ID to the capture.
- `gene cspec <gene> --capture-id <capture-id>` pages that bound capture without another provider request.
- Typed MCP `gene_cspec` returns the manifest or bounded parsed capture pages. Its caller gene must match the captured binding.
- Parsed pages include raw-byte provenance and a page-independent `cspec-semantic-v1` digest; citations are deduplicated and capped at 32 per criterion.

## Example commands

```bash
biomcp --json gene cspec ATM
```

List every available version IRI for ATM.

```bash
biomcp --json gene cspec ATM --version https://cspec.genome.network/cspec/SequenceVariantInterpretation/id/GN020/version/1.5.1
```

Select one exact version and retain its capture provenance.

```bash
biomcp gene cspec document <capture-id>
```

Stream the original local capture bytes without another provider request.

## API access

No BioMCP API key required.

## Official source

- <https://cspec.clinicalgenome.org>

## Related docs

- [Gene user guide](../user-guide/gene.md)
- [Data sources](../reference/data-sources.md)
- [Source licensing](../reference/source-licensing.md)
