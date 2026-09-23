---
flow: build
priority: 3
deps: [2002]
---

# 2004: Degrade conditions only when con-3 is broken

## Outcome

`get patient <id> conditions` marks the section `degraded` only when a Condition breaks FHIR R4 invariant con-3. Each row shows its own clinical status, or shows it as absent when the source omits it. On the blue demo server, patients whose Conditions all satisfy con-3 come back `data`, not `degraded`.

## Current Facts

- Ticket 2002 marks the section `degraded` whenever any Condition lacks `clinicalStatus` (`sdlc/tickets/2002-get-a-patient-and-conditions-from-fhir.md`, Scope).
- On the blue demo server, every patient comes back `degraded`, because many Conditions there carry no `clinicalStatus`. The mark tells the reader nothing when it never varies.
- FHIR R4 invariant con-3: a Condition whose `category` includes `problem-list-item` must have `clinicalStatus`. A Condition with `verificationStatus` of `entered-in-error` is exempt regardless of category.
- `SectionOutcomeState` already offers `degraded` alongside `data` (`src/entities/section_outcome.rs:7`).

## Scope

- Replace the "any Condition lacks `clinicalStatus`" degrade rule with a con-3 check: a Condition degrades the section only when its `category` includes `problem-list-item`, its `verificationStatus` is not `entered-in-error`, and it has no `clinicalStatus`.
- Every other missing-`clinicalStatus` Condition still renders. Its row shows clinical status as absent instead of omitting the field.
- The section stays `data` when every Condition either satisfies con-3 or is exempt from it.

## Exclusions

No other con-* invariant. No change to the paging, redirect, or repeated-link degrade rules ticket 2002 already sets.

## Acceptance

Synthetic bundles only, served by the existing spec fixture runner:

1. A fixture with encounter-diagnosis Conditions and no `clinicalStatus` renders `data`, and each such row shows clinical status as absent.
2. A `problem-list-item` Condition with no `clinicalStatus` renders `degraded`.
3. A `problem-list-item` Condition with no `clinicalStatus` but `verificationStatus` `entered-in-error` renders `data`.
4. `spec/entity/patient.md` gains these three cases.

`make lint`, `make test`, and `make spec` pass on the gate host at the pushed SHA.

## Dependencies

2002.

## Complexity

- Contract score: 1 (one degrade rule, replaced)
- State and timing score: 0
- Reach score: 1 (conditions section only)
- Proof score: 1 (three pinned fixture cases)
- Cost of error score: 1 (a wrong degrade mark misleads the reader about data quality)
- Total: 4
- Minimum level floor: none
- Final level: 1
- Reasons: a scoped rule change on one existing section
- Selected model: claude-sonnet

## Review

- Design review: pending
