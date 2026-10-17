---
priority: 2
---

# 2181 — Search full text in the exact variant name strategy

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 428ecb0, accept

## Outcome

The exact variant-name strategy searches full text wherever a source supports it, so a variant named only in a paper's body or tables is found.

- Evidence: research's spot check: `--strategy exact` for `BRCA1 F1761I` finds 0 papers while the union strategy's lexical route finds 10811118 under the same name. `src/entities/article/variant_search.rs:3085-3094` wraps the name in PubMed `[Title/Abstract]` and Europe PMC `TITLE_ABS:` only.
- Keeps: one exact name per query, per-alias and per-source yield in `alias_yields`, and PubMed's title and abstract phrase search, since PubMed has no full-text field.
- Changes: claims `src/entities/article/variant_search.rs` `tests/test_variant_article_exact_fulltext.py` `tests/fixtures/article/exact-name-fulltext.json` `docs/user-guide/variant.md` `CHANGELOG.md`. Search Europe PMC by the exact caller phrase across full text and retain its own query key.
- Proof: recorded replies show literal `F1761I` finding 10811118 through Europe PMC full text while current main TITLE_ABS finds none. Whole caller phrase `BRCA1 F1761I` remains an honest zero control. Alias yields stay per source.

Coordinator outcome before code: Search Europe PMC full text for one exact variant-name phrase, retaining PubMed title/abstract search and per-source query/yield attribution. Before product edits, reproduce on current main through the build job and report the exact command/output, proposed implementation/test files and the outside-in failing proof. Root records that evidence and scope before implementation. Developers write no ticket or status records.

## Progress

- 2026-10-17 started


## Coordinator reproduction and scope

Qualified CLI source `926b5786d400d3ec8b54c929701c1e46c62b5a95` differs from current main `90c00c371` only by ticket records. CLI SHA256: `e9992fadcb93e290a6c310827a14368444dcd1305784661c659c46311ed9083d`.

Outcome remains one exact caller phrase per source, Europe PMC full text and PubMed title/abstract. Job 2181/1792273560432-80321 ran `biomcp --json --no-cache --disable-sources pubmed,pubtator3,semantic-scholar variant articles 'BRCA1 F1761I' --strategy exact --debug-plan --limit 5`: Europe PMC healthy, hits0, query TITLE_ABS:"BRCA1 F1761I". Exit1 reflects deliberately disabled sources.

Recorded provider replies: whole phrase "BRCA1 F1761I" returns0 even across full text; "F1761I" returns PMID10811118; TITLE_ABS:"F1761I" returns0. PM decision needed: preserve approved exact caller phrase and correct positive proof to literal F1761I, with whole phrase honest zero control. Do not broaden to gene-token conjunction. Proposed claims: existing variant search source, tests/test_variant_article_exact_fulltext.py, tests/fixtures/article/exact-name-fulltext.json and changelog.

PM decision: 2026-10-17. Confirmed: literal F1761I positive through full text (10811118), whole caller phrase zero control, one exact caller phrase without changed identity/query semantics.

Actual outside-in RED: job2181/1792273942087-15084 ran the recorded-server CLI cases against qualified current main: two failures in1.18s. Literal F1761I lacks PMID10811118; whole caller phrase sends TITLE_ABS rather than the unfielded exact phrase. Production edits follow this failure.

GREEN2181/1792274031353-47616 passed: two actual recorded-server CLI cases, two existing provider/yield controls, fmt, affected Ruff, targeted Clippy with warnings denied and quality ratchet. Europe PMC now uses unfielded exact phrase with europepmc-fulltext-phrase-v1; PubMed remains title/abstract. All checks used the applied local source patch; fresh review follows without duplicated development checks.

Review scope correction: update the existing exact-strategy sentence in docs/user-guide/variant.md to describe Europe PMC full-text phrase search while preserving PubMed title/abstract. This removes a stale behavior claim; the accepted actual CLI checks remain applicable.
