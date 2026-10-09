# 2036 — Protein-change requests never resolve to an isoform-only match

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get variant "GENE change"` returns the requested change on the MANE transcript, an honest refusal, or a match whose note truthfully names the other numbering. It never returns a variant whose change matches the request only on a shorter or longer isoform, with no note. The headline shows the MANE transcript and its current version.

## Evidence

Filed 2026-10-09 from the pre-tag review of main at `e71ac046b` (ticket 2038, finding 2).

- Of 20 requests where UniProt confirms the requested residue on the canonical protein and no MyVariant record names that change on MANE, 11 resolved to an isoform-only match with no note.
- `TP53 S183Y` returns `chr17:g.7576902G>T` as `p.Ser183Tyr` on `NM_001126115.1`. MyVariant names that variant `NM_000546.5 p.Ser315Tyr`. UniProt P04637 has Ser at 183. P142Q, P190Q, P58H, T170N and P87H behave the same way.
- `BRCA1 S1587F` returns `NM_007300.3 p.Ser1587Phe`, which is MANE `p.Ser1566Phe`. L1392R, S1450Y and L52M come back on NM_007297, and S267C on NM_007298.
- The code already fetches the UniProt canonical sequence; that check makes `TP53 R209Q` refuse. It does not run when an isoform annotation already spells the request. 2016's fix-round-4 residual says no MANE source exists in this data, which the UniProt check contradicts.
- Answers with no ClinVar record show older transcript versions: NM_000546.5, NM_007294.3, NM_000059.3, NM_000051.3 and NM_004448.3 where MANE Select is .6, .4, .4, .4 and .4. ClinVar-free BRCA1 changes before residue about 1450 (K918T, L1303S, L1340W, F228S) headline `NM_007300.3`.
- `BRCA1 S1551Y` and `S395T` refuse although MANE names exactly one candidate, and the refusal does not say which candidate is on MANE.
- Replacing the honest note for `TP53 R116Q` with silence passes all 59 related unit tests.
- 0.8.25 gives the same wrong answer for `TP53 S183Y`, so this is a gap 2016 left open, not a new regression.

- Starts from: ticket 2016 and its fix round `e99ed6cfb`.
- Keeps: the ClinVar-backed MANE answers; the honest TP53 R209Q, G112D and R174H refusals; the true R116Q and BRCA1 A1844T notes.
- Changes: run the canonical-residue check before accepting any annotation that is not on the MANE transcript; prefer the MANE transcript and its current version in the headline; when MANE names exactly one candidate, return it or say which candidate is on MANE.
- Proof: tests for TP53 S183Y and BRCA1 S1587F that fail on `e71ac046b`; a unit test pins the R116Q note; a recorded live sample of ClinVar-free changes across BRCA1, BRCA2, TP53 and ATM shows no isoform-only answer.
- Defers: nothing.
