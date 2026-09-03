---
flow: build
priority: 3
---

# Article search ranks an off-topic paper above the one that matches the query

`biomcp search article` puts a paper matching one query term in six above a paper matching five in six. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
$ biomcp search article -k "zolgensma treatment of retinoblastoma randomized trial" --limit 3
| Identifier | Title | Why | Cit. |
| PMID 39013849 | Exploring treatment options in cancer: Tumor treatment strat… | hybrid 0.325 + title+abstract 2/6 | 741 |
| PMID 32328653 | Glioblastoma in adults: a Society for Neuro-Oncology (SNO) a… | hybrid 0.3 + title+abstract 1/6 | 982 |
| PMID 26427984 | Conservative treatment of retinoblastoma: a prospective phas… | hybrid 0.261 + title 5/6 | 25 |
```

The query pairs a spinal muscular atrophy drug with an unrelated eye tumour, so no article can satisfy it. Three articles are returned in confident formatting anyway. The only retinoblastoma paper in the set ranks last, below a glioblastoma review that matched one term.

The `Why` column already carries the evidence that the ranking is wrong. The card prints its own formula inline — `0.4*semantic + 0.3*lexical + 0.2*citations + 0.1*position` — and the citation term is doing the work: 982 and 741 citations outrank a 5-of-6 term match on 25 citations.

So the number needed to catch this is on screen and nothing acts on it. The same shape was found independently from a different direction: `sdlc/issues/2026-08-27-article-keyword-search-has-no-relevance-floor.md` records a keyword query returning materials-science papers among real leads, found while an outside agent built a knowledge base.

This matters more than an ordinary ranking complaint because these rows get cited. An agent that quotes the first row quotes a paper about a different disease.

## Required behavior

A result that matches very little of the query is not presented in the same way as one that matches most of it.

A search that finds nothing genuinely relevant can say so, rather than returning its least-bad rows in confident formatting.

Term-match evidence carries weight in the order, not only in the explanation column.

## Done, observably

- The retinoblastoma query above does not rank a one-term match above a five-term match.
- A query whose best match is weak is visibly marked as weak, or returns nothing.
- The `Why` column continues to show per-row match evidence, and the order is consistent with it.

## Boundary

This ticket does not add a source, does not change which sources are federated, and does not remove the citation signal from ranking. The weighting itself is a design decision; the ticket requires only that a reader cannot be handed an off-topic top row without a warning. Trial search has a separate and different ranking problem and is not in scope.
