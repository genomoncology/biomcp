---
flow: build
priority: 5
---

# The same trial reports a different number of conditions depending on how it was fetched

Three paths keep three different amounts of the same list. `from_ctgov_study` keeps 25 conditions, `from_ctgov_hit` keeps 10, and `format_conditions` keeps 10 and then cuts the joined string to 80 bytes.

So a trial fetched by search and the same trial fetched directly disagree about what it studies, and nothing in either output says the list was shortened. A caller counting conditions, or checking whether a disease is present, gets an answer that depends on which call they happened to make.

Verified in `src/transform/trial.rs` on 2026-09-02 against `0.9.0-dev.6`.

## Required behavior

One trial reports one condition count.

A list that was shortened says so, so a reader never treats a truncated list as complete.

## Done, observably

- The same trial fetched through search and fetched directly reports the same conditions.
- A shortened list is marked as shortened, in Markdown and in JSON.
- A caller can tell a trial with few conditions from a trial whose list was cut.

## Correct behavior

Conversion applies no cap. Where the renderer shortens a list, the value it renders from is complete and the rendered form states that it is abridged.

Write that as a failing test, then fix. Red before green.

The assertion to write: The same trial reached through the detail path and the search-hit path carries an identical condition list.

This behavior is held against both 0.9 and 1.0 alike. It is recorded as case 11 of the clinical-trial conformance cases in the sibling BioData repository. **An attempt cannot read that repository**, so the statement above is this ticket's own authoritative copy, reconciled against the case when the ticket was filed. If it looks wrong, stop and say so rather than implementing something different.

## Boundary

Do not change which conditions a provider returns. Deciding the limit is design work; the requirement here is that one trial has one answer and that a cut is visible.
