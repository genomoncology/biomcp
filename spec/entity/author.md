# Author

BioMCP searches Semantic Scholar author records without pretending that a provider record is a globally resolved person. Every usable identity stays provider-qualified, and same-name or split records remain separate candidates.

## Provider-exact search keeps same-name records separate

Search by a researcher's name when separate Semantic Scholar candidates are useful evidence. Each result is exact only within Semantic Scholar; matching names do not cause BioMCP to merge records.

<!-- mustmatch-lint: skip -->

```bash run id=author-search exit=0
../../tools/biomcp-ci --json search author -q "Louis Williams" --source semanticscholar --limit 5
```

```json expect=author-search contains
{
  "query": {"name": "Louis Williams"},
  "providers": [{
    "source": "semantic_scholar",
    "results": [
      {"identity": {"kind": "exact_provider", "id": "semanticscholar:2269573451"}, "display_name": "Louis S. Williams", "warnings": [{"code": "orcid_link_not_established"}]},
      {"identity": {"kind": "exact_provider", "id": "semanticscholar:1994488914"}, "display_name": "Louis S. Williams", "warnings": [{"code": "orcid_link_not_established"}]}
    ],
    "status": "available"
  }]
}
```

The response metadata gives agents provider health, evidence, and an executable
next step without inventing a BioMCP author identifier.

```json expect=author-search contains
{
  "_meta": {
    "source_status": [{"source": "semantic_scholar", "status": "available"}],
    "evidence_urls": [
      {"url": "https://www.semanticscholar.org/author/2269573451"},
      {"url": "https://www.semanticscholar.org/author/1994488914"}
    ],
    "next_commands": [
      "biomcp get author semanticscholar:2269573451",
      "biomcp get author semanticscholar:1994488914"
    ]
  }
}
```

Provider responses may grow fields, but public author results remain limited to professional identity evidence. In particular, successful JSON does not expose private-profile or inferred-demographic keys.

```text expect=author-search not-contains
"email":
"homepage":
"private_profile":
"gender":
"race":
"ethnicity":
"externalIds":
"external_ids":
private-author@example.invalid
https://private.example.invalid/author
fixture-private-profile
fixture-inferred-demographic
0000-0002-7433-2740
```

## Exact provider detail preserves identity and uncertainty

Use the qualified ID from search to retrieve exactly that Semantic Scholar record. Detail keeps provider provenance visible and does not turn an unverified external handle into a cross-provider identity link.

<!-- mustmatch-lint: skip -->

```bash run id=author-detail exit=0
../../tools/biomcp-ci --json get author semanticscholar:1716151
```

```json expect=author-detail contains
{
  "identity": {"kind": "exact_provider", "id": "semanticscholar:1716151"},
  "display_name": "A. Butte",
  "provider_records": [{"id": "semanticscholar:1716151", "status": "available"}],
  "conflicts": [],
  "warnings": [{"code": "orcid_link_not_established"}],
  "_meta": {
    "source_status": [{"source": "semantic_scholar", "status": "available"}],
    "evidence_urls": [{"url": "https://www.semanticscholar.org/author/1716151"}],
    "next_commands": ["biomcp author papers semanticscholar:1716151"]
  }
}
```

The allowlisted detail projection applies the same privacy boundary as search.

```text expect=author-detail not-contains
"email":
"homepage":
"private_profile":
"gender":
"race":
"ethnicity":
"externalIds":
"external_ids":
private-author@example.invalid
https://private.example.invalid/author
fixture-private-profile
fixture-inferred-demographic
0000-0002-7433-2740
```

## Provider-exact pivots connect authors and papers

An author record leads to compact papers with provider-owned pagination. Paper
identifiers remain usable by the article family, and metadata supplies evidence
and executable next steps.

<!-- mustmatch-lint: skip -->

```bash run id=author-papers exit=0
../../tools/biomcp-ci --json author papers semanticscholar:1716151 --limit 1
```

```json expect=author-papers contains
{
  "author": {"kind": "exact_provider", "id": "semanticscholar:1716151"},
  "papers": [{
    "paper_id": "paper-identity-1",
    "pmid": "40215974",
    "doi": "10.1016/j.fixture.2024.01.001",
    "title": "A compact author paper fixture",
    "journal": "Fixture Medicine",
    "year": 2024
  }],
  "pagination": {"offset": 0, "limit": 1, "next": 1},
  "_meta": {
    "source_status": [{"source": "semantic_scholar", "status": "available"}],
    "evidence_urls": [{"url": "https://www.semanticscholar.org/paper/paper-identity-1"}],
    "next_commands": [
      "biomcp get article 40215974",
      "biomcp author papers semanticscholar:1716151 --limit 1 --offset 1"
    ]
  }
}
```

The reverse pivot accepts an article-family identifier and returns separate,
provider-qualified author records rather than merging a byline into names.
Affiliations remain sourced assertions, and each author leads to exact detail.

<!-- mustmatch-lint: skip -->

```bash run id=article-authors exit=0
../../tools/biomcp-ci --json article authors 2110.01406
```

```json expect=article-authors contains
{
  "article": {"paper_id": "paper-byline-1", "arxiv_id": "2110.01406", "title": "A paper with provider-exact authors", "year": 2021},
  "authors": [
    {
      "identity": {"kind": "exact_provider", "id": "semanticscholar:1716151"},
      "display_name": "A. Butte",
      "affiliations": [{"value": "University of California, San Francisco", "evidence": {"source": "semantic_scholar"}}]
    },
    {
      "identity": {"kind": "exact_provider", "id": "semanticscholar:2269573451"},
      "display_name": "Louis S. Williams",
      "affiliations": [{"value": "Cleveland Clinic", "evidence": {"source": "semantic_scholar"}}]
    }
  ],
  "_meta": {
    "source_status": [{"source": "semantic_scholar", "status": "available"}],
    "evidence_urls": [
      {"url": "https://www.semanticscholar.org/author/1716151"},
      {"url": "https://www.semanticscholar.org/author/2269573451"}
    ],
    "next_commands": [
      "biomcp get author semanticscholar:1716151",
      "biomcp get author semanticscholar:2269573451"
    ]
  }
}
```

Both pivots keep the detail surface's allowlist. Provider extras nested on a
paper or byline author do not cross the public boundary.

```text expect=author-papers not-contains
"email":
"homepage":
"private_profile":
"gender":
"race":
"ethnicity":
"externalIds":
"external_ids":
private-author@example.invalid
https://private.example.invalid/author
fixture-private-profile
fixture-inferred-demographic
fixture-long-abstract-sentinel
0000-0002-7433-2740
```

```text expect=article-authors not-contains
"email":
"homepage":
"private_profile":
"gender":
"race":
"ethnicity":
"externalIds":
"external_ids":
private-author@example.invalid
https://private.example.invalid/author
fixture-private-profile
fixture-inferred-demographic
0000-0002-7433-2740
```

## Markdown and discovery explain the S2-only boundary

Human-readable results retain copyable provider IDs and explain the identity limit. The list page teaches only the search and detail operations available in this first release.

```bash
../../tools/biomcp-ci search author -q "Louis Williams" --source semanticscholar --limit 5 | mustmatch like "Identity: exact provider
Source: Semantic Scholar
semanticscholar:2269573451
semanticscholar:1994488914"
```

```bash
../../tools/biomcp-ci get author semanticscholar:1716151 | mustmatch like "Identity: exact provider
Source: Semantic Scholar
ORCID link: not established
semanticscholar:1716151"
```

```bash
../../tools/biomcp-ci list author | mustmatch like "search author -q <name>
get author semanticscholar:<id>
--source semanticscholar"
```
