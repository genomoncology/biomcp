# Get Variant Input Forms

`get variant` resolves the identity forms ClinVar reports carry: a
gene-plus-protein change, transcript HGVS with intronic offsets, a bare ClinVar
VariationID, and an rsID (ticket 1292). ClinVar-style names that keep the gene
in parentheses refuse with the colon working form instead of half-parsing, and
a protein-range deletion no source annotates reports an honest miss with a
search form. These rows replay recorded MyVariant, Mutalyzer, VariantValidator,
and ClinGen Allele Registry responses through the routine fixture bases.

## Resolving input forms

| str:input | str:gene | str:id | str:label |
|---|---|---|---|
| BRAF V600E | BRAF | chr7:g.140453136A>T | gene-protein substitution |
| NM_000249.4:c.678-14_678-3del | MLH1 | chr3:g.37055906_37055917del | intronic deletion range |
| NM_177438.3:c.4449G>A | DICER1 | chr14:g.95562808C>T | colon transcript form |
| 577152 | DICER1 | chr14:g.95562808C>T | ClinVar VariationID |
| rs334 | HBB | chr11:g.5248232T>A | rsID |

```bash each_row="Resolving input forms"
biomcp --json --no-cache get variant '{{input}}' \
  | jq -c '{id, gene, genome_build}' \
  | mustmatch like '{"id":"{{id}}","gene":"{{gene}}","genome_build":"GRCh37"}'
```

Both transcript rows normalize first; Mutalyzer refuses the intronic deletion
range and each normalized coordinate misses the MyVariant HG19 index, so
resolution falls through to ClinVar's own coding alias list. A hit answers only
when it carries the exact transcript-qualified alias the search asked for, so a
different transcript's variant never resolves silently.

Protein-range deletions parse as exact gene-plus-protein changes. dbNSFP holds
no alias for the EGFR exon 19 deletion, so the lookup reports a miss and prints
a working search form rather than guessing a neighboring variant.

```bash run id=protein-range-deletion-honest-miss exit=1
biomcp --json --no-cache get variant 'EGFR E746_A750del'
```

```json expect=protein-range-deletion-honest-miss contains
{
  "error": {
    "code": "not_found"
  }
}
```

```bash run id=protein-range-deletion-search-form
biomcp --json --no-cache get variant 'EGFR E746_A750del' 2>/dev/null \
  | jq -r '.error.message' \
  | grep -F 'Try searching: biomcp search variant -g EGFR --hgvsp E746_A750del' \
  | mustmatch 'Try searching: biomcp search variant -g EGFR --hgvsp E746_A750del'
```

ClinVar-style names with the gene in parentheses keep refusing cleanly; the
refusal prints the colon form as a working input.

## ClinVar-style names refuse with the colon working form

| str:input | str:working_form | str:label |
|---|---|---|
| NM_000249.4(MLH1):c.678-14_678-3del | NM_000249.4:c.678-14_678-3del | parenthesized colon name |
| NM_177438.3(DICER1) c.4449G>A | NM_177438.3:c.4449G>A | parenthesized space name |

```bash run id=clinvar-style-names-refuse exit=2 each_row="ClinVar-style names refuse with the colon working form"
biomcp --json --no-cache get variant '{{input}}'
```

```json expect=clinvar-style-names-refuse contains each_row="ClinVar-style names refuse with the colon working form"
{
  "error": {
    "code": "invalid_argument"
  }
}
```

```bash each_row="ClinVar-style names refuse with the colon working form"
biomcp --json --no-cache get variant '{{input}}' 2>/dev/null \
  | jq -r '.error.message' \
  | grep -F 'Working form: biomcp get variant {{working_form}}' \
  | mustmatch 'Working form: biomcp get variant {{working_form}}'
```
