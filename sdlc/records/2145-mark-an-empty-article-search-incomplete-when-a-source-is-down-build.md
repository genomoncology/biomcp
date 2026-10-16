# 2145 build record — the search-article zero-row outage rule

Branch `ticket/2145-mark-an-empty-article-search-incomplete-when-a-source-is-down`,
tip `926e326be` (2026-10-16), rebased onto main `74dfc40e9` (the package-file
pin fix that landed after this branch started; the pins merge to 1,408).
`search article` now follows the rule `variant articles` already follows,
coded at `src/entities/article/variant_search.rs` and documented in
`docs/how-to/find-articles.md` and `docs/user-guide/cli-reference.md`.

## What changed

- `ArticleSearchPage` (`src/entities/article/mod.rs`) carries `complete`,
  computed once in the entity layer from the final `source_status`
  (`article_source_statuses_complete` in `src/entities/article/search.rs`),
  mirroring the variant-article `source_statuses_incomplete`: a page is
  complete when every reported source is `ok`, `skipped`, or carries no
  availability. Sources only appear in `source_status` when they were
  degraded or unavailable (the Semantic Scholar tracker always reports), so an
  empty list means every source answered.
- The CLI JSON (`ArticleSearchResponse` in `src/cli/article/dispatch.rs`)
  adds `complete` beside `pagination` and a structured `error` (skipped when
  absent) built only when `complete` is false and no rows arrived:
  `{"code": "source_unavailable", "message": "no article rows were returned
  while some sources were unavailable: <down sources>"}`. The message names
  the degraded or unavailable sources, deduplicated and sorted.
- `handle_search` applies the exit rule: zero rows with a down source exits 1
  with the JSON on stdout; in Markdown mode the same condition prints
  `Article search returned no rows while some sources were unavailable: …`
  on stderr and exits 1, mirroring the variant-articles Markdown path. Rows
  present with a down source keep the normal exit-0 reply with
  `complete: false`.
- The rule is stated for `search article` in the two documents the ticket
  names: `docs/how-to/find-articles.md` (Start from a known anchor) and
  `docs/user-guide/cli-reference.md` (Search → Article).
- CHANGELOG bullet under 0.9.3 Fixes.

Scope held: `variant articles` code is untouched, `search all` and every other
`search_page` caller read the page as before (`complete` is additive), and the
MCP typed-search path returns the same JSON text with no disposition change
(exit codes are CLI-only there).

## Proofs

Each proof fails on current main first and passes on the branch.

- `src/cli/article/tests/outage.rs` (new sidecar, registered in
  `src/cli/article/tests/mod.rs`): a local fixture server stands in for every
  article provider on one origin (PubTator `text=`, Europe PMC `query=`,
  PubMed eutils, Semantic Scholar `/graph/v1`, PubTator autocomplete and
  biocjson export), driven through the real `handle_search` on the real argv
  `--json search article -g ABRAXAS1 -k p.G39V --limit 5`:
  - Semantic Scholar 503s, every other source answers empty: exit 1,
    `complete: false`, `error.code == "source_unavailable"`, the message
    names Semantic Scholar, and the request log pins that the fixture really
    saw the Semantic Scholar search (the assertion cannot pass vacuously).
  - Every source answers empty: exit 0, `complete: true`, no `error` field.
  - Europe PMC returns one row while Semantic Scholar 503s: exit 0,
    `complete: false`, no `error` field, `count: 1`.
- Red on main: branch `tickets/2145-red-proof` = origin/main + the test file
  only, at `9e80d7c6e`. Yellow job `1792190160322-1866808`
  (`cargo nextest run --no-default-features -E 'test(cli::article::tests::outage)'`):
  3 run, 0 passed — the down-source case exits 0 with no `complete`, the two
  completeness assertions read `Null`. Green on the branch: yellow job
  `1792190666531-1901461` at `200f5004e`, 3/3 passed. (A first red attempt at
  `eacb19f8a`, job `1792189138841-1828593`, failed on a test-side argv bug —
  the fixture command was missing the `article` word — and proved nothing; the
  corrected red is the one that counts.)
- Rust structure and packaging pins: `tests/article_cli_tests_structure.rs`
  lists the new sidecar, and `tests/test_source_package_boundary.py` carries
  `MAX_PACKAGE_FILES` 1,408 (main's `74dfc40e9` 1,407 plus the one new
  packaged file). A first bump to 1,407 rode the stale pre-`74dfc40e9` base
  and CI counted 1,408 actual; the rebase resolves it.

## Gates (yellow = routine offline lane, M5 = gate host, Linux CI authoritative)

- `make test` (yellow): `1792191210181-1920047` at `200f5004e` ran the Rust
  lane 4,164/4,165 with the structure-pin failure (fixed next commit);
  `1792191661120-1956874` at `79980358d` passed the Rust lane and failed the
  Python lane's two package-file tests on the stale pre-`74dfc40e9` base;
  `1792192290683-1988598` at the rebased tip `926e326be` passed both lanes —
  Rust 4,289 run, 4,289 passed (33 skipped), Python 1,155 passed (3 skipped),
  exit 0 in about 7 minutes.
- `make lint` (M5, `1792190225711-16682`): failed on the machine, not the
  change — no pinned ShellCheck/actionlint archive for darwin-arm64 (same as
  the 2124 record). The CI `lint` job passed at `200f5004e` (run
  `38976939276`).
- `make spec` (M5, `1792191214685-40694`): failed on the machine at
  prepare-spec — the M5's CommandLineTools Python 3.9 lacks `tomllib`, which
  `tools/with-build-identity` imports. The CI `spec-lane` job passed at
  `200f5004e` (run `38976939276`).
- CI on the branch: run `38976939276` (`200f5004e`) — build, lint,
  spec-lane, repository-contracts, release-checks, generated-sources green;
  rust-tests failed on the structure pin and python-lane on the package-file
  pins, all fixed in `79980358d`; run `38978578427` (`79980358d`) passed
  build, lint, spec-lane, rust-tests, repository-contracts and still failed
  python-lane's file count because the branch predated main's `74dfc40e9`
  pin fix. The tip run `38980012142` on `926e326be` is green in every lane
  (build, lint, rust-tests, python-lane, spec-lane, repository-contracts,
  release-checks, generated-sources); that run is the landing bar.

## Findings

- A deliberately disabled source (`--disable-sources`) also reports
  unavailable, so a zero-row search with a source the user turned off exits 1
  with `source_unavailable` under this rule. That matches the pinned
  outage-rule amendment ("`source_unavailable` and exit 1 stay for a zero-row
  result where a provider source reported unavailable or timed out") and
  2124's "a disabled source answers as an unavailable section". The
  disable-sources walk only asserts wire silence for routed entries, so it is
  unaffected; if a disabled source should not count as an outage, that is a
  product call for the 2130 lane that owns the amendment.
- 2130's lane is editing `src/entities/article/query.rs`,
  `src/entities/article/variant_search.rs`, and `src/sources/pubtator.rs` for
  the same document family. This branch touches none of those files; the only
  shared surface is the CHANGELOG section and the two documents' lines around
  the variant-article rule, so rebase conflicts should stay textual.
- The M5 cannot currently run `make lint` (no darwin lint tools) or
  `make spec` (Python 3.9 without `tomllib`); ticket 2137 covers bringing the
  gates to the macOS host.
- `src/cache/manager.rs` emits an `unused import: std::fs` warning on the
  branch and on pure main alike; pre-existing, not touched here.
