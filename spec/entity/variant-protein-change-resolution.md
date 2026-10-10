# Variant Protein-Change Resolution

A gene+protein query names a protein change, not a genomic variant. Two
provider facts force a choice. dbNSFP merges every isoform's protein name
into one alias list, so the same alias sits on several genomic variants
(`DICER1 p.Met1483Ile` spans three alternate bases at chr14:g.95562808, and
`TP53 C124Y` also names the `p.Cys135Tyr` variant chr17:g.7578526C>T through
a shorter isoform). And a ClinVar record names its own genomic variant, which
may be a different variant than the one the user named. `get variant`
therefore counts a hit as a match only when the transcript BioMCP headlines
for it — the ClinVar-named, MANE, change-naming, or first-NM_ SnpEff
annotation — spells the requested change; ClinVar presence breaks ties among
those named matches
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
| BRCA1 S1551Y | ok | chr17:g.41226371G>T | 0 |
| BRCA1 S395T | ok | chr17:g.41246365A>T | 0 |
| TP53 R209Q | invalid_argument | none | 0 |
| TP53 G112D | invalid_argument | none | 0 |
| TP53 R174H | invalid_argument | none | 0 |
| TP53 S183Y | invalid_argument | none | 0 |
| BRCA1 S1587F | invalid_argument | none | 0 |
| EGFR M766I | invalid_argument | none | 3 |

```bash each_row="Variant Protein-Change Resolution"
biomcp --json --no-cache get variant '{{query}}' \
  | jq -c 'if .error then {code: .error.code, id: "none", candidates: ([.error.message | scan("- chr[^\\n]+")] | length)} else {code: "ok", id: .id, candidates: 0} end' \
  | mustmatch like '{"code":"{{code}}","id":"{{id}}","candidates":{{candidates}}}'
```

## Protein-change numbering

The headline transcript is the MANE one whenever the answer can mark it:
ClinVar names a variant on the gene's MANE Select transcript when one exists,
so the transcript stem ClinVar's preferred names agree on marks MANE for the
query, and a hit without its own ClinVar record headlines that transcript
ahead of the first NM_ in the SnpEff list (BRCA1's NM_007300 leads the SnpEff
list and numbers residues 21 higher than MANE Select NM_007294 past the
isoform insert). A ClinVar-free response carries no such name, so BioMCP
reads the gene's canonical (MANE Select) protein from UniProt before
accepting any annotation that is not on the MANE transcript: the record's
MANE-Select cross-reference names the MANE transcript with its current
RefSeq version (NM_000546.6, NM_007294.4) where MyVariant's SnpEff build
still headlines the older versions (NM_000546.5, NM_007294.3).
MyVariant.info's SnpEff and dbNSFP sections carry no MANE status of their
own, so a response whose ClinVar preferred names disagree, and whose
canonical record is unavailable, headlines the annotation that spells the
requested change — the signal a ClinVar-free response still carries — and
falls back to the first NM_ when nothing names the request. `BRCA1 I1568N`
shows the ClinVar-free shape: nothing in the response carries a ClinVar
name, and the hit's NM_007294 annotation spells `p.Ile1568Asn`, so the
answer resolves on MANE Select numbering — now headlining the version the
MANE-Select reference carries (NM_007294.4) — with no note instead of
answering `p.Ile1589Asn` on the first NM_ with a note claiming the request
follows another transcript's numbering. The answer also shows one
accession per stem — ClinVar's preferred names and the MANE-Select
reference both carry the current RefSeq version, so `TP53 R116Q` and
`BRCA1 A1844T` show the version ClinVar names (NM_000546.6, NM_007294.4)
even though the SnpEff annotations sit on the older versions. When the
resolved hit's protein change does not spell the request and the request
named residues, the answer says which numbering matched
(`protein_numbering_note`): `TP53
R116Q` resolves to the `p.Arg248Gln` variant through a shorter isoform's
numbering, and `BRCA1 A1844T` (the long isoform's spelling, past the
insert) resolves to the MANE spelling `p.Ala1823Thr` — both with the note.
Two alias hits where exactly one names the change on MANE resolve that
candidate instead of refusing: `BRCA1 S1551Y` (the other match spells the
request only on NM_007297) and `BRCA1 S395T` (the other only on
NM_007298).

