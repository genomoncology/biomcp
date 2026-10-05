# 1295 build: find diseases by common abbreviation

Ticket: sdlc/tickets/1295-find-diseases-by-common-abbreviation.md
Branch: tickets/1295-disease-abbreviations, sha 4641dc5c
Built by a lane worker; reviewed fresh; driven by the branch lead.

## Prior evidence

Experiment 435 measured disease candidate recall at 19 percent. The
owner's 0.9.1 checks showed abbreviation searches returning nothing or
subtypes instead of the parent disease.

## Root cause

Deeper than triage assumed: the shipped retrieval query scoped to
disease_ontology.synonyms and mondo.synonym. Those are object parents in
the MyDisease index; only their exact leaf fields hold tokens. Every
synonym-based retrieval matched nothing, so abbreviations failed while
name-field searches worked.

## Changes

- Retrieval targets disease_ontology.synonyms.exact and mondo.synonym.exact.
  The provider adds no abbreviations of its own; strict exact-token scope.
- Name clauses carry a measured ^10 relevance boost: the widened candidate
  pool had pushed exact-name class parents (lymphoma, sarcoma, leukemia,
  carcinoma) below the fetch window.
- Ambiguity: search surfaces every exact holder (CAD has three) above
  loose matches; exact resolution refuses when more than one disease holds
  the token (pre-existing behavior, now pinned).
- Spec table in spec/entity/disease.md replays 12 recorded MyDisease
  responses, including the CAD set-equality block and the recorded HGSC
  miss.

## Proof

- Experiment 435 rerun, same method: candidate recall 23/120 (19.2%)
  before, 40/120 (33.3%) after; newly recalled CRS, DLBCL, HCC, HD, PR,
  RCC, exanthema, neoplasma; newly missed none. Artifacts in
  experiments/435-link-entities-with-thinkthen/.
- Code review ACCEPT, no blocking findings. Three P2 report-only notes
  recorded in the ticket. Rulings: keep the ^10 lever over a structural
  second query; accept the HGSC miss.

## Deferred gaps

- Class-name keeps (lymphoma and siblings) are live-verified only; a
  recorded fixture row per class name is a cheap follow-up.
- Remaining recall misses are loose names and subsumed concepts, not
  abbreviations; new-ticket territory.
- HGSC needs a provider-side synonym or an owner decision to allow a
  BioMCP-side alias table.
