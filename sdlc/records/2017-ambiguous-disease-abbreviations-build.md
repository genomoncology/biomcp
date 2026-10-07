# 2017 build: ambiguous disease abbreviations refuse in get disease

Ticket: sdlc/tickets/2017-ambiguous-disease-abbreviations-refuse-in-get-disease.md
Branch: tickets/2017-ambiguous-disease-abbreviations-refuse-in-get-disease
Built by a lane worker from the review finding 3 in
`sdlc/issues/2026-10-07-review-of-the-work-since-0.9.1.md`.

## Root cause

Written into the ticket before the fix. Two defects combined:

1. `resolve_disease_hit_by_name` scores name and synonym labels equally,
   picks one hit silently, and breaks ties by shortest normalized canonical
   name — where an unlabelled record's ID fallback ("mondo 0020481") beats
   real labels ("mycosis fungoides"). Nothing counted exact holders.
2. The card's gene section re-resolved the requested text in Open Targets:
   `get_with_context` overwrote an unlabelled record's name with the
   requested abbreviation before enrichment, and `add_genes_section` also
   offered the record's abbreviation-shaped synonyms as Open Targets
   queries. Definition came from one MyDisease hit; genes came from Open
   Targets' own resolution of the abbreviation (mycosis fungoides for MF,
   coronary artery disease for CAD).

Search broke exact-abbreviation ties by provider order, so `MDS` ranked
Miller-Dieker lissencephaly above myelodysplastic syndrome, and unlabelled
abbreviation holders surfaced above labelled ones.

## Changes

- `src/entities/disease/resolution.rs`: the direct name-resolution arm now
  counts candidates that hold the requested token as an exact name or exact
  synonym. Two or more holders refuse with `invalid_argument` naming every
  candidate (the 1297 pattern), with the search retry form. One holder plus
  a two-letter token also refuses: a token that short cannot name one
  disease reliably (`MM` names Miyoshi muscular dystrophy in the source
  while clinicians mean multiple myeloma). Both rankers (`get` resolution
  and `search` rerank) break ties labelled-record-first, then shorter
  canonical name, before provider order.
- `src/entities/disease/get.rs`: when the resolved record holds the
  requested term as a synonym, the requested-term label substitution
  happens after enrichment, so identity-join sections see the record's own
  terms (its ID) and never the abbreviation. Terms the record does not hold
  as synonyms keep the previous behavior (the caller's phrase remains the
  identity the record can offer).
- `src/entities/disease/associations.rs`: `add_genes_section` no longer
  offers abbreviation-shaped single-word synonyms as Open Targets queries;
  only multi-word, name-shaped synonyms join by identity after the record's
  own name and ID.
- Recorded MyDisease responses added under `testdata/sources/mydisease/`:
  `query_mf.json`, `query_mm.json`, `query_mds.json` (captured 2026-10-07,
  receipted in `testdata/sources/capture-receipts.json`, zero-coupling
  digest repinned) and `get_mondo_0024331.json` for the resolving CRC row.
  The routine disease fixture serves them.
- Spec page `spec/entity/disease.md` gains the "Ambiguous Abbreviations
  Refuse in Get Disease" section: a five-row table (MF, CAD, MM, MDS refuse
  with candidate counts; CRC resolves), the pinned CAD refusal JSON, and
  the MDS/CAD search-order contracts. The 1295 abbreviation table and CAD
  set-equality block are unchanged and stay green.

## Behavior changes

- `get disease MF|CAD|MDS` refuse with every exact holder named; `get
  disease MM` refuses naming Miyoshi muscular dystrophy. Two-letter tokens
  generally now refuse (`HD` names Huntington's disease in the source;
  Hodgkin disease is the oncology reading) — the honest cost recorded in
  the ticket.
- Single-holder abbreviations of three or more letters keep resolving
  (CRC, AML, NSCLC, DLBCL verified live), and the card assembles from that
  one record only.
- `search disease MDS` ranks myelodysplastic syndrome first; `search
  disease CAD` leads with the labelled coronary artery disease.

## Proof

- CLI-level Rust tests in `src/entities/disease/get/tests.rs` replay the
  recorded responses: MF, CAD, MM, MDS refusals name the right candidates
  (each fails on main `bc8b1808b`, which built mixed cards or returned
  Miyoshi), CRC resolves to one coherent record, and a synthetic
  single-holder unlabelled alias lookup proves the abbreviation never
  reaches Open Targets (one occurrence in the request log, the resolution
  query itself).
- Live checks with the built binary against MyDisease.info: the four cases
  refuse with the right candidate lists; `get disease CRC/AML/NSCLC/DLBCL/
  melanoma` resolve; `search disease MDS` and `CAD` order correctly; the
  1295 recall surface is unchanged because search still returns every exact
  holder.
- `make lint` green (clippy, quality ratchet, test-wait ratchet, fixture
  receipts, zero-coupling digest repinned).
- mustmatch on `spec/entity/disease.md` under the routine disease fixture
  (run after `make sync-python-dev`).

## Deferred gaps

- Curated abbreviation preferences (MM -> multiple myeloma, MDS ->
  myelodysplastic syndrome outright) stay deferred by the ticket; refusing
  is the honest behavior without them.
- `disease_ontology` sometimes arrives as an array of merged DO records;
  `provider_terms` reads only the object shape, so exact-holder detection
  relies on the mondo synonym lists for those hits (true for the recorded
  cases). Widening that reader is new-ticket territory.
- Two pre-existing test failures in this sandbox, identical on pristine
  main `bc8b1808b`: `a_stale_disease_card_tells_the_clinician_the_cache_age`
  and `a_stale_mcp_call_tells_the_clinician_the_cache_age_on_both_channels`
  (the stale-serve fallback does not survive this host's connection-refused
  behavior under 20-30 load average). Not caused by this change; recorded,
  not chased.
