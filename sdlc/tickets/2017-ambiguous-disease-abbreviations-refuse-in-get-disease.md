# 2017 — Ambiguous disease abbreviations refuse in get disease

Status: OPEN (second-review fix round built; fresh review of the head
follows). Build record:
`sdlc/records/2017-ambiguous-disease-abbreviations-build.md`.

Milestone: 0.9.2

## Outcome

`biomcp get disease <abbreviation>` returns one coherent disease card when one disease holds the abbreviation. When several do, it refuses and lists the candidates. A card never mixes one disease's definition with another disease's genes, and search ranking for common abbreviations does not get worse than 0.8.25.

## Evidence

Filed 2026-10-07 from the review of the work since v0.9.1 (`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`, finding 3). The reviewer ran a release build of main at `37631c357` against MyDisease.info.

- `src/sources/mydisease.rs:227`: the 1295 query now matches exact synonyms, which brings several diseases that share an abbreviation into one result set. `get disease` still picks one silently, and the tie-break favours records with no label because the shorter fallback name wins.
- `get disease MF` shows MONDO:0020481, "Disease label unavailable", the Myotonia fluctuans definition, and genes CCR4, NR3C1, DHFR and TNFRSF8, which belong to mycosis fungoides.
- `get disease CAD` shows the cold agglutinin disease definition with APOE, LDLR and PCSK9.
- `get disease MM` returns Miyoshi muscular dystrophy.
- With the 1295 query line reverted, all three return "not found".
- `search disease MDS` now ranks Miller-Dieker lissencephaly above myelodysplastic syndrome. 0.8.25 had the opposite order.
- The 1295 record says exact resolution refuses when more than one disease holds the token. That is false for `get disease`.

- Starts from: ticket 1295 and `sdlc/records/1295-disease-abbreviations-build.md`.
- Keeps: abbreviations held by exactly one disease resolve.
- Changes: `get disease` refuses with candidates when more than one record holds the abbreviation; sections assemble from the one resolved record only; labelled records outrank unlabelled ones; abbreviation hits do not outrank name hits in search.
- Proof: outside-in tests for MF, CAD, MM and MDS from recorded responses, each failing on `37631c357`, plus one single-holder abbreviation that still resolves.
- Defers: curated abbreviation preferences.

## Root cause

Reproduced live on 2026-10-07 with a main-tree debug build at `bc8b1808b`
against MyDisease.info; all four review cases confirmed.

1. **Resolution picks one hit silently and the tie-break favours unlabelled
   records.** The 1295 query now returns every disease that holds a token in
   `disease_ontology.synonyms.exact` or `mondo.synonym.exact`.
   `resolve_disease_hit_by_name` scores name and synonym labels equally
   (an exact abbreviation synonym scores 320, the same as an exact name),
   then breaks ties by shortest normalized canonical name
   (`scored_best_candidate_for_queries`, `src/entities/disease/resolution.rs`).
   A hit with no label in the search response falls back to its ID
   (`name_from_mydisease_hit`), and `"mondo 0020481"` is shorter than
   `"mycosis fungoides"`, so the unlabelled record wins: `MF` resolves to
   MONDO:0020481 (Myotonia fluctuans), `CAD` to MONDO:0018922 (cold
   agglutinin disease). Nothing counts exact holders, so one of several
   holders is returned silently.
2. **The card re-resolves the requested text per section.** The identity of
   the card comes from one MyDisease hit, but the gene section queries
   Open Targets with `disease.name` plus synonyms
   (`add_genes_section`, `src/entities/disease/associations.rs`). When the
   resolved record carries no label, `get_with_context` has already
   overwritten `disease.name` with the requested abbreviation, so Open
   Targets resolves `MF` on its own ontology (mycosis fungoides) and its
   genes attach to the Myotonia fluctuans card; `CAD` gets coronary artery
   disease genes the same way. Definition and genes therefore come from
   different diseases.
3. **`MM` resolves outright to Miyoshi muscular dystrophy.** MyDisease holds
   `MM` as an exact synonym of exactly one record (MONDO:0009685), so rule 1
   resolves it, but a clinician's `MM` is multiple myeloma, which holds no
   `MM` synonym in the source. A single holder is not evidence the
   abbreviation names that disease.
4. **Search breaks exact-abbreviation ties by provider order**
   (`rerank_disease_search_hits`): score, then display-name rank, then first
   seen upstream. For `MDS` both holders tie at score 320, rank 0, and
   MyDisease returns Miller-Dieker lissencephaly (score 10.81) above
   myelodysplastic syndrome (10.03), so Miller-Dieker ranks first; unlabelled
   abbreviation holders also surface above labelled ones.

## Success criteria

1. `get disease MF` and `get disease CAD` refuse with `invalid_argument`,
   naming every exact holder (MF: MONDO:0020481 and mycosis fungoides;
   CAD: all three holders), and never build a card that mixes one disease's
   definition with another's genes.
2. `get disease MDS` refuses the same way (Miller-Dieker lissencephaly
   syndrome and myelodysplastic syndrome are both exact holders), and
   `search disease MDS` ranks myelodysplastic syndrome first.
