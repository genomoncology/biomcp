---
flow: build
priority: 3
deps: []
---

# 2147: Report every section batch article is asked for

Status: OPEN.

Milestone: 0.9.3

Reviews: revision 878f243, accept

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

## Proof recorded on the branch

- Red on main: Linux gate host job `1792207710207-2221274` at `8b8a503ba`
  (origin/main `b138f6097` plus this record and the acceptance tests, no
  product change). Both acceptance tests failed:
  `batch_article_detail_refuses_assets_sections_before_any_request` (main
  runs the batch instead of refusing) and
  `batch_article_detail_returns_no_tldr_section_unless_asked_for` (main
  reports a requested-sections-only batch with `section_outcomes.tldr`
  `"data"`). CI runs 38997800282 and 38998904048 failed the same way at the
  tests-only tips.
- Green: macOS gate host job `1792209765960-83224` ran the two acceptance
  tests, the fixture goldens, the preflight rejection matrix, the batch
  preflight unit test and the batch help test at `bdd7941` — 6/6 passed.
- Gates: `make lint` passed on the macOS gate host at `855b026`
  (job `1792210074926-7700`); `make test` passed on the Linux gate host at
  `855b026` (job `1792211112998-2414047`, 25.9 minutes); CI run
  `39001365788` is green on every lane at `855b026`. `make spec` on the macOS
  gate host fails only the frozen ClinGen ERepo page, identically at
  origin/main `b138f60` (job `1792213641126-45906`), so that failure
  predates this branch and is host-specific; CI's spec lane is green at the
  same tip.
- Live at the fixed tip (macOS gate host job `1792213551327-39998`): the
  reported command now exits 2 before any provider request with `Article
  batch does not support the "assets" section; asset lists and downloads
  need get article (example: biomcp get article 22663011 assets). Supported
  article batch sections: annotations, fulltext, indexing, tldr, all`, and
  `--sections fulltext` returns fulltext `data` (PMC) with `tldr`
  `not_requested`, no tldr provenance row and no Semantic Scholar payload.

## Acceptance

1. A test of `--sections fulltext,assets` returns both sections with outcomes, or refuses before any request; it fails on current main.
2. A test shows no `tldr` section unless asked for.
3. The changelog carries a bullet for this ticket.
