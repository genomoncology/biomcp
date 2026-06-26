# Variant Cancerhotspots Recurrence

Somatic oncogenicity grading needs recurrence counts from the cohort named by
the criteria. These live canaries check that BioMCP exposes cancerhotspots.org
position-level and exact-amino-acid counts as source-labelled variant detail,
without treating cBioPortal cohort frequencies as the grading numbers.

## Cancerhotspots recurrence counts for somatic oncogenicity tiers
<!-- mustmatch-lint: skip -->

BRAF V600E is the OS3-scale happy path: the cancerhotspots residue count and the
same-amino-acid count should both clear the somatic hotspot thresholds, and the
record should name the matched transcript used for provenance.

```bash run id=braf-v600e-cancerhotspots-recurrence exit=0 timeout=180
biomcp --json --no-cache get variant "BRAF V600E" all | jq -e '
  .cancerhotspots.source == "cancerhotspots.org" and
  (.cancerhotspots.matched_transcript | type == "string" and length > 0) and
  (.cancerhotspots.position_count >= 50) and
  (.cancerhotspots.same_aa_count >= 10)
'
```

MYD88 L265P is the OM3-shape canary: it should have enough exact-amino-acid
recurrence for OM3 but remain below the OS3 position-count threshold. This keeps
BioMCP from silently substituting a larger or unrelated cohort for the
criterion-defining cancerhotspots counts.

```bash run id=myd88-l265p-cancerhotspots-recurrence exit=0 timeout=180
biomcp --json --no-cache get variant "MYD88 L265P" all | jq -e '
  .cancerhotspots.source == "cancerhotspots.org" and
  (.cancerhotspots.matched_transcript | type == "string" and length > 0) and
  (.cancerhotspots.position_count < 50) and
  (.cancerhotspots.same_aa_count >= 10)
'
```

## Variant structure joins residue domains structures and hotspots

The opt-in structure helper should join the requested BRAF V600E residue to the
protein, overlapping InterPro domains, UniProt structures, AlphaFold, and
Cancerhotspots recurrence in one source-labelled JSON response. This is a live
operator canary, not a routine check-lane fixture.

```bash run id=braf-v600e-variant-structure-context exit=0 timeout=240
biomcp --json --no-cache variant structure "BRAF V600E" | jq -r '
  select(.variant == "BRAF V600E") |
  select(.gene == "BRAF") |
  select(.residue.position == 600) |
  select(.residue.position_confidence == "requested_hgvsp_exact_match") |
  select((.domains | type == "array") and (.domains | any(.start <= 600 and .end >= 600))) |
  select((.structures.pdb | type == "array") and (.structures.pdb | length > 0)) |
  select((.structures.alphafold.url | type == "string") and (.structures.alphafold.url | test("alphafold\\.ebi\\.ac\\.uk/entry/P15056"))) |
  select(.cancerhotspots.source == "cancerhotspots.org") |
  select(._meta.next_commands | index("biomcp get protein P15056 structures")) |
  "BRAF V600E structure context includes residue, domain, PDB, AlphaFold, hotspots, and next commands"
' | mustmatch "BRAF V600E structure context includes residue, domain, PDB, AlphaFold, hotspots, and next commands"
```