3. `get disease MM` refuses: one source holder (Miyoshi muscular
   dystrophy) is not enough for a two-letter abbreviation to name a disease;
   the refusal names the holder. This also refuses two-letter tokens such as
   `HD` (Huntington's disease in the source; Hodgkin disease in oncology
   use), which is the honest cost of the rule.
4. A single-holder abbreviation of three or more letters still resolves
   (pinned on CRC), and the card's sections then come from that one record
   only: when the resolved record has no label, the requested abbreviation
   stays display-only and identity joins (genes, CIViC) use the record's own
   terms or none.
5. Labelled records outrank unlabelled ones in both `get` resolution and
   `search` ranking; the 1295 spec table (NSCLC, DLBCL, CRC, AML, CAD
   set-equality, HGSC miss, and the four full-name rows) stays green, and
   the 1295 recall win survives because search still surfaces every exact
   holder.

## Build result (2026-10-07)

All five criteria met. `get disease MF/CAD/MM/MDS` refuse with named
candidates (verified live and from recorded responses), `search disease
MDS` ranks myelodysplastic syndrome first, `search disease CAD` leads with
the labelled coronary artery disease, and `CRC/AML/NSCLC/DLBCL` still
resolve to one coherent record. Two-letter tokens refuse generally, so `HD`
now refuses too (named in the refusal); that is criterion 3's honest cost.
The 1295 spec table and CAD set-equality stay green. Deferred stays
curated abbreviation preferences.

## Build status

- Built on branch `tickets/2017-ambiguous-disease-abbreviations-refuse-in-get-disease`,
  commits 99bdea36a, 5bb4a1cae, 2026-10-07, after one timeout revival
  with a checkpoint (nothing lost).
- Code review: ACCEPT 2026-10-07. Verified: the holder-count rule with
  the 1297-pattern refusal and the short-token gate; the labelled-first
  tie-breaks closing the unlabelled-ID path; card identity closed both
  ways (substitution after enrichment; abbreviation-shaped synonyms
  never reach Open Targets; the synthetic alias test pins both halves);
  all four regressions genuinely failing on pristine base; 1295's
  recall table and CAD set-equality unchanged; the HD cost honestly
  disclosed; receipts and the digest repin clean. Three report-only
  notes: holder counting sees the provider's top-15 window
  (pre-existing), a pre-existing '/' synonym split in associations,
  and the resolution-layer tie-break carried by outside-in pins
  rather than unit pins.

D
## Fix-round review

- Code re-review (rebased head 5b2ee79c2): ACCEPT 2026-10-07. All
  four finding-14 items verified at code level with hand-checked
  holder sets; the rebase unions correct (ticket grammar, 325 receipts
  with no re-indent, digest repinned); the BIOMCP_BIN spec fix closes
  the stale-binary hole; the incident disclosure judged with no
  surviving contamination path for the builder's verification. Minor
  notes recorded: full words with zero exact-name holders resolve
  through scoring (intended), the top-15 window stays pre-existing,
  lowercase two-letter wording odd but accurate.

## Second-review fix round (2026-10-07)

Finding 14 of `sdlc/issues/2026-10-07-second-review-of-the-work-since-0.9.1.md`.

- Non-human records no longer count as holders: MONDO records descended
  from MONDO:0005583 (`non-human animal disease`) are excluded, so the
  venom-database myeloma record no longer refuses `get disease myeloma`.
- A full word is ambiguous only by that name: for a token that is not
  abbreviation-shaped, only exact-NAME holders count toward the refusal,
  so `get disease myeloma` resolves to multiple myeloma (MONDO:0009693)
  while abbreviation holders (MF, CAD, MM, MDS) keep refusing.
- Refusal candidate lists fill missing labels from the MONDO `label`:
  `CAD` now names `cold agglutinin disease (MONDO:0018922)` and
  `congenital alveolar dysplasia (MONDO:0100077)` instead of printing
  bare IDs.
- The NCI trial search condition degrade now reaches the response as a
  page note (`_meta.notes` and the markdown footer, the 2021 pattern)
  whenever grounding falls back to a plain keyword search, instead of a
  log line only.
- Proof: the myeloma resolve and the labelled CAD refusal replay through
  recorded responses (`query_myeloma.json`, `get_mondo_0009693.json`,
  re-recorded `query_cad.json`/`query_mf.json` with the widened search
  fields), holder-rule unit tests pin the non-human exclusion and the
  full-word rule, and an NCI fixture test pins the degrade note on the
  page. The five-row spec table gains the myeloma row.

## Landing

Landed 2026-10-08, merge d8afa297f. Evidence: branch CI green at
84a647904; yellow gates fully green at the same sha (lint, spec, and
test all rc=0 in the isolated gate clone); code review ACCEPT at
5b2ee79c2 plus delta ACCEPT through 84a647904. Process deviation,
recorded honestly: the merge reached main through the coordinator's
local-chain push before the landcheck run finished — the landcheck at
d8afa297f completed green afterward, and main's own CI on the landing
push covers the same tree.
