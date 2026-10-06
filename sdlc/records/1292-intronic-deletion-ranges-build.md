# 1292 build: resolve intronic deletion ranges in get variant

Ticket: sdlc/tickets/1292-resolve-intronic-deletion-ranges-in-get-variant.md
Branch: tickets/1292-intronic-deletion-ranges, commits ec551cb29, fe1399542,
8dc7e738e, 9a69a8d7e.
Built by the lane B worker; driven by the branch lead.

## Change 1: the exact failing commands and errors

Recorded live on the branch parent (main carrying 1295 and 1301; the parse
code is identical to 0.9.1, and ticket evidence from KB QA 0002 and experiment
439 confirms the same behaviors on 0.9.1):

- `biomcp get variant 'NM_000249.4(MLH1):c.678-14_678-3del' -j` exits 2
  `invalid_argument`: "Unrecognized variant format: 'NM_000249.4(MLH1):c.678-14_678-3del'".
- `biomcp get variant 'EGFR E746_A750del' -j` exits 2 `invalid_argument`:
  "Unrecognized variant format: 'EGFR E746_A750del'".
- `biomcp get variant 'NM_177438.3(DICER1) c.4449G>A' -j` exits 2
  `invalid_argument`: "Unrecognized variant format: 'NM_177438.3(DICER1) c.4449G>A'".
- `biomcp get variant 577152 clinvar -j` exits 2 `invalid_argument`:
  "Unrecognized variant format: '577152'" (ticket evidence names 1463720).
- `biomcp get variant 'NM_000249.4:c.678-14_678-3del' -j` exits 2
  `invalid_argument`: "Could not normalize transcript HGVS for `get variant`:
  'NM_000249.4:c.678-14_678-3del'. Normalization reported: Errors encountered.
  Check the 'custom' field." (Mutalyzer answered HTTP 422 that day.)
- `biomcp get variant 'NM_177438.3:c.4449G>A' -j` exits 1 `not_found` after
  about 23 seconds of all-three normalizer calls: "Try first: biomcp variant
  normalize all NM_177438.3:c.4449G>A".
- `biomcp get variant 'NM_004333.6:c.1799T>A' -j` also exits 1 `not_found`:
  the transcript alias fallback existed but its identity filter rejected every
  ClinVar-only hit (dbNSFP aliases carry no transcript, so comparison came back
  Indeterminate and the confirmed alias hit was dropped).

## Changes

- A bare positive integer classifies as a ClinVar VariationID and resolves
  through MyVariant's `clinvar.variant_id` field. Article search keeps
  refusing the form with a message naming the working forms, so the article
  surface is unchanged.
- Transcript-qualified `get variant` inputs resolve through ClinVar's
  `clinvar.hgvs.coding` alias list when normalization services refuse the
  input (the intronic deletion range) or when the normalized coordinate
  misses MyVariant's HG19 index (both colon forms). A hit answers only when
  it carries the exact transcript-qualified alias the search asked for, so a
  different transcript's variant never resolves silently.
- Both transcript refusal paths print a working form: the ClinVar VariationID
  and rsID routes, alongside the normalize suggestion.
- ClinVar-style names with the gene in parentheses (colon or space form)
  refuse with the derived colon form printed as a working input. The article
  identity parser stops half-parsing the space form as a gene named
  `NM_177438.3(DICER1)`.
- Protein-range deletions (`EGFR E746_A750del`, `p.Glu746_Ala750del`) parse
  as exact gene-plus-protein changes and normalize across one- and
  three-letter spellings; search filters keep the caller's exact spelling
  for complex changes.
- Spec page spec/entity/variant-input-forms.md proves the input-form table
  from recorded MyVariant, Mutalyzer, VariantValidator, and ClinGen Allele
  Registry responses; the fixture 400s on unexpected queries, so rows cannot
  pass vacuously.

## Proof

- Rust suite: 3938 passed (no-default-features) including new resolution
  table tests (VariationID parse, protein-range normalization, ClinVar-style
  refusal messages, transcript-anchor half-parse refusals) and two end-to-end
  alias-fallback tests that resolve and refuse against a local fixture server.
- Spec pages: variant-input-forms.md 14/14 standalone; variant-input-forms +
  variant-gene-first-routing 20/20; variant.md 107 passed under the combined
  provider-contract and variant-identity fixtures.
- make lint green; registration tests green (isolation contract 41 passed,
  article fixture lifecycle 30 passed with the 30 to 31 invocation bump,
  source package boundary 9 passed with 1387 to 1388, capture receipts audit
  306 files all receipted); quality-ratchet inventories repinned.

## Deferred gaps

- `EGFR E746_A750del` parses but resolves to `not_found`: dbNSFP holds no
  protein-range alias for exon 19 deletions and an unverified ClinVar protein
  wildcard could return a delins for a del query. Resolution belongs with
  ticket 1297's protein-change ambiguity work and the 1.0 shared HGVS parser.
- Article variant search still refuses a bare VariationID (the ticket scopes
  the form to `get variant`); teaching the article identity scan the
  VariationID is future work.
- The strict mkdocs build could not run in this sandbox (mkdocs is not
  installed and the offline fetch fails); the CI repository-contract job
  covers it.
- CHANGELOG bullet owed before the next release.
