# 2147 build record — report every section batch article is asked for

## Report being fixed

`biomcp --json batch article 9021520,9096355 --mode detail --sections
fulltext,assets` on 0.9.2 returned full text, returned nothing for the
requested `assets` section with no outcome anywhere, and included an
unrequested `tldr` section.

## Root cause

- Batch article detail dispatches each item through
  `crate::entities::article::get(id, &batch_sections, default options)`
  (`src/cli/system/dispatch.rs`). `parse_sections` accepts `assets` and
  `asset <asset-key>`, but `get()` never acts on those flags — the asset
  surface is wired only into the `get article` CLI handler
  (`src/cli/article/assets.rs::handle_asset_get`), which batch never calls.
  No `assets` key exists in the article section-outcome registry either
  (`ARTICLE_OUTCOME_KEYS` is fulltext, indexing, tldr), so the requested
  section had no channel to report through. Silence, not a wrong fetch.
- `get()` unconditionally completed the `tldr` section outcome after the
  Semantic Scholar enrichment, so every section-only request that did not ask
  for tldr still carried `section_outcomes.tldr` (and a `_meta.section_sources`
  tldr row) in JSON. The markdown renderer already hid the section; the JSON
  outcome map did not.

## Change

- `src/cli/system/batch.rs`: article batch preflight refuses `assets` and
  `asset <asset-key>` before any provider request, naming the supported
  sections and pointing asset work at `get article`. Decision recorded in the
  ticket: refuse rather than support, because the asset surface is per-article
  and interactive (article-specific keys, download destinations, manifest view
  paging) while batch sections resolve identically for every requested ID, and
  supporting assets would add a section-outcome key to every article payload.
- `src/entities/article/detail.rs`: the tldr outcome completes only when the
  request carries the Semantic Scholar section (`tldr`, `all`, or a base card
  with no `--sections`). The lookup itself still runs — its result feeds PDF
  discovery for `--pdf` — but an unrequested tldr stays `not_requested` and
  out of `_meta.section_sources`. `get article` shares this path, so its
  section-only JSON stops leaking tldr the same way.
- Help and reference: the batch `--sections` argument doc, the batch `NOTES`
  block (`src/cli/commands.rs`), and the CLI reference's Batch mode section
  state the supported article batch sections.
- Tests: `tests/article_batch_outcome.rs` gains the two acceptance tests
  (assets refusal before any request — exit 2, untouched request log and
  cache; no tldr unless asked — outcome map, provenance rows and payload);
  `src/cli/system/batch.rs` unit tests pin the refusal message;
  `src/cli/system/tests.rs` pins the help line; `spec/entity/article.md`
  gains a reader-facing section and the preflight block gains the assets
  refusal lines. `CHANGELOG.md` carries the Fixes bullet.

## Red on main

Linux gate host job `1792207710207-2221274` at `8b8a503ba` (origin/main
`b138f6097` + ticket record + tests, no product change): both acceptance
tests failed — `batch_article_detail_refuses_assets_sections_before_any_request`
because main runs the batch instead of refusing, and
`batch_article_detail_returns_no_tldr_section_unless_asked_for` with
`section_outcomes.tldr` reporting `"data"` on a fulltext-only request. CI
runs 38997800282 and 38998904048 failed the same way at the tests-only tips.

## Green evidence

| Job | Machine | Command | Verdict |
| --- | --- | --- | --- |
| 1792209765960-83224 | macOS gate host | targeted nextest: the two acceptance tests, fixture goldens, preflight matrix, preflight unit test, help test at `bdd7941` | passed, 6/6 |
| 1792210074926-7700 | macOS gate host | `make lint` at `855b026` | passed |
| 1792213551327-39998 | macOS gate host | live probe at `855b026` | passed, see below |
| 39001365788 | GitHub Linux | full ci.yml at `855b026` | green, every lane |
| 1792211112998-2414047 | Linux gate host | `make test` at `855b026` | passed (exit 0, 25.9 minutes) |

### Live probe (read-only public GETs, macOS gate host)

- The ticket's exact reported command exits 2 before any provider request:
  `Article batch does not support the "assets" section; asset lists and
  downloads need get article (example: biomcp get article 22663011 assets).
  Supported article batch sections: annotations, fulltext, indexing, tldr, all`.
- `--json batch article 9021520 --mode detail --sections fulltext` exits 0
  with fulltext `data` (PMC), `tldr` `not_requested`, no tldr row in
  `_meta.section_sources`, and no `semantic_scholar` payload.

## `make spec` on the macOS gate host

Fails the frozen ClinGen ERepo page (`clingen-erepo-frozen`, lines 24 and 28)
at `855b026` (1792210119551-15802, retried 1792213611084-42636) and
identically at origin/main `b138f60` (1792213641126-45906): the failure
predates this branch and is host-specific. CI's spec lane is green at the
same tip. All 59 other blocks pass on the host at `855b026`. Whoever owns the
gate host's fixture setup should look at the ERepo supervisor there.

## Findings

- Requested `annotations` sections that come back empty keep the
  `pubtator_fallback` flag and the markdown note as their outcome channel; no
  `annotations` key was added to the section-outcome registry (decision and
  reasoning in the ticket). If review wants annotations inside
  `section_outcomes`, that is a schema change across every article payload and
  deserves its own ticket.
- `cargo nextest` still emits the pre-existing `unused import: std::fs`
  warning for `src/cache/manager.rs:746` (present on main; see 2146's record).
- The one-job-at-a-time yellow queue ran long tonight; the `make test` gate
  waited behind sibling tickets 2148–2150.
- origin/main moved during the build (2138 and 2148 landed; new tip
  `f9a11e81e`). `git merge-tree` against that tip reports exactly one
  conflict, both sides adding bullets to `CHANGELOG.md`'s Fixes list — the
  landing rebase is mechanical and stays with the coordinator per the
  team's flow.

## Seams for 2151 (batch `--pdf` fulltext under one limiter)

- `ArticleGetOptions { allow_pdf, include_asset_summary }`
  (`src/entities/article/detail.rs`) already threads PDF permission through
  the shared `get()`; batch article detail currently passes `default()`
  (`src/cli/system/dispatch.rs`, article detail arm). `allow_pdf: true` is
  the whole entity-side wiring.
- `get()` refuses `--pdf` without the fulltext section (`--pdf requires the
  fulltext section`); batch preflight (`validate_batch_args` in
  `src/cli/system/batch.rs`) should mirror that refusal when batch gains the
  flag, so preflight keeps its no-provider-work guarantee.
- The tldr gate added by this ticket keeps the Semantic Scholar lookup alive
  on fulltext-only requests specifically so
  `fulltext::pdf_discovery_attempt` gets its enrichment result; a batch
  `--sections fulltext --pdf` inherits PDF discovery without reporting a
  tldr outcome. The `pdf_discovery` handoff in `get()` is the seam.
- Concurrency: batch settles items through `settle_batch` with
  `BATCH_MAX_IN_FLIGHT = 10` (`src/cli/system/batch.rs`); the in-process
  Semantic Scholar limiter inside `SemanticScholarClient` is what spaces the
  paper/PDF requests, so per-item futures need no extra gate.
- MCP: the typed get maps sections to `biomcp get article ...` commands
  (`src/mcp/shell` typed-get mapping); a `pdf` option maps to the `--pdf`
  flag on that command, and the typed schema test
  (`src/mcp/shell/typed_get_tests.rs`) is where the option gets pinned.
