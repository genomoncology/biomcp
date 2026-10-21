---
flow: build
priority: 6
---

# One trial reports one complete condition list

## Outcome

Search and detail conversion retain the same complete provider condition list.
JSON exposes that complete list. The bounded Markdown search cell says when it
is abridged and states the complete condition count.

## Current facts and reproducer

Four conversion call sites impose two different hidden caps in
`src/transform/trial.rs`:

- `from_ctgov_study` passes ClinicalTrials.gov conditions through
  `clean_list(..., 25)`, while `from_ctgov_hit` uses 10.
- `from_nci_trial` calls `nci_conditions(..., 25)`, while `from_nci_hit` uses
  10.
- `format_conditions` then takes at most ten values, joins them, and cuts the
  result at an 80-byte prefix. The shared helper appends its three-byte
  ellipsis after that cutoff, so the claimed 80-byte cell can actually be 83
  bytes. It provides neither the complete count nor a statement that the list
  was shortened.

The discrepancy is reproducible from this repository's receipted provider
bytes:

- `testdata/sources/ctgov/get_nct02576665_20260811.json` contains 12 ordered
  conditions. Passing its typed `CtGovStudy` through the detail converter
  retains 12; passing the same value through the search-hit converter retains
  10. The current Markdown formatter renders only the beginning of the fourth
  label (`Non-Small C…`) and gives no indication that the complete count is 12.
- `testdata/sources/nci_cts/search_melanoma_20260811.json` contains 26 disease
  objects for NCT05929768. Passing that record through the detail and search
  converters retains 25 and 10 names respectively.
- `testdata/sources/ctgov/search_keytruda_limit3_20260811.json` contains 21
  conditions for NCT03590054 and is already served by the routine CTGov spec
  fixture. The NCI capture above is already served by the provider-contract
  fixture, so both public search surfaces can be proved without new or altered
  provider bytes.

The receipted ClinicalTrials.gov field metadata at
`testdata/sources/ctgov/field_metadata_20260903.json` declares
`conditionsModule.conditions` as `text[]`; it does not define a ten- or
twenty-five-item provider limit. The ticket's conformance rule remains the
authoritative product requirement: conversion applies no cap, and presentation
may abbreviate only when the abbreviation is disclosed.

This is case 11 of the shared clinical-trial conformance cases and is held
against both 0.9 and 1.0. An implementation attempt cannot read the sibling
BioData repository, so the requirement above is this ticket's authoritative
copy. If it appears wrong, stop rather than implementing a different rule.

The NCI object parsing and unreadable-element error behavior currently in
`nci_conditions` is correct and must remain. That behavior was completed and
landed for ticket 1107 at commit `633affc2`; its completed record is
`sdlc/records/1107-an-nci-trial-reports-no-conditions-at-all.md`. This ticket
removes only the subsequent count cap.

## Required behavior

Conversion cleans conditions as it does today -- trim each label, discard
blank labels, and preserve provider order and duplicates -- but applies no
count cap. Given one provider value, the detail and search-hit converters
produce identical condition vectors for both ClinicalTrials.gov and NCI.

JSON search and detail output serialize those complete vectors. JSON does not
need a truncation flag because it is not truncated. This replaces the earlier
ambiguous requirement that a shortened list be marked "in JSON": adding a
marker to a complete JSON list would misdescribe the value and unnecessarily
change the public model.

The Markdown detail card continues to render every converted condition as a
list item. Only the trial-search table cell remains abbreviated. Preserve its
ten-item and 80-byte bounds, measured over the cleaned list, but reserve room
for the exact suffix `… [abridged; N conditions total]` whenever either bound
omits any part of the complete joined value. `N` is the size of the complete
cleaned vector. The whole cell, including the suffix, must be at most 80 bytes
and valid UTF-8. An unabridged cell is unchanged and carries no suffix.

This makes a long single condition distinguishable from a many-condition
trial: either can have a shortened label prefix, but the suffix gives the
complete list count.

## Test-first implementation

1. Add a transform regression using the receipted CTGov NCT02576665 value.
   Assert that `from_ctgov_study` and `from_ctgov_hit` produce the same ordered
   12-element vector. It fails today as 12 versus 10.
2. Add the equivalent transform regression using the first record in the
   receipted NCI melanoma capture. Assert that `from_nci_trial` and
   `from_nci_hit` both equal the 26 source disease names in source order. It
   fails today as 25 versus 10.
3. Add focused formatter/render tests for all three branches: an unchanged
   short list, more than ten short labels, and one multibyte label whose joined
   value exceeds the byte budget. The more-than-ten branch must mechanically
   assert that its cell ends with the complete exact suffix
   `… [abridged; 11 conditions total]`; the long-one-label branch must
   mechanically assert that its cell ends with
   `… [abridged; 1 conditions total]`. Both cells must remain valid UTF-8 and
   be at most 80 bytes. A containment assertion for only `abridged` or the
   numeric count is insufficient because it would not prove that the reserved
   suffix survived the cutoff.
