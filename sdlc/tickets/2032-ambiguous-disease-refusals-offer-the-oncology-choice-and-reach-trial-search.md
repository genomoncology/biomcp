# 2032 — Ambiguous disease refusals offer the oncology choice and reach trial search

Status: OPEN.

Milestone: 0.9.2

## Outcome

When `get disease` refuses an ambiguous abbreviation, the choices include the common oncology meaning. `search trial -c` refuses the same input the same way instead of falling back to a keyword search.

## Evidence

Filed 2026-10-08 from the third review of the work since v0.9.1 (ticket 2033, finding 17). Follows ticket 2017, landed in `d8afa297f`.

- Live on main: `get disease MM` refuses and offers only Miyoshi muscular dystrophy. A clinician who means multiple myeloma gets no myeloma choice.
- `src/entities/trial/search/nci.rs`: `-c MF` gets the refusal back as an error, then falls back to a plain keyword search for "MF" with the refusal text in a note.

- Starts from: ticket 2017's refusal rule and its candidate list.
- Keeps: 2017's resolutions for myeloma, NSCLC, CML and AML, and its refusals for MF, CAD and MDS.
- Changes: draw refusal candidates from the abbreviation's synonyms as well as exact labels, so MM lists multiple myeloma; make trial search return the refusal and its choices.
- Proof: `get disease MM` lists multiple myeloma (MONDO:0009693); `search trial -c MF` refuses with choices; tests pin both.
- Defers: nothing.

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
