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
