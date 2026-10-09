# 2040 — refuse trial search without a limit on the default source

Status: OPEN.

Milestone: 0.9.2

## Outcome

`search trial -c ABBR` on the default ClinicalTrials.gov source either refuses the ambiguous abbreviation the way the NCI source does, or says plainly that it ran a keyword search on an ambiguous short form. The user is never left to guess.

## Evidence

Filed 2026-10-10 from ticket 2038 (the pre-tag review), carrying 2032's recorded gaps: `search trial -c MF` on the default source runs a raw search with no note (of 50 results, 22 are myelofibrosis and 14 are mycosis fungoides trials); the MF refusal offers mycosis fungoides and myotonia fluctuans but omits myelofibrosis; the MM pointer comes from a one-entry table in `src/entities/disease/resolution.rs` that cites no source; NCI conditions over 512 bytes fail outright (`src/entities/trial/search/nci.rs`).

- Starts from: the 2032 landing (merge d18d30e7f).
- Keeps: the NCI refusal with choices.
- Changes: extend the refusal or the note to the default source; add myelofibrosis to the MF choices with a source; cite the MM pointer table's source; soften or explain the 512-byte NCI limit.
- Proof: recorded tests for the default-source note and the MF choices; a source citation for the pointer table.
- Defers: nothing.
