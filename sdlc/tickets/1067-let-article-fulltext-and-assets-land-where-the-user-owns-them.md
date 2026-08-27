---
flow: build
priority: 7
---

# Let article fulltext and assets land where the user owns them

Filed from `sdlc/issues/2026-08-27-article-downloads-to-user-directory.md`,
which carries the original finding, the verified code path
(`src/utils/download.rs:20`), and the smallest-useful-version sketch — read
it first.

The gap is now validated twice independently: an external reproduction
experiment (experiments/186) hit it against released 0.8.25, and the
botassembly knowledge-base run (experiments/188,
`experiments/188-shank3-knowledge-base/kb/raw/RUN-LOG.md`) declined
full-text retrieval entirely for exactly this reason — outputs land in a
hash-named managed cache with no map back to the article. A researcher or
agent building a local collection has to copy files by hand and invent
names.

## Done when

- `biomcp get article <id> fulltext --out DIR` writes `DIR/<pmid>-<slug>.md`
  — a person-and-agent findable filename — with a small frontmatter block
  (pmid, pmcid, doi, title, journal, date, retrieved-at, source rung).
- `biomcp get article <id> asset <key> --out DIR` writes the raw bytes to
  DIR under the asset's own name.
- Without `--out`, behavior is exactly as today: stdout summary, cache
  placement, nothing else changes.
- A test pins the naming scheme and frontmatter fields, and another pins
  the unchanged default path.

## Explicitly out of scope (the issue's "later" list, unchanged)

A `library` concept (default directory via config, `library list/search`),
and batch fetch into it. If the small version earns it, that is its own
ticket.
