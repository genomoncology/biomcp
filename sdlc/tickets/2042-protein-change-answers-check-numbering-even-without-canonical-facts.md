# 2042 — Protein-change answers check numbering even without canonical facts

Status: OPEN.

Milestone: 0.9.2

## Outcome

`biomcp get variant "GENE change"` never returns a change on a different numbering with no note, including when the requested position lies past the end of the canonical protein, when UniProt cannot be reached, when UniProt names no MANE transcript, or when the variant record carries many SnpEff rows. In each case it returns the MANE change, an honest refusal, or a note that names the other numbering or says the numbering could not be checked.

## Evidence

Filed 2026-10-09 from the independent review of ticket 2036 as landed on main at `ec8a72d0d`. 2036 holds on all 22 named probes and 45 of 47 fresh samples. Three paths still answer on the wrong numbering with no note.

- **Past the end of the protein.** `BRCA1 Y1866D` returns `p.Tyr1845Asp` and `BRCA1 H1883D` returns `p.His1862Asp`, both on NM_007294.4 with no note. The canonical BRCA1 protein (P38398) has 1863 residues. `src/entities/variant/get.rs:474` reads `let Some(residue) = facts.residue else { return Ok(None) }`, so a position the canonical protein lacks counts as unknown instead of as proof of another numbering.
- **No canonical facts.** With UniProt unreachable, `TP53 S183Y` returns `p.Ser183Tyr` on NM_001126115.1 with no note, and `BRCA1 S1587F` and `K918T` come back on NM_007300.3. This is the bug 2036 was filed for. Removing the MANE-Select reader gives the same silent answer, because the guard at `get.rs:488` returns no note when no MANE transcript is known. `get.rs:997-1009` and `445-456` fall back to the old behaviour whenever the canonical facts are missing.
- **Many SnpEff rows.** `src/sources/myvariant.rs:894` and `1160-1180` discard the whole SnpEff list when it holds more than 32 entries. BRCA1's BRCT region carries about 48 protein-structure rows per variant. `BRCA1 Q1878R`, `Q1857R` and the genomic ID itself return a headline with no protein change, no transcript and no note. v0.9.1 does the same.
- **The wiring test is missing.** Skipping the up-front UniProt check at `get.rs:997` passes all 63 unit tests; only the spec page fails (11 rows).
- **Latency.** ClinVar-free requests went from about 0.35 seconds to about 1.2 seconds uncached (`BRCA1 K918T`: 0.38 seconds on `e71ac046b`, 1.19 seconds on main). The UniProt record is not cached, so cached runs still take 0.55 to 1.1 seconds.
- **Refusal text.** The refusal never names the isoform that spells the request; for `BRCA1 L52M` that is the shorter isoform. In text mode its line breaks collapse, so it reads "Candidates: - chr17…".

- Starts from: ticket 2036 and its landing `39f9dc900`.
- Keeps: every 2036 answer, refusal and note recorded in its review; the honest TP53 R209Q, G112D and R174H refusals.
- Changes: give the other-numbering note when the requested position lies beyond the canonical length; when there is no ClinVar marker and no canonical facts and the headline is not on MANE, refuse or say the numbering could not be checked against MANE; skip SnpEff `feature_type: interaction` rows, or keep the transcript rows, instead of dropping the whole list; refuse a protein-change request whose headline carries no protein change; cache the gene-to-accession lookup and the UniProt record; name the isoform in the refusal and keep its line breaks.
- Proof: recorded-reply tests for BRCA1 Y1866D, TP53 S183Y with UniProt failing, and BRCA1 Q1878R, each failing on `ec8a72d0d`; a unit test through `resolve_base_with_hit` with a stubbed UniProt record that fails when the up-front check is skipped; timings for five ClinVar-free requests, cached and uncached.
- Defers: nothing.
