---
title: "LitVar2 MCP Tool for Variant-Linked Literature | BioMCP"
description: "Use BioMCP to merge NCBI LitVar2 variant-literature PMIDs into variant article searches for a gene plus protein change or an rsID."
---

# LitVar2

LitVar2 is NCBI's variant-to-literature index: it resolves a variant (a gene plus protein change, or an rsID) to the papers that name it. In BioMCP, LitVar2 is one of the variant article sources: `variant articles` asks it beside PubTator3's annotation route and merges the linked PMIDs into the union result with `litvar` source attribution, so variant papers the text and annotation routes miss still appear.

The match is exact. BioMCP keeps only a LitVar2 autocomplete entry whose gene and protein change, or rsID, matches the request through the same variant identity comparison the other routes use, so a reply naming BRAF p.V600K for a p.V600E request contributes nothing. A LitVar2 failure degrades that source only; the search keeps its other routes.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `biomcp variant articles "<gene> <protein change>"` | The PMIDs LitVar2 links to the matched variant, merged into the union rows | Runs in the default union and the `--strategy annotation` strategy |
| `biomcp variant articles <rsID>` | The PMIDs LitVar2 links to the rsID, merged into the union rows | Works when the rsID itself carries no other alias |
| `--disable-sources litvar2` | Nothing; the source never answers | The route reports unavailable naming the setting |

## Example commands

```bash
biomcp --json variant articles "CDKN2A A57V" --limit 50
```

Merges the 39 papers LitVar2 links to rs372266620 (CDKN2A p.A57V) beside the text and annotation routes, each merged row carrying the `litvar` source and `litvar_variant` route.

```bash
biomcp --json variant articles rs113488022 --limit 50
```

Resolves the rsID through LitVar2's autocomplete and merges the linked BRAF p.V600E literature with PubTator's annotated papers.

```bash
biomcp --json variant articles rs372266620 --limit 50
```

Returns the LitVar2-linked papers for a bare rsID whose MyVariant resolution is indeterminate, where the annotation route alone cannot run.

## API access

No BioMCP API key required. BioMCP uses NCBI's public LitVar2 API
(`variant/autocomplete` and `variant/get/<variant id>/publications`) without a
key.

## Official source

[LitVar2](https://www.ncbi.nlm.nih.gov/research/litvar2/) is NCBI's public
variant-literature search. BioMCP uses the public
[LitVar2 API](https://www.ncbi.nlm.nih.gov/research/litvar2-api/) for the
surfaces described here.

## Related docs

- [Variant](../user-guide/variant.md)
- [Article](../user-guide/article.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing](../reference/source-licensing.md)
