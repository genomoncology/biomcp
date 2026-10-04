# 1295 — find diseases by common abbreviation

Proposed 2026-10-03 by the BioMCP owner for the next 0.9 release.

Status: OPEN.

## Outcome

`search disease` finds the parent disease for common abbreviations such as NSCLC and DLBCL, using the abbreviation and synonym lists the disease source already holds. BioMCP adds no abbreviations of its own.

## Evidence

- Starts from: The owner ran `biomcp search disease NSCLC --limit 3 -j` on 0.9.1: no results. `search disease DLBCL` returned three subtypes (central nervous system, BN2, N1) and not `MONDO:0018905 diffuse large B-cell lymphoma`. The full name returns MONDO:0018905 first. Experiment 435 built candidate lists from BioMCP lookups for 120 disease mentions in cancer abstracts. The right disease was in the list for 19% of them. Abbreviations and loose names (DLBCL, NSCLC, HGSC) failed outright.
- Keeps: Full-name searches return the same first result.
- Changes: See Change detail.
- Proof: An input table in a spec page: NSCLC, DLBCL, HGSC, CRC, AML, and their full names, with expected first identifiers from recorded responses. A rerun of experiment 435's disease mentions reporting candidate recall before and after.
- Defers: Free-text phrases that are not names, such as "tumor".

## Change detail

1. Measure how `search disease` matches synonyms today. The ticket review found that it queries MyDisease.info (`src/entities/disease/search.rs:241-255`), rewrites a few queries by hand (`src/entities/disease/resolution.rs:548-594`), and already reranks by MONDO and Disease Ontology synonyms with exact-match bonuses and a subtype penalty (`resolution.rs:412-492`). The likely gap is retrieval: the parent disease never enters the candidate set. Record which abbreviations the source's synonym lists actually hold.
2. Bring exact synonym and abbreviation matches from the source's own synonym lists into the candidate set, so the existing ranking can place the parent first.
3. Rerun experiment 435's disease mentions and report candidate recall before and after.

## Review

- Design review: ACCEPT 2026-10-03 on the second pass, dispatch 9962598b (fresh SWE-2 researcher, read-only). The first pass accepted with notes; the revision recorded them.
