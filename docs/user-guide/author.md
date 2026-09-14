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
biomcp author papers semanticscholar:1716151 --full
```

The default page stays compact. `--full` returns the same page with source
metadata per paper: abstract, publication date, citation and reference
counts, open-access data, fields of study, publication types, and the full
byline. Both modes use one bounded Semantic Scholar page (at most 100 rows)
and never fetch a second page. The rich projection is source-exact: it does
not infer affiliations, resolve ORCID, or merge byline records.

`author papers` also accepts an exact ORCID record ID (`orcid:dddd-dddd-dddd-dddC` with its checksum). For ORCID IDs the page lists the public works claimed on that record, locally paginated after one bounded request, and stays compact: `--full` applies only to Semantic Scholar IDs. Semantic Scholar paper pages preserve provider order and pagination.

BioMCP does not infer that an ORCID and a Semantic Scholar author ID name the same person, and `get author` never merges records across providers. ORCID lookups need the `ORCID_ACCESS_TOKEN` environment variable (see the API keys guide).

Use `biomcp article authors <id>` to pivot from a PMID, PMCID, DOI, arXiv ID, or Semantic Scholar paper ID to separate provider-qualified byline records with sourced affiliations.

Affiliations and counts are source assertions, not a current profile. Coauthor aggregation, topics, affiliation filtering, PubMed candidates, and cross-provider merging are future work.
