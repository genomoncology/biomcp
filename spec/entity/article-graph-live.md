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
<!-- mustmatch-lint: skip -->

Semantic Scholar can return a PMID, DOI, arXiv ID, or only its own paper ID.
The human-readable citation and recommendation tables therefore label the column
generically instead of calling every available value a PMID. Deterministic
renderer tests pin each typed row label because live provider identifiers vary.

```bash run id=typed-article-citations exit=0
../../tools/biomcp-ci --no-cache article citations 22663011 --limit 5
```

```text expect=typed-article-citations contains
| Identifier | Title | Intents | Influential | Context |
```

```text expect=typed-article-citations not-contains
| PMID | Title | Intents | Influential | Context |
```

```bash run id=typed-article-recommendations exit=0
../../tools/biomcp-ci --no-cache article recommendations 22663011 --limit 5
```

```text expect=typed-article-recommendations contains
| Identifier | Title | Journal | Year |
```

```text expect=typed-article-recommendations not-contains
| PMID | Title | Journal | Year |
```

## Article Search Uses an Honest Identifier Heading
<!-- mustmatch-lint: skip -->

Semantic Scholar search rows do not always have PubMed identifiers. The live
human-readable search table therefore uses a neutral heading; deterministic
renderer tests pin the PMID, DOI, arXiv, and provider-only row labels.

```bash run id=typed-article-search exit=0
../../tools/biomcp-ci --no-cache search article --source semanticscholar --keyword "cancer genomics" --limit 1
```

```text expect=typed-article-search contains
| Identifier | Title | Source(s) | Date | Why | Cit. |
```

```text expect=typed-article-search not-contains
| PMID | Title | Source(s) | Date | Why | Cit. |
```
