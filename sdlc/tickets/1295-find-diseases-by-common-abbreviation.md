# 1295 — find diseases by common abbreviation

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: complete.

## Outcome

`search disease` finds the parent disease for common abbreviations such as NSCLC and DLBCL, using the abbreviation and synonym lists the disease source already holds. BioMCP adds no abbreviations of its own.

## Evidence

- Starts from: The owner ran `biomcp search disease NSCLC --limit 3 -j` on 0.9.1: no results. `search disease DLBCL` returned three subtypes (central nervous system, BN2, N1) and not `MONDO:0018905 diffuse large B-cell lymphoma`. The full name returns MONDO:0018905 first. Experiment 435 built candidate lists from BioMCP lookups for 120 disease mentions in cancer abstracts. The right disease was in the list for 19% of them. Abbreviations and loose names (DLBCL, NSCLC, HGSC) failed outright.
- Keeps: Full-name searches return the same first result.
- Changes: See Change detail.
- Proof: An input table in a spec page: NSCLC, DLBCL, HGSC, CRC, AML, and their full names, with expected first identifiers from recorded responses. A rerun of experiment 435's disease mentions reporting candidate recall before and after.
- Defers: Free-text phrases that are not names, such as "tumor".

## Build status

- Built on branch `tickets/1295-disease-abbreviations`, sha `4641dc5c`, 2026-10-05.
- Code review: ACCEPT 2026-10-05, no blocking findings. Three P2 report-only notes (class-name keeps live-verified only; theoretical window starvation of abbreviation holders; `^10` binds the trailing token of multi-word clauses, all pre-existing or measured-safe). Rulings: keep the `^10` provider-side lever over a structural second query; accept the recorded HGSC miss because the source holds no synonym and the ticket forbids BioMCP-side aliases.
- Root cause found deeper than triage: the shipped retrieval query scoped to `disease_ontology.synonyms` and `mondo.synonym`, which are object parents in the provider index holding no tokens; every synonym retrieval matched nothing. The fix targets the `exact` leaf fields. A `^10` name boost restores exact-name class parents (lymphoma, sarcoma, leukemia, carcinoma) to the top of the fetch window after the widened pool demoted them.
- Experiment 435 rerun, same method: candidate recall 23/120 (19.2%) before, 40/120 (33.3%) after; newly recalled CRS, DLBCL, HCC, HD, PR, RCC, exanthema, neoplasma; newly missed none. Artifacts: `experiments/435-link-entities-with-thinkthen/` (`scripts/rerun_disease_recall_1295.py`, `data/recall_1295_{before,after}.jsonl`, `data/recall_1295_report.json`).
- Input table proven live and from recorded replay: NSCLC, DLBCL, CRC, AML first-result parents; CAD resolves to all three exact holders in search while exact resolution refuses; HGSC returns no results (recorded miss); full names unchanged.
- Land-time record: `sdlc/records/1295-disease-abbreviations-build.md`. Land after branch CI green and yellow gates.

## Change detail

1. Measure how `search disease` matches synonyms today. The ticket review found that it queries MyDisease.info (`src/entities/disease/search.rs:241-255`), rewrites a few queries by hand (`src/entities/disease/resolution.rs:548-594`), and already reranks by MONDO and Disease Ontology synonyms with exact-match bonuses and a subtype penalty (`resolution.rs:412-492`). The likely gap is retrieval: the parent disease never enters the candidate set. Record which abbreviations the source's synonym lists actually hold.
2. Bring exact synonym and abbreviation matches from the source's own synonym lists into the candidate set, so the existing ranking can place the parent first.
3. Rerun experiment 435's disease mentions and report candidate recall before and after.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 9962598b (fresh researcher, read-only). The first pass accepted with notes; the revision recorded them.
