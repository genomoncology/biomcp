# 2016 — Protein-change resolution keeps the named variant

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get variant '<GENE> <protein change>'` returns the variant whose protein change in the standard numbering matches the query. The 1297 rule ("pick the one matching hit that has a ClinVar record") no longer overrides a correct first hit with a different variant.

## Root cause

Recorded 2026-10-07 against MyVariant.info with the production query shape; both suspected halves are real and compound.

1. The gene+protein arm matches hits through dbNSFP's merged `dbnsfp.hgvsp`
   alias list, which carries other isoforms' protein names on other genomic
   variants. `TP53 C124Y` queries `dbnsfp.hgvsp:"p.C124Y"`; MyVariant returns
   `chr17:g.7579316C>T` (canonical `NM_000546.5:c.371G>A p.Cys124Tyr`, no
   ClinVar record) and `chr17:g.7578526C>T` (canonical `c.404G>A
   p.Cys135Tyr`, ClinVar 141762), whose alias list includes `p.C124Y` through
   a shorter isoform. `compare_variant_identity` sees only the alias list, so
   both hits are "compatible": the isoform-alias match accepts a different
   protein's change.
2. `resolve_protein_change_hit` (`src/entities/variant/get.rs`) then applies
   the 1297 rule — exactly one hit carries a ClinVar record, so it wins — and
   never checks that the ClinVar record's own transcript names the requested
   change. The ClinVar-recorded lookalike outranks the exact named change.
   `BRCA1 A314T` is the same shape: `chr17:g.41246608C>T`
   (`NM_007300.3:c.940G>A p.Ala314Thr`, no ClinVar record) loses to
   `chr17:g.41199660C>T` (ClinVar 55588, `p.Ala1823Thr`).

`BRCA1 C61G` shows the same hole from the refusal side: both alias hits
(`chr17:g.41197805A>C`, canonical `p.Cys1828Gly`; `chr17:g.41258504A>C`,
canonical `NM_007294.4:c.181T>G p.Cys61Gly`) carry ClinVar records (409329,
17661), so the conflicting-records branch refuses even though exactly one
hit names `C61G` on its canonical transcript.

## Success criteria

1. A hit counts as a match only when the transcript BioMCP headlines for it —
   the ClinVar-named or canonical SnpEff annotation — spells the requested
   protein change. ClinVar presence breaks ties among those named matches
   only. A named match outranks a merely ClinVar-recorded lookalike.
2. `get variant 'TP53 C124Y'` returns `chr17:g.7579316C>T`
   (`p.Cys124Tyr`), not the ClinVar 141762 `p.Cys135Tyr` lookalike.
3. `get variant 'BRCA1 A314T'` returns `chr17:g.41246608C>T`
   (`p.Ala314Thr`), not ClinVar 55588 `p.Ala1823Thr`.
4. `get variant 'BRCA1 C61G'` returns `chr17:g.41258504A>C` (ClinVar 17661,
   `p.Cys61Gly`) — the canonical spelling resolves instead of refusing.
5. The 1297 behavior holds on its recorded cases: `DICER1 p.Met1483Ile`
   still resolves to `chr14:g.95562808C>T` (577152), and `EGFR M766I`
   still refuses with the same three candidates and message.
6. A unique provider hit still resolves; true ambiguity still refuses with
candidates and a working input form.

Each criterion 2 to 5 fails on `37631c357`.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 2). The reviewer ran a release build of main at `37631c357` against MyVariant.info and compared with 0.8.25.

- `src/entities/variant/get.rs:239` (`resolve_protein_change_hit`) picks the single matching hit that has a ClinVar record. It never checks that the ClinVar record names the protein change the user asked for. dbNSFP lists other-isoform protein names on other variants, so a hit can "match" by an isoform name.
- `biomcp get variant 'TP53 C124Y'` now returns `chr17:g.7578526C>T`, headline `# TP53 p.Cys135Tyr`, Pathogenic. 0.8.25 returned `chr17:g.7579316C>T p.C124Y`. MyVariant lists `chr17:g.7579316C>T` first, with no ClinVar record, then `chr17:g.7578526C>T` with ClinVar 141762.
- `biomcp get variant 'BRCA1 A314T'` now returns `chr17:g.41199660C>T` (p.Ala1823Thr, ClinVar 55588). It used to return `chr17:g.41246608C>T`.
- A scan found at least 8 TP53 and 10 BRCA1 queries with this shape before it stopped.
- `biomcp get variant 'BRCA1 C61G'` now refuses because two hits both carry ClinVar records (409329 and 17661). Refusing is safer than 0.8.25, which silently returned p.C319G, but the canonical-transcript answer is unambiguous.
- Ticket 1297 promised that queries which resolve unambiguously today keep their answer. The spec pins the new rule, so nothing catches the change.

