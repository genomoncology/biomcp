---
title: "PubMed MCP Tool for AI Agents | BioMCP"
description: "Search PubMed in BioMCP with PubTator3 annotations, article summaries, and PMC full-text handoff so AI agents can review literature faster."
---

# PubMed

"PubMed" is an umbrella label for BioMCP's PMID-centric literature workflow, so it is the starting point for most biomedical literature work: researchers get a shared identifier system, durable abstracts, and the fastest path from a gene, disease, or drug question to the papers that matter. If you want an MCP-friendly literature workflow that still speaks the language of PMIDs, this is the page to start with.

In BioMCP, PubMed is both a direct article-search source and part of the
default compatible article federation. `search article --source pubmed` uses
BioMCP's PubMed ESearch/ESummary loop directly, while the default `--source
all` route combines PubTator3, Europe PMC, and PubMed when the selected
filters are PubMed-compatible. Direct PubMed search and the compatible
federated PubMed leg clean question-format unfielded article terms before
ESearch; BioMCP keeps the raw gene, disease, drug, or keyword wording in
markdown and JSON query echoes, and other article sources keep their existing
query behavior. The opt-in `indexing` section uses PubMed citation EFetch XML for associated author affiliations, ORCID, and structured MeSH headings; `all` includes it while ordinary detail/search/batch do not. Full-text resolution uses Europe PMC, NCBI E-utilities, PMC OA, NCBI ID Converter, PMC HTML, and opt-in Semantic Scholar PDF metadata; full text and PDFs remain governed by article-level licenses. Article JSON records the full-text ladder as `not_requested`, `data`, confirmed `empty`, or `unavailable`; a later successful source wins, but a healthy miss cannot erase an earlier source failure. Markdown and `_meta.section_sources` project the same outcome.
Semantic Scholar TLDR, citation, reference, and recommendation helpers belong
on the [Semantic Scholar](semantic-scholar.md) page because they come from a
different provider surface.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search article` | PMID-ranked literature search results with typed filters | Direct `--source pubmed` route plus default compatible federation with PubTator3 and Europe PMC |
| `get article <id>` | Article summary card with identifiers, journal, and abstract context | Uses Europe PMC metadata with BioMCP normalization |
| `get article <id> annotations` | PubTator entity annotations for a paper | PubTator3-only section |
| `get article <id> indexing` | Associated citation authors/affiliations/ORCID and structured MeSH headings | Opt-in PubMed citation XML; explicit status separates available-empty from unavailable; included by `all` |
| `get article <id> fulltext` | Open-access full-text handoff with saved Markdown path and rendered references when available | Uses Europe PMC, NCBI E-utilities, PMC OA, PMC HTML, and opt-in Semantic Scholar PDF fallbacks; NCBI ID Converter bridges PMID/DOI identifiers to PMCID before the PMCID-dependent source attempts |
| `article entities <pmid>` | Entity-grouped follow-up view for a PMID | Derived from PubTator3 annotation output |

## Example commands

```bash
biomcp search article -g BRAF --limit 3
```

Returns an article table with PMID and title columns for a fast literature scan.

```bash
biomcp get article 22663011
```

Returns an article card with PMID, journal, and summary metadata.

```bash
biomcp get article 22663011 annotations
```

Returns a PubTator annotation section with entity groups and counts.

Use `biomcp get article 22663011 indexing` for PubMed citation indexing metadata
that preserves author-affiliation associations and MeSH descriptor/qualifier
flags.

```bash
biomcp article entities 22663011
```

Returns an entity-grouped follow-up view with separate genes, diseases, and drugs sections.

```bash
biomcp get article 27083046 fulltext
```

Returns a full-text section with a `Saved to:` cache path.
XML, PMC HTML, or explicitly opted-in PDF sources can resolve. JATS Markdown can
render references, figure captions, supplementary-material metadata, and complex-table
omission markers. Semantic Scholar PDF is attempted only when the caller passes `--pdf`.
Provider-returned PMC OA archive links are accepted only from reviewed NCBI HTTPS
origins; scheme, origin, port, DNS answers, and every redirect are checked before
contact without exposing a rejected URL in the public error. PMC OA packages are
capped at 64 MiB compressed, 256 physical tar entries, 8 MiB per regular member,
64 MiB aggregate payload, and 1 MiB of path extension metadata. Archive resource
or metadata-policy failures are sanitized as source unavailable.

## API access

Optional `NCBI_API_KEY` for higher NCBI throughput. Set it through the [API Keys](../getting-started/api-keys.md) guide and create one in [My NCBI](https://www.ncbi.nlm.nih.gov/account/settings/).

## Official source

[PubMed](https://pubmed.ncbi.nlm.nih.gov/) is the official NLM literature search surface most researchers already anchor on.

## Related docs

- [Article](../user-guide/article.md)
- [How to find articles](../how-to/find-articles.md)
- [API Keys](../getting-started/api-keys.md)
- [Source Licensing and Terms](../reference/source-licensing.md)
