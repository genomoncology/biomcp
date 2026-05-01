---
title: "Semantic Scholar MCP Tool for Citation Graphs | BioMCP"
description: "Use BioMCP to add Semantic Scholar TLDRs, citations, references, and recommendations to literature-review workflows for AI agents."
---

# Semantic Scholar

Semantic Scholar matters when you already have the paper and need the graph around it: the TLDR, the follow-up literature, the references it builds on, and the related papers worth checking next. It turns a flat article lookup into a literature-review workflow that an agent can keep extending without losing the thread.

In BioMCP, `search article` does not expose `--source semantic-scholar`. Instead, Semantic Scholar is an automatic optional search leg when the filter set is compatible, with shared-pool mode at 1 req/2sec without `S2_API_KEY` and authenticated mode at 1 req/sec with the key. The dedicated helper commands on this page are the direct reason to come here: `get article <id> tldr`, `article citations`, `article references`, and `article recommendations`.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search article` | Optional compatible search-leg enrichment plus source status | Semantic Scholar joins article search automatically when the filter set allows it; `--source semantic-scholar` is not a public source switch |
| `get article <id> tldr` | TLDR text, influence counts, and related article metadata | Dedicated Semantic Scholar helper |
| `article citations <id>` | Citation graph rows | Dedicated Semantic Scholar helper |
| `article references <id>` | Reference graph rows | Dedicated Semantic Scholar helper |
| `article recommendations <id>` | Related-paper recommendations | Dedicated Semantic Scholar helper |

## Example commands

```bash
biomcp get article 22663011 tldr
```

Returns a Semantic Scholar section with TLDR text and influence metadata.

```bash
biomcp article citations 22663011 --limit 3
```

Returns a citation graph table with intents, influential flags, and context columns.

```bash
biomcp article references 22663011 --limit 3
```

Returns a reference graph table with the same citation-context fields.

```bash
biomcp article recommendations 22663011 --limit 3
```

Returns a recommendations table with PMID, title, journal, and year columns.

## API access

Optional `S2_API_KEY` for dedicated quota and higher reliability. Configure it with the [API Keys](../getting-started/api-keys.md) guide and request one from the [Semantic Scholar API page](https://www.semanticscholar.org/product/api).

Without `S2_API_KEY`, BioMCP uses the shared unauthenticated pool at
1 req/2sec. A shared-pool HTTP 429 fails fast with guidance to set the key
instead of retrying against the same public pool. With `S2_API_KEY`, BioMCP
sends authenticated requests at 1 req/sec and honors authenticated
`Retry-After` responses before retrying. Source status and debug-plan output
report `auth_mode` as `shared_pool` or `authenticated`, but never print the
secret key or key prefix.

## Runtime behavior

`search article` exposes Semantic Scholar as an automatic compatible leg rather
than a user-selectable source flag. Keep using `--source all`, `pubtator`,
`europepmc`, `pubmed`, or `litsense2` for the public source switch; Semantic
Scholar joins only when the article filters can support it.

JSON search responses can include redacted Semantic Scholar source status under
`_meta.source_status[]`, and `--debug-plan` mirrors that redacted status in the
article leg so operators can distinguish `ok`, `degraded`, and `unavailable`
without exposing credentials. Degradation of the optional Semantic Scholar leg
should not be read as a PubMed, Europe PMC, or PubTator failure.

## Official source

[Semantic Scholar](https://www.semanticscholar.org/) is the official literature-graph product behind BioMCP's TLDR and citation helper workflows.

## Related docs

- [Article](../user-guide/article.md)
- [How to find articles](../how-to/find-articles.md)
- [API Keys](../getting-started/api-keys.md)
