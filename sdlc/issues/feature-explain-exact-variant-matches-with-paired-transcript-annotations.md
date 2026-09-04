# Explain exact variant matches with paired transcript annotations

Severity: should-fix

A rare disease case report used `HSD17B4 c.1619A>G (p.His540Arg)`. BioMCP resolved the protein query, but its human-readable result displayed a different coding and protein pair without explaining the relationship:

```text
$ biomcp search variant -g HSD17B4 --hgvsp H540R --limit 10
Requested variant: HSD17B4 H540R
Resolution: Resolved
| chr5:g.118860951A>G | GRCh37 | HSD17B4 | NM_000414.3 | c.1544A>G | p.His515Arg | ... |
```

The JSON proves why BioMCP retained the row. It reports `matched_alias: p.H540R` and includes `p.His540Arg` and `c.1619A>G` in separate `source_identity` arrays. Those flat arrays do not show which transcript joins the coding and protein descriptions.

The upstream MyVariant record supplies the missing pairs in `snpeff.ann`:

```text
NM_001199291.2  c.1619A>G  p.His540Arg
NM_000414.3     c.1544A>G  p.His515Arg
```

ClinVar names the second representation as its preferred annotation. The source data are internally consistent. BioMCP correctly keeps each selected SNPEff annotation paired in `select_transcript_annotation` at `src/transform/variant.rs:75-105`. Exact filtering then stores the matched alias at `src/entities/variant/search/mod.rs:839-844`. `templates/variant_search.md.j2` does not render `matched_alias`, and `SourceVariantIdentity` flattens alternate transcript annotations into independent arrays. BioMCP therefore discards the relationship that would explain the result to a person or an agent.

The workaround required a direct MyVariant API request and manual inspection of `snpeff.ann`. A case workflow cannot safely infer transcript pairings from the flat arrays.

## Cheapest useful shape

1. Show `matched_alias` in Markdown whenever it differs from the displayed coding or protein annotation. Explain that the table uses another transcript.
2. Preserve paired transcript, coding, and protein annotations in structured exact-search output. Mark the displayed or preferred annotation.
3. Consider MANE or caller-selected transcript ranking later. That policy needs a separate design and does not block the first two improvements.

## Success criteria

- The HSD17B4 command explains that `p.His540Arg` matched an alternate transcript while the row displays `p.His515Arg`.
- JSON preserves the paired `NM_001199291.2`, `c.1619A>G`, and `p.His540Arg` annotation from the fixed provider response.
- JSON also preserves the paired selected annotation and marks it as selected.
- Markdown does not imply that `p.His540Arg` was renumbered on the displayed transcript.
- Broad variant searches retain their compact output unless an alternate match needs explanation.
- A deterministic provider fixture proves the behavior without a live request.
