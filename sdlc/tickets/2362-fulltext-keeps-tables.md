# 2362 — Full text keeps its tables

Status: OPEN.

Milestone: 0.9.4

Priority: 1

- Changes: `CHANGELOG.md`, `src/transform/article/jats.rs`, `tests/feature-links.json`, `tests/test_article_fulltext_tables.py`.

## Outcome

Article full text includes every table in the source document, so a variant named only in a table is found by line reads and searches.

## Evidence

- Research recheck of the 0.9.4 candidate b56138d27 (research repo f1c305c, sdlc/records/0042-test-the-0-9-4-candidate-hard.md; raw outputs on Blue under /home/ian/workspace/data/research/runs/0042/), relayed by Jarvis in m3902. These are observations: reproduce on current main first, and withdraw with evidence if it does not reproduce.
- C001-TABLE: `biomcp get article 20516115 fulltext --lines 1:200 --json` returns all 161 lines with truncated=false but drops Table 2 and its literal P1749R. NCBI efetch db=pmc id=3040717 has table-wrap T2 with that token. Europe PMC fullTextXML for PMC3040717 returned 500 from Blue and the Beelink, so the fallback route is the one under test.

## Proof

- Outside-in replay test: the fixture document with a table-wrap renders the table rows, and the P1749R token appears in the returned lines.
- Lands before the final 0.9.4 requalification and rehearsal; research rechecks this item on the final build.
