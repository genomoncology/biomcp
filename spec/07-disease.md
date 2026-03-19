# Disease Queries

Disease commands normalize labels to ontology-backed identifiers and provide cross-entity pivots. This file validates melanoma-centric disease workflows including genes, trials, articles, and drugs. Assertions focus on stable schema and identifier markers rather than dynamic counts.

| Section | Command focus | Why it matters |
|---|---|---|
| Disease search | `search disease melanoma` | Confirms disease normalization output |
| Disease detail | `get disease melanoma` | Confirms canonical disease card |
| Disease genes | `get disease melanoma genes` | Confirms association section rendering |
| Disease to trials | `disease trials melanoma` | Confirms trial helper path |
| Disease to articles | `disease articles melanoma` | Confirms literature helper path |
| Disease to drugs | `disease drugs melanoma` | Confirms treatment helper path |

## Searching by Name

Search should return ontology-backed disease rows and canonical MONDO identifiers. We assert table schema and the melanoma MONDO ID marker.

```bash
out="$(biomcp search disease melanoma --limit 3)"
echo "$out" | mustmatch like "| ID | Name | Synonyms |"
echo "$out" | mustmatch like "MONDO:0005105"
```

## Getting Disease Details

The disease detail card should resolve the query label to a normalized concept. This check targets heading and canonical ID line.

```bash
out="$(biomcp get disease melanoma)"
echo "$out" | mustmatch like "# melanoma"
echo "$out" | mustmatch like "ID: MONDO:0005105"
echo "$out" | mustmatch like "OT "
```

## Disease Genes

Associated-gene expansion is central for translating phenotype-level queries into molecular follow-up. We assert on section heading and table structure.

```bash
out="$(biomcp get disease melanoma genes)"
echo "$out" | mustmatch like "## Associated Genes"
echo "$out" | mustmatch like "| Gene | Relationship | Source | OpenTargets |"
echo "$out" | mustmatch like "overall "
```

## Disease to Trials

Disease helper commands should map directly into trial search with condition context retained. The check asserts query echo and trial columns.

```bash
out="$(biomcp disease trials melanoma --limit 3)"
echo "$out" | mustmatch like "condition=melanoma"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## Disease to Articles

Disease-linked literature retrieval supports rapid evidence triage. Assertions check heading context and the article table schema.

```bash
out="$(biomcp disease articles melanoma --limit 3)"
echo "$out" | mustmatch like "# Articles: disease=melanoma"
echo "$out" | mustmatch like "| PMID | Title |"
```

## Disease to Drugs

Disease-to-drug pivoting provides treatment-oriented context when starting from diagnosis. The output should include indication heading and compact drug table.

```bash
out="$(biomcp disease drugs melanoma --limit 3)"
echo "$out" | mustmatch like "# Drugs: indication=melanoma"
echo "$out" | mustmatch like "|Name|Mechanism|Target|"
```

## Exact Disease Ranking

Exact disease labels should be reranked to the front of the returned page even when upstream ordering is noisy. This regression checks that the canonical colorectal cancer node appears in the surfaced result set.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search disease "colorectal cancer" --limit 10)"
echo "$out" | mustmatch like "| ID | Name | Synonyms |"
echo "$out" | mustmatch like "| MONDO:0024331 | colorectal carcinoma |"
```