The note is only honest when the request's numbering genuinely differs from
MANE's, so BioMCP checks the requested reference residue against the gene's
canonical (MANE Select) protein sequence from UniProt before printing it —
and before accepting any annotation that is not on the MANE transcript
(ticket 2036). When that protein carries the requested residue at the
requested position, the request's own numbering is valid on MANE — and if
no matching record names the change there, the only alias match is a
lookalike naming a different change, so `get variant` refuses instead of
returning it with the other-transcript note: `TP53 R209Q` (Arg at 209, only
match `p.Arg248Gln`), `TP53 G112D` (Gly at 112, only match `p.Gly244Asp`),
and `TP53 R174H` (Arg at 174, only match `p.Arg333His`). The refusal also
covers the isoform-only spelling: `TP53 S183Y` matches
`chr17:g.7576902G>T` only because the shorter isoform NM_001126115.1 names
`p.Ser183Tyr` there (UniProt P04637 has Ser at 183 while MANE Select
NM_000546.6 names `p.Ser315Tyr`), and `BRCA1 S1587F` matches
`chr17:g.41223234G>A` only through the long isoform NM_007300.3's
`p.Ser1587Phe` (UniProt P38398 has Ser at 1587 while MANE Select
NM_007294.4 names `p.Ser1566Phe`); both refuse instead of resolving the
isoform spelling with no note. An unavailable sequence, or a request
that names no reference residue, proves nothing either way — an unverified
other-transcript claim is exactly the false note this check removes — so
the answer carries no note rather than an unproven one.

The check also covers the cases a residue comparison cannot reach
(ticket 2042). A requested position the canonical protein cannot hold
proves the request follows another numbering without a residue to
compare: `BRCA1 Y1866D` — P38398 ends at residue 1863 — resolves the
MANE headline `p.Tyr1845Asp` on NM_007294.4 with the other-transcript
note, and `BRCA1 Q1878R`, whose SnpEff list carries 48
`feature_type: interaction` protein-structure rows the projection now
reads past instead of dropping the whole list for their count, resolves
`p.Gln1857Arg` on NM_007294.4 the same way. A record that names no
protein change on any transcript refuses rather than answer a bare
genomic variant. And when the canonical facts never arrive — UniProt
unreachable, or the record naming no MANE transcript — nothing is
proven either way, so no other-numbering claim prints: a headline that
spells the request resolves under a note saying plainly the numbering
could not be checked against MANE, and a headline naming a different
change refuses.

| query | id | hgvs_p | transcript | note |
|---|---|---|---|---|
| BRCA1 A314T | chr17:g.41246608C>T | p.Ala314Thr | NM_007294.4 | no |
| TP53 R116Q | chr17:g.7577538C>T | p.Arg248Gln | NM_000546.6 | yes |
| BRCA1 A1844T | chr17:g.41199660C>T | p.Ala1823Thr | NM_007294.4 | yes |
| BRCA1 I1568N | chr17:g.41223228A>T | p.Ile1568Asn | NM_007294.4 | no |
| BRCA1 S1551Y | chr17:g.41226371G>T | p.Ser1551Tyr | NM_007294.4 | no |
| BRCA1 S395T | chr17:g.41246365A>T | p.Ser395Thr | NM_007294.4 | no |
| BRCA1 Y1866D | chr17:g.41197754A>C | p.Tyr1845Asp | NM_007294.4 | yes |
| BRCA1 Q1878R | chr17:g.41197717T>C | p.Gln1857Arg | NM_007294.4 | yes |

