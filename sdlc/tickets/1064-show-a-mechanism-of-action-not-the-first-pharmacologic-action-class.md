---
flow: build
priority: 9
---

# Show a mechanism of action, not the first pharmacologic-action class

Filed from `sdlc/issues/2026-08-26-drug-mechanism-shows-pharmacologic-action.md`,
which carries the full verified mechanism — read it first.

In brief: the Drugs section of `search all --gene BRAF --disease melanoma`
shows dabrafenib's mechanism as "Cytochrome P450 2C9 Inducers" — a MeSH
pharmacologic-action term for enzyme induction, not the drug's mechanism.
Verified cause: `fallback_mechanism_from_hit` (`src/transform/drug.rs`)
takes the first NDC pharm class tagged `[MoA]`, and the NDC list orders the
inducer class first. The correct answer exists in the upstream data —
DrugCentral's `fda_moa` for dabrafenib lists "Protein Kinase Inhibitors"
first — and vemurafenib's row is right only because its single MoA class
happens to be the mechanism.

## Done when

- The mechanism column shows a mechanism of action for drugs where the
  upstream data contains one: prefer the ChEMBL `mechanism_of_action`
  record, then a kinase/BRAF-classifying MoA entry, before any
  enzyme-induction or metabolism class. The design settles the ranking
  policy and writes it down.
- Dabrafenib renders as a BRAF/kinase inhibitor; vemurafenib's row is
  unchanged.
- A drug whose upstream data genuinely carries no mechanism renders an
  honest empty state, not a misleading pharmacologic-action term.
- The ranking policy is pinned by tests using both drugs as the
  contrasting fixtures.

Filed as build, not quickfix: green suite, authored proof, and a ranking
policy the design stage must settle.
