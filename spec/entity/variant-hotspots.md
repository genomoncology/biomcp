# Variant Cancerhotspots Recurrence

Somatic oncogenicity grading needs recurrence counts from the cohort named by
the criteria. Captured BRAF V600E and MYD88 L265P recurrence is routine coverage
in [Variant Queries](variant.md); this live canary retains the multi-provider
structure workflow.

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