```bash each_row="Protein-change numbering"
biomcp --json --no-cache get variant '{{query}}' \
  | jq -c '{id: .id, hgvs_p: .hgvs_p, transcript: .transcript, note: (if .protein_numbering_note then "yes" else "no" end)}' \
  | mustmatch like '{"id":"{{id}}","hgvs_p":"{{hgvs_p}}","transcript":"{{transcript}}","note":"{{note}}"}'
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
true matches. An ambiguity refusal still names every candidate with a
working input form; a numbering refusal names the isoform that spells the
request instead of a bare candidate list (ticket 2042).

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

```bash run id=mane-numbering-refusal exit=2
biomcp --json --no-cache get variant 'TP53 R209Q'
```

```json expect=mane-numbering-refusal contains
{
  "error": {
    "code": "invalid_argument",
    "message": "Invalid argument: No MANE-numbered variant matches 'TP53 R209Q': the gene's canonical protein (UniProt P04637) has Arg at 209, so the requested numbering is valid there, but no matching record names that change; the only alias match is p.Arg248Gln on NM_000546.6 — a different change; the request is spelled p.Arg209Gln on NM_001126118.1, another transcript. BioMCP refuses rather than return the wrong variant.\nRetry `biomcp get variant` with a transcript-qualified HGVS naming 'R209Q', or search the spelling: biomcp search variant -g TP53 --hgvsp R209Q."
  }
}
```

```bash run id=isoform-spelling-refusal-tp53-s183y exit=2
biomcp --json --no-cache get variant 'TP53 S183Y'
```

```json expect=isoform-spelling-refusal-tp53-s183y contains
{
  "error": {
    "code": "invalid_argument",
    "message": "Invalid argument: No MANE-numbered variant matches 'TP53 S183Y': the gene's canonical protein (UniProt P04637) has Ser at 183, so the requested numbering is valid there, but no matching record names that change; the only alias match is p.Ser315Tyr on NM_000546.6 — a different change; the request is spelled p.Ser183Tyr on NM_001126115.1, another transcript. BioMCP refuses rather than return the wrong variant.\nRetry `biomcp get variant` with a transcript-qualified HGVS naming 'S183Y', or search the spelling: biomcp search variant -g TP53 --hgvsp S183Y."
  }
}
```

```bash run id=isoform-spelling-refusal-brca1-s1587f exit=2
biomcp --json --no-cache get variant 'BRCA1 S1587F'
```

```json expect=isoform-spelling-refusal-brca1-s1587f contains
{
  "error": {
    "code": "invalid_argument",
    "message": "Invalid argument: No MANE-numbered variant matches 'BRCA1 S1587F': the gene's canonical protein (UniProt P38398) has Ser at 1587, so the requested numbering is valid there, but no matching record names that change; the only alias match is p.Ser1566Phe on NM_007294.4 — a different change; the request is spelled p.Ser1587Phe on NM_007300.3, another transcript. BioMCP refuses rather than return the wrong variant.\nRetry `biomcp get variant` with a transcript-qualified HGVS naming 'S1587F', or search the spelling: biomcp search variant -g BRCA1 --hgvsp S1587F."
  }
}
```

With UniProt unreachable the same requests still cannot answer silently
(ticket 2042): the residue check cannot run, so a headline that spells
the request resolves under a note saying plainly the numbering could not
be checked against MANE, and a lookalike naming a different change
refuses. The dead loopback port stands in for the unreachable provider.

```bash run id=unchecked-numbering-note-uniprot-unreachable exit=0
BIOMCP_UNIPROT_BASE=http://127.0.0.1:9 biomcp --json --no-cache get variant 'TP53 S183Y' \
  | jq -c '{id: .id, hgvs_p: .hgvs_p, transcript: .transcript, note: .protein_numbering_note}' \
  | mustmatch like '{"id":"chr17:g.7576902G>T","hgvs_p":"p.Ser183Tyr","transcript":"NM_001126115.1","note":"Numbering note: resolved on NM_001126115.1 as p.Ser183Tyr; the requested S183Y could not be checked against MANE numbering."}'
```

```bash run id=unchecked-numbering-refusal-uniprot-unreachable exit=2
BIOMCP_UNIPROT_BASE=http://127.0.0.1:9 biomcp --json --no-cache get variant 'TP53 R116Q'
```

```json expect=unchecked-numbering-refusal-uniprot-unreachable contains
{
  "error": {
    "code": "invalid_argument",
    "message": "Invalid argument: No MANE-numbered variant matches 'TP53 R116Q': the canonical (MANE Select) protein could not be read, so the requested numbering could not be checked, and the only alias match is p.Arg248Gln on NM_000546.6 — a different change; the request is spelled p.Arg116Gln on NM_001126115.1, another transcript. BioMCP refuses rather than return the wrong variant.\nRetry `biomcp get variant` with a transcript-qualified HGVS naming 'R116Q', or search the spelling: biomcp search variant -g TP53 --hgvsp R116Q."
  }
}
```
