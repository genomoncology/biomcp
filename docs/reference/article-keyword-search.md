# Article Keyword Search

This reference documents how `-k/--keyword` behaves and when to pair it with
typed article filters.

## When `--keyword` should stand alone

Use keyword-only article search when the question does not start with a known
gene, disease, or drug anchor.

- Unknown-entity questions: search the evidence first instead of inventing a
  typed `-g`, `-d`, or `--drug` value.
- Dataset or method questions: keep the search free-text and add
  `--type review` when you want synthesis papers or surveys.

Examples:

```bash
biomcp search article -k '"cafe-au-lait spots" neurofibromas disease' --type review --limit 5
biomcp search article -k "TCGA mutation analysis dataset" --type review --limit 5
```

On the default `--source all` route, adding `-k/--keyword` keeps the source set
on PubTator3, Europe PMC, PubMed, and compatible Semantic Scholar while making
the default relevance mode `hybrid`. Use `--source semanticscholar` or
`--source litsense2` explicitly when you want one of those sources alone.
Semantic-aware ranking uses the LitSense2-derived semantic signal when a row has
LitSense2 provenance; rows without LitSense2 provenance contribute `semantic=0`.
BioMCP caps each federated source's contribution after deduplication and before
ranking. Default: 40% of `--limit` on federated pools with at least three
surviving primary sources. Rows count against their primary source after
deduplication. Use `--max-per-source <N>` to override that cap, use
`--max-per-source 0` for the default cap explicitly, and set it equal to
`--limit` to disable capping.

## Exact entity suggestions from keyword-only search

If the whole normalized keyword exactly matches a gene, drug, or disease
vocabulary label or exact alias, article search may add a direct typed follow-up
command. The same command appears in markdown `See also` and in JSON
`_meta.next_commands`; JSON also includes an article-local
`_meta.suggestions[]` object with `command`, `reason`, and `sections`.

Examples:

```bash
biomcp search article -k BRAF --limit 5
biomcp search article -k imatinib --limit 5
biomcp search article -k melanoma --limit 5
```

The exact check is for the whole keyword, not any token inside the phrase.
`BRAF V600E` does not suggest `biomcp get gene BRAF`, and `lung cancer
immunotherapy` does not invent a single disease card. Searches that already
include `-g/--gene`, `-d/--disease`, or `--drug` also suppress these direct
entity suggestions because the typed anchor was chosen explicitly.

## Session loop-breaker suggestions

Use `--session <token>` when a caller may issue multiple keyword-only article
searches for one task and wants JSON guidance if the wording starts to loop.
The token is a local correlation label, not an authentication token or secret.
Use short non-identifying labels such as `lit-review-1`; do not include PHI,
credentials, email addresses, or user identifiers.

Example:

```bash
biomcp --json search article -k "Oncotype DX review" --session lit-review-1 --limit 5
biomcp --json search article -k "Oncotype DX DCIS" --session lit-review-1 --limit 5
```

BioMCP compares consecutive successful article keyword searches with the same
session token. It lowercases the keyword, removes common search filler words,
and triggers when the post-stopword term-set Jaccard overlap is at least 60%.
The session baseline expires after 10 minutes and stores only the last keyword
terms plus up to 20 PMIDs from the previous result page under the local cache
root.

Loop-breaker guidance appears only in JSON `_meta.suggestions[]`; default
markdown output is unchanged. Exact entity suggestions keep `sections`.
Loop-breaker suggestions omit `sections` and are ordered by fallback strategy:

1. `biomcp article batch ...` for the previous search's top PMIDs, when any
   were available.
2. `biomcp discover "<topic>"` to map the current topic to structured
   biomedical entities.
3. A date-narrowed `biomcp search article ... --year-min ... --year-max ...`
   retry derived from the current result page when such a retry is available.

Calls without `--session`, first calls in a new session, disjoint keyword
changes, expired session state, or keywords that normalize to no meaningful
terms do not emit loop-breaker suggestions.

## When `--keyword` should be combined with typed filters

If the gene, disease, or drug is already known, keep that anchor in a typed
flag and use `-k` for mechanisms, phenotypes, outcomes, datasets, and other
free-text concepts.

Example:

```bash
biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5
```

Without `-k`, typed-only searches stay on the compatible federated lexical
route.

## Do not invent typed flags for unknown entities

If the question is "which disease causes this phenotype?" or "which drug causes
this effect?", do not guess a disease or drug name just to fill `-d` or
`--drug`. Start with keyword-only search, then rerun with a typed flag once the
first page reveals the likely anchor. If you need BioMCP to resolve the entity
before you search, use `biomcp discover "<question>"`.

## Keyword behavior

`--keyword` (`-k`) is provider-neutral free text and no longer auto-quotes
whitespace-containing values. It is not a raw PubMed or Europe PMC query field.
Recognized author fields (`[author]`, `[au]`, `AUTH:`), affiliation fields
(`[ad]`, `AFFILIATION:`), and journal fields (`[journal]`, `[jour]`, `JOURNAL:`)
are rejected with guidance to use `--author`, `--journal`, or ordinary unfielded
keyword text. Matching is case-insensitive. Other bracket and colon forms remain
literal, so biomedical text such as `NM_004333.6:c.1799T>A`, `BRAF[variant]`,
and `protein:protein interaction` remains searchable.

PubMed-specific behavior: direct `--source pubmed` searches and the compatible
federated PubMed leg clean bounded question-format filler words from
unfielded gene, disease, drug, and keyword terms before PubMed ESearch.
Markdown and JSON query echoes keep the original wording, PubTator3, Europe
PMC, and Semantic Scholar keep their existing default-route query behavior,
explicit `--source semanticscholar` searches use that same Semantic Scholar
query behavior alone, and explicit `--source litsense2` searches keep the
LitSense2 query behavior.

This allows multi-word keyword retrieval such as:

```bash
biomcp search article -k "large language model clinical trials" --limit 5
```

## Phrase behavior for entity filters

Entity-oriented filters retain phrase quoting behavior:

- `--gene`
- `--disease`
- `--drug`
- `--author` (the default candidate search uses Europe PMC and compatible PubMed;
  stricter filters may narrow further, and explicit PubTator3, Semantic Scholar,
  and LitSense2 author searches are rejected)

Example:

```bash
biomcp search article -g "BRAF V600E" --author "Jane Doe" --limit 5
```

## Combined filters

Filters can also be combined with typed anchors and other article controls:

```bash
biomcp search article --drug amiodarone -k "photosensitivity mechanism" --limit 5
```
