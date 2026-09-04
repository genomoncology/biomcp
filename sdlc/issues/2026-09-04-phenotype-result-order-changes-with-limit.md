# Phenotype result order changes with the requested limit

Severity: should-fix

The same phenotype query returned a different first result when the provider limit changed. Direct Monarch semantic-similarity requests for `HP:0000256` returned an X-linked macrocephaly syndrome first with `limit=2`. The request returned isolated microcephaly first with `limit=3` and `limit=5`. This breaks the normal paging expectation that a larger page begins with the same rows as a smaller page.

BioMCP makes the provider result depend on the requested CLI window. `search_phenotype_page` in `src/entities/disease/search.rs` computes `fetch_limit` from `offset + limit + 1`, sends that limit to Monarch, sorts the returned subset locally, and then truncates it. A request for two rows can therefore receive a different candidate pool and order from a request for three rows. Offset pagination can skip or repeat diseases for the same reason.

The provider supports at most 50 results for this BioMCP surface. BioMCP can fetch one stable candidate window and page that result locally. The command should report a visible coverage limit when the provider cannot supply a complete result set.

## Success criteria

- Results for one normalized phenotype query have a stable prefix across supported limits.
- Paging through offsets returns each candidate once in the same order as one request for the combined window.
- The result states the provider coverage limit when BioMCP cannot inspect more candidates.
- A fixed provider fixture reproduces the limit-dependent order and proves the correction without a live request.

Found on 2026-09-04 during a rare disease case research exercise.
