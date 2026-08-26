---
flow: build
priority: 8
---

# Do not present gnomAD's residual grouping as an ancestry row

Found in the 2026-08-26 slide-lab run (experiments/184, keyless, binary
0.9.0-dev.6), re-verified against current main the same day.

## What a reader sees

`biomcp get variant rs1426654 population` renders gnomAD v4
per-population rows, and one row's population id is the literal string
`remaining` — gnomAD's residual bucket for individuals not assigned to a
named genetic ancestry group. In the markdown table it reads as though it
were an ancestry group like any other, which misleads both human readers
and agents summarizing the card ("rare in whom?" needs real group names).
The JSON carries the raw id `remaining` too.

## The design choice this ticket settles

- Label it in markdown rendering (e.g., "Other / not assigned (gnomAD
  residual)") while JSON keeps the truthful raw id as the machine key, or
- exclude the residual row from default tables with a note that it exists,
  or
- keep raw ids everywhere but document the vocabulary.

Design picks one and pins it; population-table assertions and any
spec text restating them belong to the design stage.

## Done when

- The markdown population table never presents `remaining` as an unlabeled
  ancestry row, under the chosen resolution.
- JSON output remains truthful (no renaming of the raw id unless the design
  says so and says why).
- Tests pin the rendered table for a variant whose population data includes
  the residual group.

Filed as build, not quickfix: the suite is green — the current rendering
is asserted nowhere as wrong, and display-vocabulary changes restate
pinned rendering assertions, which is design-stage work.
