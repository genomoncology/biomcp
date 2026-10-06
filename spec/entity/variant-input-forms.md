# Get Variant Input Forms

`get variant` resolves a gene and protein change, transcript HGVS with intronic offsets, a bare ClinVar VariationID, and an rsID. Parenthesized transcript and gene names use the existing complete source tuple lookup. A protein range deletion without source annotation reports not found and prints a search form. These rows replay recorded MyVariant, Mutalyzer, VariantValidator, and ClinGen Allele Registry responses through the routine fixture bases.

## Resolving input forms

| str:input | str:gene | str:id | str:label |
|---|---|---|---|
| BRAF V600E | BRAF | chr7:g.140453136A>T | gene-protein substitution |
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

Parenthesized names retain BioMCP 1.0's exact transcript, gene and coding tuple selection. The recorded preferred-name assertions supply these detail cards.

## Exact transcript source tuple

| str:input | str:gene | str:id | str:transcript | str:coding | str:protein |
|---|---|---|---|---|---|
| NM_000249.4:c.678-14_678-3del | MLH1 | chr3:g.37055906_37055917del | NM_000249.4 | c.678-14_678-3del | null |
| NM_000249.4(MLH1):c.678-14_678-3del | MLH1 | chr3:g.37055906_37055917del | NM_000249.4 | c.678-14_678-3del | null |
| NM_177438.3:c.4449G>A | DICER1 | chr14:g.95562808C>T | NM_177438.3 | c.4449G>A | "p.Met1483Ile" |
| NM_177438.3(DICER1) c.4449G>A | DICER1 | chr14:g.95562808C>T | NM_177438.3 | c.4449G>A | "p.Met1483Ile" |

```bash each_row="Exact transcript source tuple"
biomcp --json --no-cache get variant '{{input}}' \
  | jq -c '{id, gene, transcript, hgvs_c, hgvs_p, genome_build}' \
  | mustmatch like '{"id":"{{id}}","gene":"{{gene}}","transcript":"{{transcript}}","hgvs_c":"{{coding}}","hgvs_p":{{protein}},"genome_build":"GRCh37"}'
```
