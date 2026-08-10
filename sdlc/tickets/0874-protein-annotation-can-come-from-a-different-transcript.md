---
flow: build
priority: 10
---
# Keep coding and protein changes on the same transcript

This is a clinical correctness defect. Independent array positions are not a
transcript relationship.

## Done when

For chr10:g.89720808T>G, detail and PTEN search output pair
NM_000314 c.959T>G with p.Leu320*, never p.Leu129*. BRAF V600E detail and
search no longer derive a legacy BRAF V640E identity from another transcript.

The chosen coding HGVS, protein HGVS, transcript, and legacy name always come
from one transcript-associated provider record.

## Selection contract

1. Request and parse transcript-keyed MyVariant data such as snpeff.ann and
   ClinVar coding/protein HGVS pairs.
2. Apply the existing canonical transcript preference to those paired records,
   preferring MANE/RefSeq canonical evidence when present.
3. Use ClinVar's paired coding/protein record as fallback when no preferred
   snpEff record exists.
4. Never zip or independently choose from dbnsfp.hgvsc and dbnsfp.hgvsp.
5. If no consistent pair exists, emit the coding change and transcript without
   a protein/legacy value. Missing is safer than a mismatched clinical fact.

The same selected annotation feeds variant detail, variant search, headings,
Protein fields, and Legacy Name. Do not maintain separate first-element paths.

## Proof required

- Real receipted PTEN and BRAF responses pass through the production parser.
- A RequestPlan test proves the transcript-associated fields are requested.
- A deliberately adversarial fixture orders independent arrays differently and
  proves they are ignored for pairing.
- JSON and Markdown pin transcript, coding, protein, and legacy consistency.
- A no-pair fixture proves the protein value is omitted rather than guessed.

## Authorized test changes

Design commits may restate MyVariant request/parser fixtures, variant transform
tests, detail/search CLI tests, renderer tests, and schemas/examples that
currently encode mismatched annotations. Mechanical construction fixes may
land with implementation while unrelated assertions remain unchanged.

The src line ceiling may rise by at most 260 lines.
