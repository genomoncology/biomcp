---
flow: build
priority: 3
deps: []
---

# 2147: Report every section batch article is asked for

Status: OPEN.

Milestone: 0.9.3

## Outcome

`batch article --sections` returns each requested section or an outcome saying why it is missing, adds no section nobody asked for, and refuses a section it never supports.

## Evidence

Reported 2026-10-16 by the research team on the released 0.9.2.

- `biomcp --json batch article 9021520,9096355 --mode detail --sections fulltext,assets` returns full text but no assets and no outcome for that section, and includes an unrequested `tldr` section.
- The `get article` help lists annotations, indexing, fulltext, tldr, assets, `asset <asset-key>` and all (`src/cli/article/mod.rs:310`). Batch builds items in `src/entities/article/batch.rs` and parses sections in `src/cli/system/batch.rs`.

## Reproduction on main (2026-10-16, before any code)

Live probe at `b138f6097` (origin/main) on the macOS gate host, pm job
`1792206258508-89030` (read-only public GETs), the exact reported command:

```bash
biomcp --json batch article 9021520,9096355 --mode detail --sections fulltext,assets
```

Exit 0, summary `{failed: 0, succeeded: 2, total: 2}`, both items `ok`, and
identically for each article:

- `section_outcomes`: `fulltext: data (PMC)`, `indexing: not_requested`,
  `tldr: data (Semantic Scholar)`; no `assets` key exists in any channel.
- `_meta.section_sources` rows: bibliography, authors, abstract, fulltext,
  tldr — the unrequested `tldr` row appears on both articles.
- No assets payload of any kind: no manifest, no asset list, no outcome.

The report reproduces exactly: a requested `assets` section returns nothing
and says nothing, and an unrequested `tldr` section appears.

## Work

- Reproduce on main first.
- Support `assets` in batch, or refuse it at parse time naming the supported sections. Pick one and record why in this ticket.
- Return only requested sections, and give each requested section that comes back empty an outcome.
- State the supported sections in the batch help and the CLI reference.

## Decisions

- Batch refuses `assets` and `asset <asset-key>` at preflight, naming the
  supported sections (annotations, fulltext, indexing, tldr, all). Why refuse
  rather than support: the assets surface is per-article and interactive —
  asset keys are article-specific, downloads write to a destination file, and
  the manifest views take per-asset paging flags (`--asset-view`,
  `--asset-limit`, `--asset-offset`) a uniform batch flag cannot carry per
  item; every section batch supports today resolves the same way for every
  requested ID. Supporting assets would also need a new `assets`
  section-outcome key in every article payload, reshaping `get article`
  output for a batch-only ask. The refusal fires in preflight before any
  provider request.
- The unrequested `tldr` leak is fixed in the shared article detail path that
  `get article` and batch both use: the tldr section outcome is completed
  only when the request carries the Semantic Scholar section (`tldr`, `all`,
  or a base-card request with no `--sections`). The Semantic Scholar lookup
  itself still runs when only fulltext was asked, because it feeds PDF
  discovery for `--pdf`; its outcome and payload stay out of the response.
  That seam is where ticket 2151 attaches batch `--pdf`.
- A requested `annotations` section that comes back empty (PubTator lag)
  keeps its existing channel — `pubtator_fallback: true` in JSON and the
  markdown note — rather than a new `annotations` section-outcome key, which
  would reshape every article payload for a base-card section every article
  carries. fulltext, indexing and tldr each already report data, empty and
  unavailable outcomes.

## Acceptance

1. A test of `--sections fulltext,assets` returns both sections with outcomes, or refuses before any request; it fails on current main.
2. A test shows no `tldr` section unless asked for.
3. The changelog carries a bullet for this ticket.
