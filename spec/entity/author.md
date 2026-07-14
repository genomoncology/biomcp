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
      {"identity": {"kind": "exact_provider", "id": "semanticscholar:1994488914"}, "display_name": "Louis S. Williams"}
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
    "evidence_urls": [{"url": "https://www.semanticscholar.org/author/2269573451"}],
    "next_commands": ["biomcp get author semanticscholar:2269573451"]
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
private-author@example.invalid
https://private.example.invalid/author
fixture-private-profile
fixture-inferred-demographic
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
    "next_commands": []
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
private-author@example.invalid
https://private.example.invalid/author
fixture-private-profile
fixture-inferred-demographic
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
