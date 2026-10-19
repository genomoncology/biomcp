---
title: "PMC Article Source Guide | BioMCP"
description: "Understand PMC article discovery, full-text availability, and reuse boundaries in BioMCP."
---

# PMC

PMC is the article archive behind BioMCP's PMC XML and HTML retrieval routes. A PMC copy can be a published paper or an author manuscript; a PMCID alone does not guarantee that every retrieval route will succeed.

BioMCP tries Europe PMC XML, NCBI EFetch XML, and PMC OA XML before PMC HTML. Readable PMC copies outside the OA subset can still resolve through another route. PMC OA and other PMC copies retain article-level licenses; reading access does not grant unrestricted reuse. Eligible preprints can use a separate advertised JATS route without PMC.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `get article <id> fulltext` | A saved article body from an eligible XML or HTML route | An earlier route miss does not end the ladder |
| `get article <id> assets` | An article asset manifest | Assets and full text have separate availability |
| `get article <id> fulltext --pdf` | An optional PDF fallback after XML and HTML | Semantic Scholar supplies the PDF URL; PMC is not required |

## Example commands

These are live examples. Provider content and availability can change.

```bash
biomcp get article 24200969 fulltext
```

Requests the body and reports the winning source when successful.

```bash
biomcp --json get article 24200969 fulltext
```

Includes full-text coverage, source attempts, and the reuse manifest.

```bash
biomcp --json get article 24200969 assets
```

Requests the separate article asset manifest.

## API access

Optional `NCBI_API_KEY` for NCBI requests.

## Official source

[PMC](https://pmc.ncbi.nlm.nih.gov/) documents the upstream service. Follow each article license; see the source terms below.

## Related docs

- [Article](../user-guide/article.md)
- [Find articles](../how-to/find-articles.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
