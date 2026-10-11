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

## Research (2026-10-11)

Recorded before coding, live against the sources.

- No indexed source record holds `MM` on a myeloma record or `MF` on a myelofibrosis record. MyDisease returns exactly two exact-synonym holders for `MF` (mycosis fungoides, MONDO:0009691; myotonia fluctuans, MONDO:0020481) and one for `MM` (Miyoshi muscular dystrophy, MONDO:0009685), via the scoped queries `mondo.synonym.exact:"MF"` and `disease_ontology.synonyms.exact:"MF"` (and the `MM` forms). The Disease Ontology entry for myelofibrosis (DOID:4971) lists no `MF`; its multiple myeloma entry (DOID:9538) lists no `MM`. The NCI Thesaurus synonym lists, read through EBI OLS4, carry no `MM` on Multiple Myeloma (NCIT:C3242), no `MF` on Myelofibrosis (NCIT:C3248), and no `MF` on Primary Myelofibrosis (NCIT:C2862). Orphanet and HPO hold neither token either. So the pointer table's readings cannot cite a synonym list that carries them; the citation records the negative checks and the corpus that does carry the readings.
- The corpus evidence is the ClinicalTrials.gov registry itself, read through the public API v2 on 2026-10-11. Of the first 50 studies matching `query.cond=MM`, 30 list a myeloma condition. Of the first 50 matching `query.cond=MF`, 28 list a myelofibrosis condition against 6 for mycosis fungoides (the ticket's filed counts, 22 and 14, were the 2026-10-10 reading; the corpus shifts daily). Myelofibrosis is therefore the reading the trial corpus carries most, and the pointer uses `MONDO:0009692` because that is the record `get disease "myelofibrosis"` resolves to (it holds the Disease Ontology name `myelofibrosis`); the umbrella `myelofibrosis` record MONDO:0044903 holds the token only as a synonym and never wins resolution.
- Red-on-main probes, run on the commit main pointed at when the work started (`7c1faf778`, remotely through `~/bin/yr`, 2026-10-11): `--json search trial -c MF --limit 1` on the default source returns `_meta` with next commands only and no note; `get disease MF` refuses naming mycosis fungoides and myotonia fluctuans with no myelofibrosis pointer; `--json search trial -c <600-byte condition> --source nci` fails outright with `Invalid argument: Query is too long.` (a 500-byte control reaches the NCI request, so the boundary is the MyDisease query limit in `query_plan`).

## Decisions

- The default source takes the note, not the refusal. The ticket allows either; the note keeps the registry's own keyword search (which the default source is for) while telling the user plainly what ran, which diseases hold the token, and the clinical reading. The refusal stays the NCI behavior, where the condition grounds to a concept. The note fires only for an abbreviation-shaped condition (one word, ASCII capitals), so ordinary conditions add no MyDisease request, and a resolver failure degrades to today's note-free search rather than failing.
- `--count-only` and the cross-entity pivot commands (`disease trials`, `gene trials`, `drug trials`, `pathway trials`) keep no note because their output shapes carry no note channel; the ticket's outcome names `search trial -c ABBR`, and the pivots share its search helper.
- The truncation note applies whether grounding succeeds on the truncated text or degrades to a keyword search, because either way the search ran on fewer bytes than the caller sent.

## Build status

- Built on branch `tickets/2040-work`.