- Starts from: ticket 1297 and its record `sdlc/records/1297-protein-change-refusal-build.md`; the DICER1 case it fixed.
- Keeps: the DICER1 answer (577152), the EGFR refusal with candidates, and refusing rather than guessing when the query is truly ambiguous.
- Changes: a hit counts as a match only when its protein change in the canonical (or ClinVar-named) transcript equals the query. ClinVar presence breaks ties among true matches only. Prefer the canonical transcript before refusing.
- Proof: outside-in tests for TP53 C124Y, BRCA1 A314T, BRCA1 C61G, DICER1 and EGFR, from recorded MyVariant responses. Each fails on `37631c357`.
- Defers: a full transcript-aware resolver.

## Build outcome (2026-10-07)

Branch `tickets/2016-protein-change-resolution-keeps-the-named-variant` from
main `bc8b1808b`. Record: `sdlc/records/2016-protein-change-resolution-keeps-the-named-variant.md`.
All six success criteria hold: the recorded TP53 C124Y, BRCA1 A314T, and
BRCA1 C61G responses resolve to the named/canonical variant in unit tests
and in the spec table; the DICER1 577152 answer and the byte-identical EGFR
M766I three-candidate refusal stay green; a unique provider hit still
resolves. Deferred: the 1.0 shared HGVS parser remains the full
transcript-aware resolver; a CHANGELOG bullet is owed via ticket 1305.

## Build status

- Built on branch `tickets/2016-protein-change-resolution-keeps-the-named-variant`,
  commits 1bf775733, 0663802e1, plus the review fold, 2026-10-07,
  after one timeout revival with a checkpoint (nothing lost).
- Code review: ACCEPT 2026-10-07. Verified: canonical_protein_change
  shares the headline transcript selection with hgvs_p; the named-match
  rule resolves TP53 C124Y, BRCA1 A314T, and BRCA1 C61G correctly while
  1297's DICER1 and byte-identical EGFR refusal hold; the tests
  genuinely encode the old rule's failures; the fixture serves only
  recorded queries; receipts and inventory exact; the unannotated-set
  refusal narrowing is safe, conservative, and now stated in the
  ticket. Deferred honestly: the MANE/RefSeq-aware resolver stays with
  the shared parser; variant.md reruns under the full fixture set
  before tagging.

## Fix-round review

- Code re-review (rebased head c172e04d7, covering the MANE fix and
  the whole landed shape): ACCEPT 2026-10-07. No findings. Verified:
  the MANE preference with the past-insert BRCA1 pin (A1844T on
  NM_007294 numbering) and the fallback pin with its note (I1568N);
  the numbering note end to end on the TP53 R116Q shape; the original
  named-match rule, 1297 tiebreak, and byte-identical EGFR refusal
  surviving the rebase; the branch ticket's review-line grammar
  passing the scanner's own rules; receipts and inventory exact.

## Repin review

- Code re-review (delta c172e04d7..d6801ed8b, the ratchet repin):
  ACCEPT 2026-10-07. All three entries verified against measured
  counts; variant/mod.rs confirmed as a genuinely new entry (main sits
  at exactly 1000, under the threshold; the fix's 5-line field pushed
  it over for the first time); no other changes in the commit
  (git show --stat confirms one file, the inventory). The lane head is
  d6801ed8b.

## Behavior note (review fold, 2026-10-07)

Among several hits where none can prove it names the requested change
(no SnpEff canonical and no ClinVar-named annotation), BioMCP now
refuses with candidates where the 1297 rule could resolve through a
bare ClinVar variant id — the exact wrong-answer channel this ticket
closes. Unique-hit queries are unaffected. No recorded case exercises
the shape; the narrowing is strictly conservative.
