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

## Build outcome (2026-10-09)

Branch `tickets/2036-pre-tag` from main `203daec02`. The
`variant-protein-change-resolution` spec page gained the S183Y and S1587F
refusal rows (byte-exact messages), the S1551Y and S395T resolution rows,
and the I1568N headline moved to the MANE-Select current version
(NM_007294.4). Four new receipted MyVariant captures serve the rows
offline through the variant-identity fixture, and the two minimized
UniProt captures keep the record's MANE-Select cross-reference
(`mane_select_transcript`) with the provider response otherwise unchanged.

Red first: commit `adb7dec32` (captures, fixture and spec rows only, main
source) ran the spec page in an isolated worktree on the build host —
16 passed, 11 failed. The failures were exactly the probe rows: S183Y
and S1587F resolved the isoform spelling with no note (`chr17:g.7576902G>T`
p.Ser183Tyr on NM_001126115.1; `chr17:g.41223234G>A` p.Ser1587Phe on
NM_007300.3), S1551Y and S395T refused with two candidates, I1568N
headlined NM_007294.3, and the two refusal blocks exited 0. Every
pre-existing row passed on the same commit.

Green: commit `e3a47eeb1` (the fix) — the same page passes 27/27, and
the same 27 pass on the final head `6b0fb754`. The fix reads the gene's
canonical protein from UniProt up front whenever the response carries no
ClinVar MANE marker (none is fetched when it does), takes the
MANE-Select cross-reference as the MANE transcript with its current
version, passes that marker into `resolve_protein_change_hit` so the MANE
annotation headlines a hit ahead of the change-naming isoform, and runs
the refuse-or-note decision on every resolved hit. The decision split
into a pure function (`refuse_or_note_with_facts`) that the unit tests
pin offline: the two recorded refusals, the two MANE-named resolutions,
the R116Q true-note pin, and the MANE-Select reader (63 targeted tests
pass via nextest: `test(protein_change) + test(mane) + test(uniprot)`).

`make lint` passes on `6b0fb754` after two ratchet updates: the
capture-receipts digest the coupling allowlist carries, and the get.rs
source-size authorization (+95 lines, ticket 2036 appended).

Verification ran through `yr` in isolated git worktrees on the build
host: an earlier run against the shared checkout picked up another
lane's checkout mid-run (its page rows answered from a stale page and
binary), so the recorded runs above each start from a private worktree
at the pushed commit.

### Live sample (2026-10-09, fixed binary `6b0fb754`, live providers)

ClinVar-free gene+protein queries across the four genes; none resolves
an isoform-only spelling. TP53 refusals name the MANE spelling on
NM_000546.6; BRCA1 refusals name it on NM_007294.4.

| query | answer |
|---|---|
| TP53 S183Y | refuse: canonical P04637 has Ser at 183; only alias match p.Ser315Tyr on NM_000546.6 |
| TP53 P142Q | refuse: Pro at 142; only alias match p.Pro301Gln on NM_000546.6 |
| TP53 P190Q | refuse: Pro at 190; only alias match p.Pro322Gln on NM_000546.6 |
| TP53 P58H | refuse: Pro at 58; only alias match p.Pro190His on NM_000546.6 |
| TP53 T170N | refuse: Thr at 170; only alias match p.Thr329Asn on NM_000546.6 |
| TP53 P87H | refuse: Pro at 87; only alias match p.Pro219His on NM_000546.6 |
| BRCA1 S1587F | refuse: Ser at 1587; only alias match p.Ser1566Phe on NM_007294.4 |
| BRCA1 L1392R | refuse: Leu at 1392; only alias match p.Leu1439Arg on NM_007294.4 |
| BRCA1 S1450Y | refuse: Ser at 1450; only alias match p.Ser1497Tyr on NM_007294.4 |
| BRCA1 L52M | refuse: Leu at 52; only alias match p.Leu99Met on NM_007294.4 |
| BRCA1 S267C | refuse: Ser at 267; only alias match p.Ser1370Cys on NM_007294.4 |
| BRCA1 K918T | resolves chr17:g.41244795T>G p.Lys918Thr on NM_007294.4, no note |
| BRCA1 L1303S | resolves chr17:g.41243640A>G p.Leu1303Ser on NM_007294.4, no note |
| BRCA1 L1340W | resolves chr17:g.41243529A>C p.Leu1340Trp on NM_007294.4, no note |
| BRCA1 F228S | resolves chr17:g.41246865A>G p.Phe228Ser on NM_007294.4, no note |
| BRCA1 S1551Y | resolves chr17:g.41226371G>T p.Ser1551Tyr on NM_007294.4, no note (the candidate MANE names) |
| BRCA1 S395T | resolves chr17:g.41246365A>T p.Ser395Thr on NM_007294.4, no note (the candidate MANE names) |
| BRCA2 I2944V | resolves chr13:g.32953529A>G p.Ile2944Val on NM_000059.4 (current; MyVariant carried NM_000059.3), no note |
| ATM E1979D | refuses: two true matches, no ClinVar record names one |

Gaps: none deferred. The full `make test`/`make spec` lanes and CI run
with the coordinator's gate; the recorded runs above are the page, the
targeted nextest scopes, and `make lint`.
