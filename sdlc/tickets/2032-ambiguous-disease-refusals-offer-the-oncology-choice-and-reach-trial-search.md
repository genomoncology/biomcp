# 2032 — Ambiguous disease refusals offer the oncology choice and reach trial search

Status: complete.
Landed: d18d30e7f

Milestone: 0.9.2

## Outcome

When `get disease` refuses an ambiguous abbreviation, the choices include the common oncology meaning. `search trial -c` refuses the same input on the NCI source instead of falling back to a keyword search. The default ClinicalTrials.gov source still runs the keyword search; ticket 2040 carries widening the refusal to it.

## Evidence

Filed 2026-10-08 from the third review of the work since v0.9.1 (ticket 2033, finding 17). Follows ticket 2017, landed in `d8afa297f`.

- Live on main: `get disease MM` refuses and offers only Miyoshi muscular dystrophy. A clinician who means multiple myeloma gets no myeloma choice.
- `src/entities/trial/search/nci.rs`: `-c MF` gets the refusal back as an error, then falls back to a plain keyword search for "MF" with the refusal text in a note.

- Starts from: ticket 2017's refusal rule and its candidate list.
- Keeps: 2017's resolutions for myeloma, NSCLC, CML and AML, and its refusals for MF, CAD and MDS.
- Changes: name the oncology reading the source cannot see in the refusal's candidate surface, so MM points at multiple myeloma; make trial search return the refusal and its choices.
- Proof: `get disease MM` lists multiple myeloma (MONDO:0009693); `search trial -c MF` refuses with choices; tests pin both.
- Defers: refusing ambiguous conditions on the default ClinicalTrials.gov source (the NCI source refuses; the default source still keyword-searches).

## Build status addendum

- Code review: ACCEPT 2026-10-08. All six points verified at the code
  level: the one-entry clinical-reading table renders a pointer line
  that never joins the candidates; the NCI InvalidArgument arm
  propagates the ambiguity refusal with zero requests while NotFound
  keeps the visible keyword degrade; the tests pin both plus the kept
  2017/2021 pins; the lint opt-out matches the sibling sections; the
  lint-fix commit story is consistent; nothing else in the branch. CI
  run 37798569759 green at the code tip 6815a33b7; the record-only tip
  74414629c rides the docs lane.

## Root cause

Verified live against MyDisease.info on 2026-10-08.

1. The refusal's candidate list draws only from the source's exact holders
   (`exact_token_holder_ids`, `src/entities/disease/resolution.rs`). MyDisease
   holds `MM` on exactly one record, Miyoshi muscular dystrophy
   (MONDO:0009685). No myeloma record carries `MM` in any indexed field: the
   scoped query `mondo.synonym.exact:"MM" OR disease_ontology.synonyms.exact:"MM"`
   returns one hit (Miyoshi), and the MONDO:0009693 record itself holds no
   `MM` token. The honest source surface therefore cannot offer the oncology
   reading, so a clinician whose `MM` is multiple myeloma gets no myeloma
   choice. Ticket 2017's root cause 3 named exactly this and deferred
   curated preferences.
2. `search trial -c MF --source nci` grounds the condition through the same
   `resolve_disease_hit_by_name` resolver, but
   `resolve_nci_disease_filter_with_client`
   (`src/entities/trial/search/nci.rs`) catches every error except
   `NotFound` and degrades to `NciDiseaseFilter::Keyword("MF")`, with the
   refusal text demoted to a degrade note. A keyword search for the raw token
   then runs anyway — the mixing hazard 2017's refusal exists to stop,
   executed behind a note.

## Success criteria

1. `get disease MM` still refuses under 2017's short-token rule and still
   names Miyoshi muscular dystrophy (MONDO:0009685) as the one source holder;
   the refusal adds one pointer line naming the common oncology reading,
   multiple myeloma (MONDO:0009693), with a working
   `biomcp get disease "multiple myeloma"` form. The pointer is message-only:
   `MM` never resolves to myeloma, and curated abbreviation preferences stay
   deferred.
2. The MF, CAD and MDS refusals keep their candidate lists and candidate
   counts; the CAD message pin and the abbreviation spec rows stay green.
3. `search trial -c MF --source nci` returns the same invalid-argument
   refusal `get disease MF` returns, with both named candidates, and sends no
   NCI request; the refusal is not demoted to a keyword degrade note.
4. A condition that cannot ground still degrades to the NCI keyword search
   with the visible partial note (the 2021 pattern); the existing degrade
   tests and the recorded request log stay green.

## Build status

- Built on branch `tickets/2032-disease-refusal-choices`, commits
  ce648b2b9, 3a17d6421, 33800d255, 6815a33b7, 2026-10-08.
- Verified on the build host at 6815a33b7: `make lint` green; `make spec`
  green with the two changed pages passing (disease.md 18 assertions,
  trial.md 40); the disease and trial-search nextest scopes pass 202
  tests, including the new MM pointer pin, the NCI refusal pin, and the
  kept keyword-degrade pin. The first lint run failed the spec lint on
  the new trial.md section (a run-and-expect section with no mustmatch
  assertion); it now carries the same lint opt-out the other
  run-and-expect sections use.

## Fourth-review records (findings 18 and 19)

- Scope change recorded: the approved Changes line said to draw refusal
  candidates from the abbreviation's synonyms; the delivered fix uses a
  pointer line instead, because no source query can surface a reading
  the source does not hold (verified live before coding). The review
  covering the delivered shape is recorded above.
- Outcome narrowing recorded: the NCI refusal covers the --source nci
  path; the default ClinicalTrials.gov source still runs a raw keyword
  search for an ambiguous condition (22 of 50 MF results are
  myelofibrosis trials). Widening to the default source is deferred to
  a follow-up; the Defers line is amended accordingly.

## Follow-ups filed (2026-10-10, ticket 2038 item 6)

- Ticket 2039 carries the product names shared by unrelated drugs
  (2031's deferral).
- Ticket 2040 carries the default-source trial refusal plus the
  recorded gaps: the MF refusal omits myelofibrosis, the MM pointer
  table cites no source, and NCI conditions over 512 bytes fail
  outright.
