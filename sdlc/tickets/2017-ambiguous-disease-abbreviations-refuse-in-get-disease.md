# 2017 — Ambiguous disease abbreviations refuse in get disease

Status: OPEN.

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
