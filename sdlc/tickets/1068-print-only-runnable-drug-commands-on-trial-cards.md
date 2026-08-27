---
flow: build
priority: 8
---

# Print only runnable drug commands on trial cards

Filed from `sdlc/issues/2026-08-27-trial-cards-print-get-drug-commands-for-non-drug-interventions.md`,
which carries the repro and the verified mechanism — read it first.

`get trial NCT06662188` prints `biomcp get drug JAG201`, which fails:
JAG201 is an intervention name, not a drug. The trial branch of
`src/render/markdown/related.rs` (~line 799) derives the command from the
first intervention string, while its sibling alias branch correctly prints
the search form. Ticket 1056 established the guarantee for variant cards;
this extends it to the trial→drug pivot.

## Done when

- A trial card never prints a `get drug` command whose name cannot resolve
  as a drug: intervention-derived names print `search drug -q ...` (the
  form the error message itself recommends), or the card verifies the name
  resolves before printing the get form — the design settles which.
- The same derivation sweep covers every card that builds drug commands
  from non-drug fields (arms, interventions, evidence strings); each site
  either prints the search form or proves resolvability.
- A test pins the JAG201 case: the exact card must print commands that all
  run, with the failing form absent.

Additional repro evidence (same sweep, 2026-08-27): the derivation is
ordering-dependent — `trial.interventions.first()` is registration order,
and `get drug Placebo` / `get drug Saline` both fail, so any trial listing
a placebo or saline arm first prints a broken command. NNZ-2591's card
(NCT07281079) only works because the drug happens to be listed first.

Filed as build, not quickfix: green suite, authored proof.
