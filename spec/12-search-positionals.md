# Search Positional Intuitiveness

This file validates positional-query consistency across `search` subcommands and the variant-specific `GENE CHANGE` normalization path. Assertions target stable query echoes and section headers.

| Section | Command focus | Why it matters |
|---|---|---|
| Variant positional unquoted | `search variant BRAF V600E` | Ensures multi-token positional joins into a single query |
| Variant positional quoted | `search variant "BRAF V600E"` | Ensures quoted gene-change auto-splits to gene+hgvsp |
| Variant positional long-form | `search variant BRAF p.Val600Glu` | Ensures long-form protein notation normalizes into typed gene+hgvsp search |
| Variant positional complex free text | `search variant "EGFR Exon 19 Deletion"` | Ensures non-simple text stays in search space (`condition`) |
| Variant positional plus flag | `search variant BRAF V600E --limit 5` | Ensures positional and later flags coexist |
| Trial positional | `search trial melanoma` | Ensures positional maps to `--condition` |
| Trial positional multi-word | `search trial "non-small cell lung cancer"` | Ensures multi-word positional condition works |
| PGx positional | `search pgx CYP2D6` | Ensures positional maps to `--gene` |
| GWAS positional | `search gwas BRAF` | Ensures positional maps to `--gene` |
| Adverse-event positional | `search adverse-event pembrolizumab` | Ensures positional maps to `--drug` |
| Search-all positional | `search all BRAF` | Ensures positional maps to `--keyword` |
| Trial positional plus status | `search trial melanoma --status recruiting` | Ensures positional and explicit flags coexist |

## Variant Positional Unquoted (`GENE CHANGE`)

Unquoted multi-token input should parse and normalize to the same query shape as explicit `--gene` + `--hgvsp`.

```bash
out="$(biomcp search variant BRAF V600E --limit 3)"
echo "$out" | mustmatch like "gene=BRAF"
echo "$out" | mustmatch like "hgvsp=V600E"
```

## Variant Positional Quoted (`GENE CHANGE`)

Quoted `GENE CHANGE` input should resolve through the same normalization path and produce identical query markers.

```bash
out="$(biomcp search variant "BRAF V600E" --limit 3)"
echo "$out" | mustmatch like "gene=BRAF"
echo "$out" | mustmatch like "hgvsp=V600E"
```

## Variant Positional Long-Form

Long-form protein notation should normalize to the same canonical typed query as
the short one-letter form.

```bash
out="$(biomcp search variant BRAF p.Val600Glu --limit 3)"
echo "$out" | mustmatch like "gene=BRAF"
echo "$out" | mustmatch like "hgvsp=V600E"
```

## Variant Positional Complex Free Text

Complex clinical phrases should remain in filter search space, not exact-ID lookup space.

```bash
out="$("$(git rev-parse --show-toplevel)/target/release/biomcp" search variant "EGFR Exon 19 Deletion" --limit 3)"
echo "$out" | mustmatch like "gene=EGFR"
echo "$out" | mustmatch like "consequence=inframe_deletion"
echo "$out" | mustmatch like "# Variant Search Results"
```

## Variant Positional With Flag Coexistence

Positional query tokens must still allow regular options that follow.

```bash
out="$(biomcp search variant BRAF V600E --limit 5)"
echo "$out" | mustmatch like "gene=BRAF"
echo "$out" | mustmatch like "hgvsp=V600E"
```

## Trial Positional Query

Trial positional input should map to the condition filter.

```bash
out="$(biomcp search trial melanoma --limit 3)"
echo "$out" | mustmatch like "condition=melanoma"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## Trial Positional Multi-word Query

Quoted multi-word condition values should map cleanly to `--condition`.

```bash
out="$(biomcp search trial "non-small cell lung cancer" --limit 3)"
echo "$out" | mustmatch like "condition=non-small cell lung cancer"
echo "$out" | mustmatch like "|NCT ID|Title|Status|Phase|Conditions|"
```

## PGx Positional Query

PGx positional input should alias to the gene filter.

```bash
out="$(biomcp search pgx CYP2D6 --limit 3)"
echo "$out" | mustmatch like "# PGx Search: gene=CYP2D6"
echo "$out" | mustmatch like "| Gene | Drug | CPIC Level | PGx Testing | Guideline |"
```

## GWAS Positional Query

GWAS positional input should alias to the gene filter.

```bash
out="$(biomcp search gwas BRAF --limit 3)"
echo "$out" | mustmatch like "# GWAS Search: gene=BRAF"
echo "$out" | mustmatch like "| rsID | Trait | p-value | Effect | Risk AF | Genes | Study | PMID |"
```

## Adverse-event Positional Query

Adverse-event positional input should alias to the FAERS drug filter.

```bash
out="$(biomcp search adverse-event pembrolizumab --limit 3)"
echo "$out" | mustmatch like "# Adverse Events: drug=pembrolizumab"
echo "$out" | mustmatch like "|Report ID|Drug|Reactions|Serious|"
```

## Search-all Positional Query

Search-all positional input should alias to keyword search.

```bash
out="$(biomcp search all BRAF --limit 3)"
echo "$out" | mustmatch like "# Search All: keyword=BRAF"
echo "$out" | mustmatch like "## Articles"
```

## Trial Positional Plus Status Flag

Positional and explicit filter flags should compose without ambiguity.

```bash
out="$(biomcp search trial melanoma --status recruiting --limit 3)"
echo "$out" | mustmatch like "condition=melanoma"
echo "$out" | mustmatch like "status=recruiting"
```
