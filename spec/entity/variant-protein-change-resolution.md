# Variant Protein-Change Resolution

A gene+protein query names a protein change, not a genomic variant. Two
provider facts force a choice. dbNSFP merges every isoform's protein name
into one alias list, so the same alias sits on several genomic variants
(`DICER1 p.Met1483Ile` spans three alternate bases at chr14:g.95562808, and
`TP53 C124Y` also names the `p.Cys135Tyr` variant chr17:g.7578526C>T through
a shorter isoform). And a ClinVar record names its own genomic variant, which
may be a different variant than the one the user named. `get variant`
therefore counts a hit as a match only when the transcript BioMCP headlines
for it — the ClinVar-named or canonical SnpEff annotation — spells the
requested change; ClinVar presence breaks ties among those named matches
(ticket 1297) but never outranks the named change (ticket 2016, where the
1297 rule returned `p.Cys135Tyr` for `TP53 C124Y` and `p.Ala1823Thr` for
`BRCA1 A314T`). A unique provider match resolves, the one ClinVar record
among several true matches resolves, and everything else refuses with every
candidate and a working input form, so a protein-change query never silently
returns a different variant. Protein-range forms such as `EGFR E746_A750del`
keep refusing with the search form: ClinVar catalogs several distinct
exon-19 deletions under that protein notation (VariationIDs 45233, 163343,
and 177620 include a delins), so expanding the alias would guess. These rows
replay recorded MyVariant.info responses through the routine variant-identity
fixture; the fixture answers only the recorded queries.

| query | code | id | candidates |
|---|---|---|---|
| BRAF V600E | ok | chr7:g.140453136A>T | 0 |
| TP53 C124Y | ok | chr17:g.7579316C>T | 0 |
| BRCA1 A314T | ok | chr17:g.41246608C>T | 0 |
| BRCA1 C61G | ok | chr17:g.41258504A>C | 0 |
| DICER1 p.Met1483Ile | ok | chr14:g.95562808C>T | 0 |
| EGFR M766I | invalid_argument | none | 3 |

```bash each_row="Variant Protein-Change Resolution"
biomcp --json --no-cache get variant '{{query}}' \
  | jq -c 'if .error then {code: .error.code, id: "none", candidates: ([.error.message | scan("- chr[^\\n]+")] | length)} else {code: "ok", id: .id, candidates: 0} end' \
  | mustmatch like '{"code":"{{code}}","id":"{{id}}","candidates":{{candidates}}}'
```

The recorded `TP53 C124Y` response ranks the ClinVar-less
`chr17:g.7579316C>T` (canonical `NM_000546.5:c.371G>A p.Cys124Tyr`) first
and also returns the ClinVar 141762 lookalike `chr17:g.7578526C>T`
(`p.Cys135Tyr`), so the row proves the named change beats a ClinVar-recorded
lookalike. `BRCA1 A314T` is the same shape against ClinVar 55588. The
recorded `BRCA1 C61G` response carries ClinVar records on both alias hits
(409329 naming `p.Cys1828Gly`, 17661 naming `p.Cys61Gly`); exactly one hit
spells `C61G` on its canonical transcript, so it resolves instead of
refusing as conflicting records. The recorded DICER1 response ranks the
ClinVar-less variant `chr14:g.95562808C>A` first, so a first-match pick
answers the wrong variant; the row proves the ClinVar-named
`chr14:g.95562808C>T` (VariationID 577152, rs1454569806) wins among three
true matches. A refusal names every candidate with a working input form.

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