4. Add public CLI contracts to `spec/entity/trial.md` using the existing
   fixture servers. JSON for CTGov NCT03590054 must contain all 21 conditions,
   and JSON for NCI NCT05929768 must contain all 26. The CTGov Markdown row must
   disclose `21 conditions total` and `abridged`.
5. Update `docs/user-guide/trial.md` to say that JSON condition arrays are
   complete while Markdown search cells are bounded and state the total when
   abridged.

Implement in the owning transform/renderer layers:

- add or use an uncapped condition-cleaning path for both CTGov converters;
- remove the `max` argument and terminal `.take(max)` from `nci_conditions`,
  updating both call sites without changing its parsing/error semantics; and
- make `format_conditions` compute the complete cleaned count before building
  the bounded cell and its reserved suffix.

`tests/test_capture_receipts.py` contains mutation tests whose search strings
name the current three-argument `nci_conditions` call. Update those expectations
to the new live call shape so the provider-key audit continues to exercise the
same comment-safe root/key discovery; do not weaken or bypass that audit.

## Done, observably

- The same CTGov provider value converted as detail and as a search hit carries
  the same 12 conditions, in order.
- The same NCI provider value converted as detail and as a search hit carries
  the same 26 conditions, in order.
- JSON search output exposes all 21 receipted CTGov conditions and all 26
  receipted NCI conditions.
- Both item-cap and byte-cap Markdown regressions end with the exact full
  suffix `… [abridged; N conditions total]`, are at most 80 bytes, and are
  valid UTF-8. A short unabridged cell has no such marker.
- The detail Markdown list is complete, and no public output silently presents
  a capped condition vector as source-complete.
- Documentation describes the JSON/Markdown distinction.

## Scope and exclusions

Do not change which conditions either provider returns, their order, duplicate
handling, or NCI unreadable-element failures. Do not add condition pagination
or a new field to `Trial` or `TrialSearchResult`.

Do not change intervention caps, intervention aliases, summaries, or the shared
`truncate_utf8` behavior. Ticket 1113 owns summary sentence/abbreviation
handling. A condition-specific bounded formatter may reuse a low-level UTF-8
helper only if its marker cannot itself be cut off.

Do not alter recorded captures or `testdata/sources/capture-receipts.json`; all
needed provider evidence is already present and receipted.

## Dependencies and roadmap impact

Dependencies: none. The necessary NCI reader behavior is already present in
the current tree from completed ticket 1107 (`633affc2`). Its source history
and completed record remain unchanged; 1111 neither depends on nor reopens it.

No later ticket depends on 1111 in frontmatter. Tickets 1112, 1121, 1122 and
1141 concern different trial fields or presentation paths and require no
change when this lands.

## Acceptance gates

Run focused Rust transform and Markdown-render tests, the focused Python
capture-receipt contract tests affected by the call-shape change, and the trial
mustmatch page while iterating. Before closure run the repository gates exactly:

```text
make lint
make test
make spec
```

## Review

- Design review: REJECT on 2026-09-04 because the test plan did not require the
  full suffix to survive both truncation branches and described ticket 1107 as
  live. Both findings were resolved above; independent re-review then ACCEPTED
  the design on 2026-09-04.
- Code review: REJECT on 2026-09-04 because the 11-item formatter regression
  proved the disclosure suffix but did not mechanically prove that the item
  bound retained item 10 and omitted item 11. The regression now asserts both
  boundaries while retaining the exact suffix, UTF-8, and 80-byte assertions;
  independent re-review then ACCEPTED the implementation on 2026-09-04.

## Completed 2026-09-04

Removed the hidden CTGov and NCI conversion caps so detail and search retain
the same complete, ordered, cleaned condition vectors and JSON exposes them
unchanged. The Markdown search formatter alone remains bounded: it preserves
the ten-item presentation limit, is valid UTF-8 and at most 80 bytes including
the exact `… [abridged; N conditions total]` suffix, and leaves short cells
unchanged. Public fixture-backed specs prove complete 21-condition CTGov and
26-condition NCI JSON results and disclosed Markdown abbreviation. The NCI
unreadable-element behavior and the capture audit remain intact.

Independent design review accepted the amended semantics after the exact
suffix, byte-budget, and completed-ticket-1107 corrections. A distinct
implementer produced red-before-green provider and formatter regressions. Fresh
code review rejected an item-bound assertion that could admit item 11; after a
mechanical item-10-present/item-11-absent remediation, independent re-review
accepted the implementation.

Final gates passed on the reviewed tree: `make lint`; `make test`, including the
complete Rust lane, 877 Python tests passed (3 skipped), and the strict
documentation build; and `make spec`, including the expanded 26-case trial page
and the static lane.
