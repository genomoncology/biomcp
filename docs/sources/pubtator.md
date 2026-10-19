---
title: "PubTator3 Article Source Guide | BioMCP"
description: "Understand PubTator3 article discovery, full-text availability, and reuse boundaries in BioMCP."
---

# PubTator3

PubTator3 supplies annotated article search and entity annotations in BioMCP. It helps connect a clinical literature question to papers and the genes, diseases, drugs, and variants named in them.

PubTator3 full-text exports add PMC passages when a PMC copy is available; otherwise the export can remain abstract-only. Those annotations do not establish that the full-text resolver can save a body. LitVar2 contributes variant-linked paper discovery, not a separate body source. Preserve article provenance and article-level rights when reusing annotations or text.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search article --source pubtator` | Annotated literature discovery | Search hits can have abstract-only content |
| `get article <id> annotations` | Article entity annotations | This section does not request the saved full-text body |
| `article entities <pmid>` | Grouped entity annotations | Variant identity verification can consult PMC full-text passages |

## Example commands

These are live examples. Provider content and availability can change.

```bash
biomcp search article -g BRAF --source pubtator --limit 3
```

Searches the PubTator3 article index.

```bash
biomcp get article 24200969 annotations
```

Requests article annotations independently of full-text retrieval.

```bash
biomcp article entities 24200969
```

Groups available article annotations by entity.

## API access

Optional `NCBI_API_KEY` for NCBI requests.

## Official source

[PubTator3](https://www.ncbi.nlm.nih.gov/research/pubtator3/api) documents the upstream service. Follow each article license; see the source terms below.

## Related docs

- [Article](../user-guide/article.md)
- [Find articles](../how-to/find-articles.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
