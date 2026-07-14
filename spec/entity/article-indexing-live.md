# Live PubMed Article Indexing

This operator-run canary checks the real PubMed citation boundary that supplies
researcher and MeSH indexing metadata. It deliberately asserts only stable
response shape because PubMed can revise individual citation fields over time.

## Live PubMed indexing canary

The documented PMID should return available indexing with at least one author
and one MeSH heading. `--no-cache` ensures this verifies the current upstream
response rather than a previously cached citation.

```bash
../../tools/biomcp-ci --no-cache --json get article 22663011 indexing | mustmatch like '{"indexing":{"status":"available","authors":[{}],"mesh_headings":[{}]}}'
```
