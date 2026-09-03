---
flow: build
priority: 2
---

# The adverse-event percent column reads as an incidence rate and is not one

`biomcp drug adverse-events vincristine` prints a percentage that a clinician will read as a rate of occurrence in treated patients. Verified against the repository build `biomcp 0.9.0-dev.6` (`./target/debug/biomcp`) on 2026-09-01:

```
- Total reports (OpenFDA FAERS): 104334
- Top reactions: aggregate count query; Percent = count / total reports.
| Reaction | Count | Percent |
| FEBRILE NEUTROPENIA | 11433 | 11.0% |
| OFF LABEL USE | 8387 | 8.0% |
| NEUTROPENIA | 6992 | 6.7% |
```

FAERS holds spontaneous reports and exposes no count of treated patients, so there is no denominator from which an incidence can be computed. The figure is a share of reports. The existing note explains the arithmetic and not what the arithmetic means, and `OFF LABEL USE` sitting second in a list of reactions shows how far the column is from a clinical rate.

A second confusion sits in the same output. The per-report table lists rows whose drug is `pegfilgrastim`, `prednisolone` or `dexamethasone`. Those are co-reported agents from the same reports, and nothing marks them as such, so a reader can attribute another drug's reaction to the one they asked about.

## Required behavior

A reader who does not know what FAERS is cannot mistake the percent column for a rate of occurrence in treated patients.

A row describing a co-reported agent is distinguishable from a row describing the drug that was asked about.

## Done, observably

- The adverse-event output states what the percentage is a share of, in terms that do not require prior knowledge of spontaneous reporting.
- Co-reported drugs in the per-report table are marked as co-reported.
- The counts and the arithmetic are unchanged.

## Boundary

This ticket changes how the numbers are framed and labelled. It does not change what is queried, does not change the counts, does not add a disproportionality statistic, and does not remove the percent column.
