# Author

BioMCP exposes exact Semantic Scholar author records without claiming they are globally resolved people. Same-name and split provider records remain separate.

## Search

```bash
biomcp search author -q "Louis Williams" --source semanticscholar --limit 5
biomcp --json search author -q "Louis Williams"
```

Results use provider-qualified IDs such as `semanticscholar:2269573451`. Use a returned follow-up command to retrieve one exact record.

## Detail and papers

```bash
biomcp get author semanticscholar:1716151
biomcp author papers semanticscholar:1716151 --limit 10 --offset 0
```

The ID prefix is case-sensitive and the value must be numeric. Unqualified IDs, `pubmed:` IDs, and `orcid:` IDs are not accepted. BioMCP does not establish ORCID links in this release. Paper pages preserve Semantic Scholar order and provider pagination.

Use `biomcp article authors <id>` to pivot from a PMID, PMCID, DOI, arXiv ID, or Semantic Scholar paper ID to separate provider-qualified byline records with sourced affiliations.

Affiliations and counts are source assertions, not a current profile. Coauthor aggregation, topics, affiliation filtering, PubMed candidates, and cross-provider merging are future work.
