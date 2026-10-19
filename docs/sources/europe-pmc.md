---
title: "Europe PMC Article Source Guide | BioMCP"
description: "Understand Europe PMC article discovery, full-text availability, and reuse boundaries in BioMCP."
---

# Europe PMC

Europe PMC supplies article search and bibliographic metadata in BioMCP. Its search index can return a paper even when no full-text copy is available.

The full-text XML endpoint serves PMC records only. A PMID-only record does not provide another XML copy through that endpoint; an eligible bioRxiv/medRxiv DOI can use the separate preprint JATS route. Article licenses vary; metadata access does not grant body reuse rights.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search article --source europepmc` | Search results and identifiers | Search availability does not imply full text |
| `get article <id>` | Bibliographic metadata and abstract when available | The article card can exist without a body |
| `get article <id> fulltext` | PMC-backed JATS XML when available | The resolver continues through its other eligible sources |

## Example commands

These are live examples. Provider content and availability can change.

```bash
biomcp search article -g BRAF --source europepmc --limit 3
```

Searches Europe PMC for article metadata.

```bash
biomcp get article 24200969
```

Returns an article card without requesting its body.

```bash
biomcp get article 24200969 fulltext
```

Requests full text through the complete resolver, including Europe PMC where eligible.

## API access

No BioMCP API key required.

## Official source

[Europe PMC](https://europepmc.org/RestfulWebService) documents the upstream service. Follow each article license; see the source terms below.

## Related docs

- [Article](../user-guide/article.md)
- [Find articles](../how-to/find-articles.md)
- [Data Sources](../reference/data-sources.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
