---
flow: build
priority: 9
---
# Pair a variant's protein change with its own transcript

## Done when

`biomcp get variant "chr10:g.89720808T>G"` reports `p.Leu320*` beside
`c.959T>G`, and the same pair appears in `search variant --gene PTEN`.
A regression test pins a variant whose `dbnsfp.hgvsp` array is ordered
against its `hgvsc` array, so indexing element zero fails the test.

## The finding

Raised as an issue during BioMCP research on 2026-08-08, then folded
in here and the issue file removed. Reproduced in full below.

<!-- from protein-annotation-can-come-from-a-different-transcript.md -->

# A variant's protein change can come from a different transcript than its cDNA

Severity: blocking. This one reports a wrong clinical fact, not a
missing one.

`biomcp get variant "chr10:g.89720808T>G"` prints:

    # PTEN p.Leu129* (rs1114167667)
    Protein: p.Leu129*
    Legacy Name: PTEN L129stop
    cDNA: c.959T>G

Those two lines describe different transcripts. On `NM_000314`
(MANE Select) the variant is `c.959T>G` → **p.Leu320\***. The
`p.Leu129*` reading belongs to `NM_001304718`, where the same
genomic change is `c.386T>G`. The row asserts a stop at residue 129
of a 403-residue protein when the truncation is actually at 320 —
different domain, and for PTEN specifically it lands on the wrong
side of the c.1121 / p.D375 boundary reasoning that PVS1 turns on.

Root cause is a pick-the-first over an unordered array. MyVariant
returns:

    dbnsfp.hgvsp  = ["p.Leu129*", "p.Leu320*", "p.Leu320Ter",
                     "p.L320X", "p.Leu129Ter"]
    dbnsfp.hgvsc  = ["c.959T>G", "c.386T>G"]

Element zero of `hgvsp` is not element zero of `hgvsc`. Neither
array is transcript-keyed, so they cannot be zipped.

Upstream has the correct pairing in two places, both already in the
response:

- `clinvar.hgvs.protein` → `NP_000305.3:p.Leu320Ter`, alongside
  `clinvar.hgvs.coding` → `NM_000314.8:c.959T>G`
- `snpeff.ann[]`, where each entry carries `feature_id`, `hgvs_c`
  and `hgvs_p` together — the `NM_000314.6` entry has
  `c.959T>G` / `p.Leu320*`

Fix shape: choose the transcript first, then read cDNA and protein
from that transcript's record. Prefer the MANE Select entry in
`snpeff.ann`, falling back to the matching `clinvar.hgvs` pair.
Never index into `dbnsfp.hgvsp`. If no transcript-consistent pair
can be formed, print the cDNA alone rather than a mismatched pair —
a missing protein change is recoverable, a wrong one is not.

It propagates: the same wrong value fills the Protein and Legacy
Name columns of `biomcp search variant --gene PTEN`, so it is
visible before anyone opens the detail view.

Found 2026-08-08 while researching PTEN GN003 for varclassify2.
