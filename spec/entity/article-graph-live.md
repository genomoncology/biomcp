# Live Article Graph Collections

This operator-run canary checks the real Semantic Scholar graph boundary used
by article recommendations. It pins only stable response shape because the
provider can revise its related-paper corpus over time.

## Empty Article Recommendations Keep an Iterable JSON Collection

<!-- mustmatch-lint: skip -->

A successful recommendation lookup always returns its named collection, even
when Semantic Scholar has no related papers. JSON callers can therefore iterate
`recommendations` without first repairing a missing field.

```bash run id=empty-article-recommendations exit=0
../../tools/biomcp-ci --no-cache --json article recommendations 23450558 --limit 5
```

```json expect=empty-article-recommendations contains
{"recommendations": []}
```

## Related-paper Tables Label Their Identifier Honestly

Semantic Scholar can return a PMID, DOI, arXiv ID, or only its own paper ID.
The human-readable recommendations table therefore labels the column generically
and identifies each available value by its real type instead of calling every
value a PMID.

```bash run id=typed-article-recommendations exit=0
../../tools/biomcp-ci --no-cache article recommendations 22663011 --limit 5
```

```text expect=typed-article-recommendations contains
| Identifier | Title | Journal | Year |
```

```text expect=typed-article-recommendations not-contains
| PMID | Title | Journal | Year |
```
