# Variant Protein-Change Resolution

A gene+protein query names a protein change, not a genomic variant: dbNSFP
carries the same alias on several genomic variants when different alternate
bases produce the same substitution (`DICER1 p.Met1483Ile` spans three
alternate bases at chr14:g.95562808). `get variant` resolves the query to the
variant ClinVar itself names when exactly one matching variant carries a
ClinVar record, keeps a unique match, and refuses with every candidate and a
working input form when the match stays ambiguous, so a protein-change query
never silently returns a different variant (ticket 1297). Protein-range forms
such as `EGFR E746_A750del` keep refusing with the search form: ClinVar
catalogs several distinct exon-19 deletions under that protein notation
(VariationIDs 45233, 163343, and 177620 include a delins), so expanding the
alias would guess. These rows replay recorded MyVariant.info responses
through the routine variant-identity fixture; the fixture answers only the
recorded queries.

| query | code | id | candidates |
|---|---|---|---|
| BRAF V600E | ok | chr7:g.140453136A>T | 0 |
| DICER1 p.Met1483Ile | ok | chr14:g.95562808C>T | 0 |
| EGFR M766I | invalid_argument | none | 3 |

```bash each_row="Variant Protein-Change Resolution"
biomcp --json --no-cache get variant '{{query}}' \
  | jq -c 'if .error then {code: .error.code, id: "none", candidates: ([.error.message | scan("- chr[^\\n]+")] | length)} else {code: "ok", id: .id, candidates: 0} end' \
  | mustmatch like '{"code":"{{code}}","id":"{{id}}","candidates":{{candidates}}}'
```

The recorded DICER1 response ranks the ClinVar-less variant
`chr14:g.95562808C>A` first, so a first-match pick answers the wrong variant;
the row proves the ClinVar-named `chr14:g.95562808C>T` (VariationID 577152,
rs1454569806) wins instead. A refusal names every candidate with a working
input form.

```bash run id=protein-change-ambiguity-refusal exit=2
biomcp --json --no-cache get variant 'EGFR M766I'
```

```json expect=protein-change-ambiguity-refusal contains
{
  "error": {
    "code": "invalid_argument",
    "message": "Invalid argument: Ambiguous protein change 'EGFR M766I': 3 variants match and none of them carries a ClinVar record that names one; BioMCP refuses rather than return the wrong variant.\nCandidates:\n- chr7:g.55249000G>A (rs1322818258)\n- chr7:g.55249000G>C (rs1322818258)\n- chr7:g.55249000G>T (rs1322818258)\nRetry `biomcp get variant` with one candidate's exact form: its genomic HGVS, ClinVar VariationID, rsID, or a transcript-qualified HGVS."
  }
}
```
