# Live Article Graph Collections

This operator-run canary checks the real Semantic Scholar graph boundary used
by article recommendations. It pins only stable response shape because the
provider can revise its related-paper corpus over time.

## Article Search Uses an Honest Identifier Heading
<!-- mustmatch-lint: skip -->

Semantic Scholar search rows do not always have PubMed identifiers. The live
human-readable search table therefore uses a neutral heading; deterministic
renderer tests pin the PMID, DOI, arXiv, and provider-only row labels.

```bash run id=typed-article-search exit=0
# Use the release binary so this Semantic Scholar canary receives S2_API_KEY.
../../target/release/biomcp --no-cache search article --source semanticscholar --keyword "cancer genomics" --limit 1
```

```text expect=typed-article-search contains
| Identifier | Title | Source(s) | Date | Why | Cit. |
```

```text expect=typed-article-search not-contains
| PMID | Title | Source(s) | Date | Why | Cit. |
```
