# 2016 build: protein-change resolution keeps the named variant

Ticket: sdlc/tickets/2016-protein-change-resolution-keeps-the-named-variant.md
Branch: tickets/2016-protein-change-resolution-keeps-the-named-variant, cut
from main bc8b1808b.

## Root cause

Recorded live on 2026-10-07 against MyVariant.info with the production query
shape (`dbnsfp.genename:<GENE> AND dbnsfp.hgvsp:"p.<CHANGE>"` plus the CLI's
field list). Both halves the review suspected are real and compound.

1. dbNSFP merges every isoform's protein name into one `dbnsfp.hgvsp` alias
   list, so the alias search returns other proteins' changes as "matches".
   `TP53 C124Y` returns `chr17:g.7579316C>T` (canonical
   `NM_000546.5:c.371G>A p.Cys124Tyr`, no ClinVar record) and
   `chr17:g.7578526C>T` (canonical `c.404G>A p.Cys135Tyr`, ClinVar 141762),
   whose alias list carries `p.C124Y` through a shorter isoform.
   `compare_variant_identity` sees only the alias list, so both hits are
   compatible.
2. `resolve_protein_change_hit` applied the 1297 rule — exactly one hit
   carries a ClinVar record, so it wins — without checking that the ClinVar
   record's own transcript names the requested change. The ClinVar-recorded
   lookalike outranked the exact named change. `BRCA1 A314T` is the same
   shape (`chr17:g.41246608C>T` `NM_007300.3:c.940G>A p.Ala314Thr` loses to
   ClinVar 55588 `p.Ala1823Thr`); `BRCA1 C61G` hit the refusal side of the
   same hole (both alias hits carry ClinVar records, so 1297 refused even
   though exactly one hit, `chr17:g.41258504A>C` `NM_007294.4:c.181T>G
   p.Cys61Gly`, names the change on its canonical transcript).

## Changes

- `src/transform/variant.rs` exposes `canonical_protein_change(hit)`: the
  protein change on the transcript BioMCP headlines (the ClinVar-named or
  canonical SnpEff annotation, the same selection that builds `hgvs_p`).
- `src/entities/variant/get.rs`: a hit counts as a match only when its
  canonical protein change spells the requested change
  (`hit_names_requested_change`). Resolution order for several provider
  hits: a single named match resolves; among several named matches the
  1297 ClinVar tiebreak applies (exactly one ClinVar record resolves,
  otherwise refusal); zero named matches refuses with a new reason
  ("none of them names that change on its canonical (or ClinVar-named)
  transcript") listing every candidate. A unique provider hit still
  resolves, and the no-hits search suggestion is unchanged. The EGFR
  M766I refusal message is byte-identical (three named matches, no ClinVar
  record).
- Unit tests (`src/entities/variant/get/protein_change_tests.rs`) rebuild
  the 1297 DICER1/EGFR shapes with their recorded SnpEff annotations so the
  rule sees what the recorded data carries, and pin the new rows: TP53
  C124Y, BRCA1 A314T (named change beats the ClinVar lookalike), BRCA1 C61G
  (canonical spelling resolves over conflicting records), and a
  lookalike-only refusal that ClinVar cannot promote.
- Spec `spec/entity/variant-protein-change-resolution.md` adds the three
  rows from recorded responses served by the variant-identity fixture
  (only recorded queries answer; anything else 400s), and the prose states
  the named-match rule. The 1297 rows (BRAF V600E, DICER1, EGFR refusal)
  stay.
- `docs/user-guide/variant.md` states the rule in one sentence.
- Captures `query_tp53_c124y_20261007.json`,
  `query_brca1_a314t_20261007.json`, `query_brca1_c61g_20261007.json`
  recorded byte-faithful and receipted in `capture-receipts.json`
  (zero-coupling digest repinned; source-size inventory baselines for
  `get.rs` and `transform/variant.rs` extended with ticket 2016).

## Proof

- Root-cause reproduction on `37631c357`'s rule: the recorded TP53/BRCA1
  responses make the old rule return the lookalike; each new unit test
  fails against the pre-change resolver shape.
- Rust: protein-change tests 17 passed; full variant+myvariant scope 409
  passed; `make lint` green (fmt, clippy `-D warnings`, cargo deny,
  zero-coupling, quality ratchet, test-wait ratchet); pre-commit clippy
  `--lib --tests` green.
- Spec: `variant-protein-change-resolution.md` standalone 8 passed; with
  `variant-input-forms.md` + `variant-gene-first-routing.md` 28 passed
  under the shared variant-identity fixture. DICER1 resolves to
  `chr14:g.95562808C>T` and the EGFR refusal keeps its message.
- Receipts audit green (318 files, three new captures receipted).
- Live smoke on 2026-10-07 through the fixed resolver logic is covered by
  the recorded fixtures; no further live calls were needed.

## Deferred gaps

- A full transcript-aware resolver (all isoform numbering mapped through
  MANE/RefSeq) stays deferred to the 1.0 shared HGVS parser, as 1297
  deferred it.
- A hit with neither a ClinVar-named nor a SnpEff transcript annotation
  cannot prove a named match; among several such hits BioMCP now refuses
  where 1297 could resolve via a bare ClinVar variant_id. No recorded case
  exercises that shape.
- A CHANGELOG bullet is owed before the 0.9.2 tag (ticket 1305 owns the
  bullets; 2016's bullet belongs there).
- `variant.md`'s "Finite score thresholds" rows timed out twice on this
  machine while sibling lanes drove load past 80; the same commands exit 2
  in milliseconds when run by hand, and the rows are unrelated to this
  change (input validation before any provider call). Rerun that page on a
  quiet machine before tagging.
