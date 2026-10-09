# 2016 — Protein-change resolution keeps the named variant

Status: complete.

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

## Fix-round-3 review

- Code re-review (head 86aadbc88, the false-note fix closing the third
  review's blocker 3): ACCEPT 2026-10-08. Verified: the residue rule
  (request parse, UniProt canonical sequence, MANE-stem guard
  protecting I1568N); the three false-note cases refuse with full
  messages against their captures; the honest notes survive with
  graceful degradation; the fixture wiring across the routine lane; the
  inventory re-measures; the 342-entry receipts union with the digest
  repin; the ticket grammar. No findings. Minor notes recorded: miss-
  path latency (two sequential lookups), the record describing round 1
  only, an em-dash cosmetic.

## Behavior note (review fold, 2026-10-07)

Among several hits where none can prove it names the requested change
(no SnpEff canonical and no ClinVar-named annotation), BioMCP now
refuses with candidates where the 1297 rule could resolve through a
bare ClinVar variant id — the exact wrong-answer channel this ticket
closes. Unique-hit queries are unaffected. No recorded case exercises
the shape; the narrowing is strictly conservative.

## Fix round (2026-10-07, finding 13)

Second-review finding 13: `src/transform/variant.rs` took the first NM_
transcript in the SnpEff list as canonical, which for BRCA1 is NM_007300,
21 residues higher than MANE Select NM_007294 past the isoform insert; and a
resolved hit whose protein change does not spell the request answered with
no word about numbering (`TP53 R116Q` resolving to `p.Arg248Gln` silently).

MyVariant.info's SnpEff and dbNSFP sections carry no MANE status (checked
against the recorded captures and the live provider with `fields=all`), so
the MANE marker in this data is the transcript ClinVar's preferred names
use: ClinVar names a variant on the gene's MANE Select transcript when one
exists. The headline selection now prefers, in order: the hit's own
ClinVar-preferred transcript; the query response's agreed ClinVar-preferred
stem (so a ClinVar-less hit in a multi-hit response headlines the same MANE
transcript — the recorded A314T answer moves from NM_007300.3 to
NM_007294.3); then the first NM_ (yesterday's rule) when the response marks
no stem. When the resolved change does not spell the request and the request
named residues, the answer carries one line, `protein_numbering_note`,
naming the transcript and spelling that matched.

New recorded captures (production query shape): `query_tp53_r116q_20261007.json`
(ClinVar 12356, headline `p.Arg248Gln` on NM_000546.5, note),
`query_brca1_a1844t_20261007.json` (the long isoform's spelling past the
insert; the answer pins to MANE `p.Ala1823Thr` on NM_007294.3, note), and
`query_brca1_i1568n_20261007.json` (ClinVar-less; no marker in the response,
first-NM_ fallback `p.Ile1589Asn` on NM_007300.3, note). The spec page adds
a numbering table on these rows, and unit tests cover the cohort rule past
the insert (composed from the recorded I1568N and ClinVar 55588 hits) and
the marker-less fallback.

## Fix round 3 (2026-10-08, ticket 2033 finding 3)

The numbering note never checked the requested reference residue against the
MANE sequence, so it could be false: when the MANE-numbered change has no
MyVariant record, the alias search still returns an isoform lookalike, and
the note claimed the request followed another transcript's numbering.
`TP53 R209Q` (UniProt P04637 has Arg at 209) resolved to `p.Arg248Gln`,
`TP53 G112D` (Gly at 112) to `p.Gly244Asp`, and `TP53 R174H` (Arg at 174) to
`p.Arg333His`; a live scan found 40 such TP53 positions.

Fix: before the note prints, BioMCP checks the requested reference residue
against the gene's canonical (MANE Select) protein sequence from UniProt
(`uniprot_reference_residue` through the existing MyGene symbol resolution).
When the residue matches and no annotation on the MANE transcript spells the
request (`mane_annotation_names_change`), the request's own numbering is
valid on MANE and the only alias match names a different change, so `get
variant` refuses with the lookalike as the candidate instead of printing the
other-transcript note. The note still prints when the residue differs
(`TP53 R116Q`: residue 116 is Ser), when the response marks no MANE
transcript, when the hit names the request on the MANE transcript under a
fallback headline (`BRCA1 I1568N`), and when the sequence lookup is
unavailable (an unavailable sequence proves nothing either way).

New recorded captures: `query_tp53_r209q_20261008.json`,
`query_tp53_g112d_20261008.json`, `query_tp53_r174h_20261008.json`, and the
minimized `uniprot/get_p04637_20261008.json` (accession and sequence; the
original response's SHA-256 is in the receipt). The variant-identity fixture
serves the queries and the UniProt record, and the complexportal fixture —
which owns the routine lane's UniProt base — serves the same record bytes.
The spec page pins the three refusals plus `TP53 R116Q`'s honest note.

## Fix round 4 (2026-10-09, ticket 2035 findings 4 and 21)

The MANE marker depended on a ClinVar hit, so a response without ClinVar
records fell back to the first NM_ and the refusal returned early before the
UniProt check: `get variant "BRCA1 I1568N"` answered `p.Ile1589Asn` on
`NM_007300.3` with a note claiming I1568N follows another transcript's
numbering, while UniProt P38398 has Ile at 1568 and MyVariant's NM_007294
entry reads p.Ile1568Asn — the note was false and the headline renumbered.
The recorded captures carry no MANE status outside ClinVar (checked against
`query_brca1_i1568n_20261007.json` and, in round 1, the live provider with
`fields=all`), so the ClinVar-free signal is the annotation that names the
request: the SnpEff ranking now headlines the annotation spelling the
requested change whenever the response marks no ClinVar-preferred stem
(tier order: the hit's own ClinVar-preferred transcript, the response's MANE
marker, the annotation naming the request, the first NM_, anything else), so
I1568N resolves honestly on NM_007294's `p.Ile1568Asn` with no note, and the
ClinVar marker still outranks the naming annotation — `TP53 R209Q` keeps
refusing instead of resolving through NM_001126118's `p.Arg209Gln`.

The residue check now runs before any note prints: the refusal no longer
returns early without a marker, and the note prints only when the check
proved the canonical protein carries a different residue at the requested
position (`TP53 R116Q`). When the request names no reference residue, or the
UniProt lookup fails, nothing is proven either way and the answer stays
silent rather than carry an unverified other-transcript claim — round 3's
"the note still prints when the response marks no MANE transcript / when the
lookup is unavailable" is replaced by this rule.

Finding 21's smaller items: the MANE marker now carries the versioned
accession ClinVar's preferred names use, and the headline shows that current
RefSeq accession where the SnpEff build lags it (NM_000546.6 against .5,
NM_007294.4 against .3, NM_004333.6 against BRAF's .4), so one gene's
answers stop mixing transcript versions; the MANE-numbering refusal's retry
line names the requested change (`a transcript-qualified HGVS naming
'R209Q'`, or `biomcp search variant -g TP53 --hgvsp R209Q`) instead of
pointing at the candidate already known to be a different change; and the
variant-identity fixture's synthetic TP53 R273H hold now carries invented
identifiers (chr17:g.7676000G>A, VariationID 6000001, RCV006000001) instead
of real identifiers borrowed from VLDLR, FANCB and TP53 P72L.

New recorded capture: the minimized `uniprot/get_p38398_20261009.json`
(accession and sequence; the original response's SHA-256 is in the receipt),
served by the variant-identity and complexportal fixtures so BRCA1's residue
checks run in the spec lane. The spec page's numbering table repins
`BRCA1 I1568N` to the honest resolution (p.Ile1568Asn on NM_007294.3, no
note) and the unified transcript versions, and the R209Q refusal block pins
the new retry line. Unit tests pin the ClinVar-free naming tier (single-hit
and two-hit cohorts), the marker outranking the naming annotation, the
version upgrade on a recorded BRAF V600E shape, and the refusal retry line.
