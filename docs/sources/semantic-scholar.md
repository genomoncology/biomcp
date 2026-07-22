---
title: "Semantic Scholar MCP Tool for Citation Graphs | BioMCP"
description: "Use BioMCP to add Semantic Scholar TLDRs, citations, references, and recommendations to literature-review workflows for AI agents."
---

# Semantic Scholar

Semantic Scholar supplies provider-defined TLDR, citation, reference, and recommendation records around a paper. BioMCP relays those records; recommendation quality and recall are provider-dependent and not validated by BioMCP.

In BioMCP, Semantic Scholar provides provider-exact author search/detail and an automatic optional `search article --source all` leg when the filter set is compatible; article search is also individually selectable with `--source semanticscholar`. `variant articles --strategy union` uses the bulk phrase endpoint only for its bounded strict lane and retains relevance search as discovery; `--debug-plan` identifies each versioned request. These routes use shared-pool mode at 1 req/2sec without `S2_API_KEY` and authenticated mode at 1 req/sec with the key. The dedicated article helper commands on this page are `get article <id> tldr`, `article citations`, `article references`, and `article recommendations`.

## What BioMCP exposes

| Command | What BioMCP gets from this source | Integration note |
|---|---|---|
| `search author -q <name> --source semanticscholar` | Exact Semantic Scholar provider-record candidates | Results remain separate and provider-qualified; BioMCP does not claim a globally resolved person |
| `get author semanticscholar:<id>` | One exact Semantic Scholar provider record | Requires the qualified numeric ID and does not establish an ORCID link |
| `search article` | Optional compatible search-leg enrichment plus source status | Semantic Scholar joins article search automatically when the filter set allows it and can be selected alone with `--source semanticscholar` |
| `get article <id> tldr` | TLDR text, influence counts, and related article metadata | Dedicated Semantic Scholar helper |
| `article citations <id>` | Citation graph rows | Dedicated Semantic Scholar helper |
| `article references <id>` | Reference graph rows | Dedicated Semantic Scholar helper |
| `article recommendations <id>` | Provider-defined paper recommendations | BioMCP relay; relatedness and recall are not validated by BioMCP |

## Example commands

```bash
biomcp search author -q "Louis Williams" --source semanticscholar --limit 5
biomcp get author semanticscholar:1716151
biomcp search article -k "BRAF melanoma" --source semanticscholar --limit 5
```

Returns provider-qualified author records or article rows from Semantic Scholar.

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

Returns the provider-defined recommendations in a table with typed identifier, title, journal, and year columns.

## API access

Optional `S2_API_KEY` for dedicated quota and higher reliability. Configure it with the [API Keys](../getting-started/api-keys.md) guide and request one from the [Semantic Scholar API page](https://www.semanticscholar.org/product/api).

Without `S2_API_KEY`, BioMCP uses the shared unauthenticated pool at
1 req/2sec. A shared-pool HTTP 429 fails fast with guidance to set the key
instead of retrying against the same public pool. With `S2_API_KEY`, BioMCP
sends authenticated requests at 1 req/sec and honors authenticated numeric
`Retry-After` responses before retrying, bounded by BioMCP's shared 5-second
per-attempt cap and 15-second total retry-sleep budget. Source status and
debug-plan output report `auth_mode` as `shared_pool` or `authenticated`, but
never print the secret key or key prefix.

## Runtime behavior

`search article` exposes Semantic Scholar both as an automatic compatible leg
inside `--source all` and as a standalone source with `--source semanticscholar`.
The standalone route uses the same client, auth mode, rate limits, and graceful
degradation behavior as the compatible federated route.

JSON search responses can include redacted Semantic Scholar source status under
`_meta.source_status[]`, and `--debug-plan` mirrors that redacted status in the
article leg so operators can distinguish `ok`, `degraded`, and `unavailable`
without exposing credentials. Degradation of the optional Semantic Scholar leg
should not be read as a PubMed, Europe PMC, or PubTator failure.

Provider-returned PDF URLs and supported Figshare asset handoffs are fetched
only through BioMCP's shared outbound policy. PDF, Figshare ndownloader, and
reviewed CDN origins are explicit HTTPS allowlists; DNS answers and every
redirect are revalidated before contact, and rejected URLs are not echoed.

## Official source

[Semantic Scholar](https://www.semanticscholar.org/) is the official literature-graph product behind BioMCP's TLDR and citation helper workflows.

## Related docs

- [Article](../user-guide/article.md)
- [How to find articles](../how-to/find-articles.md)
- [API Keys](../getting-started/api-keys.md)
