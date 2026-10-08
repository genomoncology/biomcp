# 2016 — Protein-change resolution keeps the named variant

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get variant '<GENE> <protein change>'` returns the variant whose protein change in the standard numbering matches the query. The 1297 rule ("pick the one matching hit that has a ClinVar record") no longer overrides a correct first hit with a different variant.

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
