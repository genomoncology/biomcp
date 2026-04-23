# Changelog

## 0.8.22 — 2026-04-23

### New features

- Extended WHO Prequalification drug coverage to include active pharmaceutical
  ingredients (APIs) alongside finished pharmaceutical products;
  `search drug --region who --product-type api` filters explicit WHO searches
  to API rows. (232)
- Added the CDC CVX/MVX vaccine identity bridge for default and EU/all
  vaccine-brand drug search, backed by local CDC bundle downloads and explicit
  refresh via `biomcp cvx sync`. (233)
- Added CDC WONDER VAERS as the aggregate vaccine adverse-event source for
  `search adverse-event --source <faers|vaers|all>`, including combined
  vaccine search, health visibility, and source inventory/docs surfaces. (235)
- Added the local GTR-backed `diagnostic` entity with `search/get diagnostic`,
  first-use bundle refresh, explicit `biomcp gtr sync`, and the supporting
  health/docs/spec contract. (236)
- Expanded `diagnostic` into a local multi-source surface with WHO Prequalified IVD
  infectious-disease products, source-aware search/get behavior, and explicit
  `biomcp who-ivd sync`. (237)
- Added FDA 510(k) and PMA regulatory overlays to
  `get diagnostic <id> regulatory`, rendering source-aware FDA device status
  when records match and a truthful empty regulatory section when they do not.
  (238)
- Added opt-in diagnostic pivots to gene and disease cards with
  `get gene <symbol> diagnostics` and `get disease <name_or_id> diagnostics`,
  backed by local GTR and WHO IVD diagnostic search. (239)
- Extended article search follow-up guidance with cross-entity keyword hints:
  JSON `_meta.next_commands` now includes gene and drug pivots for recognizable
  keyword patterns, while markdown `See also:` adds `discover` and
  `get disease` recovery paths for short disease-like phrases. (242)
- Added opt-in `clinical_features` disease output backed by MedlinePlus
  clinical summaries, reviewed HPO phenotype mapping, source-native evidence
  URLs, and unsupported-disease empty states via
  `get disease <name> clinical_features`. (252, 253)
- Added offline routing via `biomcp suggest <question>` so worked-example
  prompts can resolve to structured `_meta.workflow` guidance without requiring
  a live tool call. (279)
- Added workflow-ladder sidecars and schema-backed `_meta.ladder[]` payloads
  so routed workflows can carry explicit ladder steps across the CLI/MCP
  contract. (282)

### Docs

- Added BioASQ-oriented how-to routing rows to `skills/SKILL.md` for
  gene-disease association, gene localization/protein-function, drug regulatory
  date, and variant clinical-significance workflows, steering agents to
  structured-data shortcuts before article search. (240)
- Refreshed architecture, source-integration, and current-state docs so the
  shipped architecture and CLI guidance match the post-decomposition repo
  surface. (272, 273, 274, 275)
- Repaired backtick quoting in the mustmatch study/spec contract, published
  BioASQ benchmark guidance, and made the canonical `SKILL.md` render/install
  contract explicit for agents and operators. (276, 280, 281)
- Aligned the public landing-copy contract with the shipped `suggest` and
  workflow landing bullets so `make test-contracts` accepts the current README
  and docs homepage feature counts. (286)

### Fixes

- Added compact diagnostic rows and capped disease diagnostic pivots so
  gene/disease diagnostic follow-ups stay scannable and bounded. (266, 267)
- Replaced the stale GTR sample with a live-valid GTR example and added
  zero-result recovery for the local diagnostic surface. (268, 269)
- Tightened entity-aware article follow-ups with a same-session loop-breaker
  for overlapping suggestions and PubMed ESearch cleanup for bounded
  question-format filler words. (277, 278, 283)
- Repaired the targeted `SPEC_SMOKE_ARGS` lane so it stores stable smoke
  section IDs and resolves them to current mustmatch pytest item IDs at runtime;
  the quality ratchet now checks collectability before stale line-qualified
  selectors can reach `make spec-smoke`. (288)

### Internal

- Completed release/docs contract cleanup for the `0.8.22` line so the latest
  release notes, ticket inventory, and release metadata stay aligned. (264, 265)
- Reworked `SPEC_SMOKE_ARGS` handling around stable smoke inventory and current
  mustmatch pytest item IDs, and cleaned up `.march` artifact handling for
  repo submission paths. (270, 271)
- Recorded the neural-reranking spike as a no-go for `0.8.22`: no runtime wiring
  shipped, and the spike remains deferred from the release surface. (284)

## 0.8.21 — 2026-04-16

### New features

- Added article date-range filtering (`--date-from`/`--date-to` and
  `--year-min`/`--year-max` aliases) to `search article`. (182)
- Added EMA regulatory region (`--region eu`) to `search drug` and `get drug`,
  backed by local EMA human-medicine feeds; data auto-downloads on first use via
  `biomcp ema sync`. (203)
- Added entity-aware HATEOAS suggestions (`_meta.next_commands`) to article
  search JSON output; gene-anchored and keyword searches return contextually
  appropriate follow-up commands. (195, 202)
- Added article search fallback suggestions when `discover` resolves no entities,
  so dead-end queries get recovery paths. (204)
- Surfaced EuropePMC and PubMed `first_index_date` per article in
  `search article --json`; `null` for sources that do not expose index dates. (196)
- Expanded trial search with drug alias union so searches match trials registered
  under any approved drug name variant. (198)
- Surfaced opt-in section hints (`More:` block) in gene and disease base card
  responses; `get gene <symbol>` and `get disease <name>` now show `funding`
  and `survival` opt-in hints. (209)

### Fixes

- Fixed `search all --counts-only --json` so sections correctly omit `results`,
  `links`, and `total` fields per the counts-only contract. (206)
- Disambiguated FAERS 404 (drug not in post-marketing database) from "found but
  no matching events"; added ClinicalTrials.gov fallback for adverse-event
  lookups when FAERS returns 404. (197)
- Improved error messages for `study download` 403/404 (actionable, with
  `--list` recovery hint) and DisGeNET 403 (env var name and docs URL). (212)
- Added study subcommand help descriptions and a regression test so help
  completeness is enforced going forward. (210)

### Docs

- Refreshed architecture docs to the post-decomposition state: source map,
  modes prose, and MCP allowlist contract. (213)
- Fixed documentation drift: README source count, CHANGELOG WHO entry, and
  root help command count. (211)
- Published two blog posts on biomcp.org: daraxonrasib six-commands workflow
  article and a follow-up that removes biomcp-python references. (200, 201)

### Internal

- Added `mustmatch lint` for spec bash blocks so spec sections that silently
  skip due to a missing pipe are now caught. (221)
- Untracked `.march` build artifacts from git so worktree state stays clean. (193)
- Added Compact Approval Fields spec block for EMA and EU regulatory data
  contract assertions. (194)
- Consolidated `EnvVarGuard` and `TempDirGuard` into a central `test_support`
  module shared across all test crates. (199)
- Added docs-only validation profile so `make docs` can run without triggering
  the full build gate. (205)
- Updated `tar` and `rustls-webpki` for active RustSec advisories. (207)
- Fixed 7 `make test-contracts` failures from assertion drift after the
  decomposition batch. (208)

## 0.8.20 — 2026-03-30

### Security

- Security fix: remove partial API key from health output so shared operator
  reports no longer leak credential fragments. (067)
- Hardened the MCP allowlist so mutating `study download` paths stay blocked
  while the safe `--list` path remains available. (068)

### Fixes

- Fixed `batch --json` so responses always include the `_meta` contract field.
  (069)
- Hardened the GWAS variant section against live decode failures. (070)
- Fixed gene-to-PGx and disease-to-drug See-also routing so those pivots land
  on the correct entity helpers. (077)
- Fixed large PMC OA full-text downloads that were timing out on bigger
  payloads. (080)

### New features

- Added cross-entity See-also links for drug→PGx, gene→PGx, disease→study,
  and zero-result→discover routing. (076)
- Added the quality ratchet to `make check` so the repo gate now audits
  mustmatch lint, MCP allowlist coverage, and source-registry drift. (075)
- Added the study top-mutated helper plus disease literature fallback for
  missing direct evidence paths.
- Added SEER Explorer-backed disease survival sections so `get disease <name_or_id> survival`
  can show 5-year relative survival by sex for mapped cancers. (173)
- Added NIH Reporter funding sections so `get gene <symbol> funding` and
  `get disease <name_or_id> funding` can show top grants from the most
  recent 5 NIH fiscal years. (174)
- Added WHO Prequalification as the shipped local-data drug source for
  `get drug <name> regulatory --region who` and `search drug --region who`;
  run `who sync` to force-refresh the local CSV before freshness-sensitive
  regional drug lookups.

### Improvements

- Reworked CLI help, typed-path guidance, and the skill catalog for
  self-teaching agent navigation. (072, 073, 074)
- `get disease` now shows full disease definitions instead of truncated
  characterizations. (080)
- `get disease` now separates defining phenotypic features from the full HPO
  list with ranked key-feature summaries. (081)
- `get gene` now shows UniProt protein isoforms in the protein section. (082)
- `get drug` now shows drug target family names alongside individual targets.
  (083)
- `get gene` now shows protein legacy names alongside current symbols. (084)
- `get variant` now shows variant legacy nomenclature alongside current HGVS
  names. (085)
- Improved purine-analog mechanism labels so ATC-exact and mechanism-of-action
  coverage stay aligned. (086)
- Closed the remaining BioASQ product gaps around protein function truncation,
  variant-specific targets, and disease feature coverage. (087)
- Replaced the in-shell mustmatch shim with the packaged `mustmatch lint`
  entrypoint plus a small compatibility layer for the repo’s existing ratchet
  contract. (088)

### Docs

- Added architecture contract docs for See-also links, discover grammar, CI,
  and post-tag public proof expectations. (078)
- Documentation consistency audit across the public docs surface.
- Repo cleanup for layout, example paths, and tracked `.march` artifacts.
  (079)

## 0.8.19 — 2026-03-26

### Improvements

- Cleaned up Semantic Scholar health output so unauthenticated setups are
  reported as optional/shared-pool usage instead of looking like a broken
  configuration. (055)
- Added six more built-in study chart types: pie, density, violin,
  ridgeline, heatmap, and stacked bar. (041)
- EMA-backed drug data now auto-downloads on first use and refreshes stale
  local caches, so EU drug coverage no longer requires manual setup. (057)

### CI

- Moved EMA-backed specs out of the PR-blocking stable lane into the smoke
  lane so CI keeps EMA coverage without destabilizing the release gate. (056)

### Docs

- Refreshed the README, docs index, and source landing pages with clearer
  public-facing messaging.
- Added three new blog posts covering agent skills, article search, and
  built-in cancer genomics charts.

## 0.8.18 — 2026-03-25

### New sources

- Added EMA regional drug coverage backed by local EMA human-medicine feeds,
  so `search drug`, `get drug`, and `get drug shortage` support
  `--region us|eu|all` across EU regulatory status, DHPCs, referrals, PSUSAs,
  shortages, and the existing US label surface. (043, 044)
- Added the BioASQ benchmark module with `benchmarks/bioasq/ingest_public.py`,
  dataset and validity manifests, and the published BioASQ benchmark
  reference at `docs/reference/bioasq-benchmark.md`. (045, 046)

### Improvements

- `search drug` now defaults to `--region all`, `get drug --region` is
  surfaced in help, OpenFDA label fallback prefers exact brand-name matches,
  and OpenTargets drug-indication lookups are corrected for mixed US/EU
  workflows. (043, 044)
- `S2_API_KEY` is now optional for Semantic Scholar-backed article
  enrichment and helpers: unauthenticated calls use the shared Semantic
  Scholar pool at 1 req/2sec, while authenticated calls keep the 1 req/sec
  quota with retry backoff. (047)

## 0.8.17 — 2026-03-23

### New sources

- Added WikiPathways as a third pathway source alongside Reactome and KEGG,
  with source-aware section handling plus operator proof coverage in
  `biomcp health` and `scripts/contract-smoke.sh`. (025)
- Added an optional Semantic Scholar search leg to `search article` when
  `S2_API_KEY` is set, merging PubTator3, Europe PMC, and Semantic Scholar
  results with identifier-aware deduplication and directness-first ranking.
  (034)
- Added KEGG as a pathway source alongside Reactome and WikiPathways.
- Added gnomAD constraint metrics for variant-interpretation context.
- Added Human Protein Atlas tissue expression to gene output.
- Added ComplexPortal-backed protein complex data.
- Added DisGeNET gene-disease association scores.

### New commands

- Added `biomcp discover <query>` as the free-text concept-resolution
  entrypoint, using OLS4 as the required backbone with optional UMLS
  crosswalks and MedlinePlus disease/symptom context. (028)
- Added `biomcp article batch <id>...` for compact multi-article summary
  cards with parallel article fetch and optional batched Semantic Scholar
  enrichment; the command is limited to 20 IDs. (037)

### Improvements

- Restored pathway progressive disclosure and exact-title-first ranking, so
  default pathway cards stay concise and exact pathway-title matches rise to
  the top across sources. (017)
- Expanded operator proof surfaces and refreshed source-aware architecture docs
  so the shipped source inventory, API-key handling, proof expectations, and
  section-availability rules are explicit and consistent. (018, 019, 030)
- Improved protein complexes terminal layout and normalized pathway CLI usage
  errors and remediation. (020, 021)
- Added source labels and missing evidence URLs across entity detail outputs in
  Markdown and JSON for stronger traceability. (022, 025)
- Added paper/reproducibility and community-packaging artifacts, including
  `paper/`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, and `.zenodo.json`. (023)
- `get gene` and `get drug` misses now return discovery-backed canonical
  suggestions with structured `_meta.alias_resolution` /
  `_meta.next_commands` guidance instead of dead-end errors. (029)
- Help, `list`, and docs now surface working typed paths, and `search all` /
  `search article` can emit `--debug-plan` routing metadata in Markdown or
  JSON when requested. (032)
- Typed variant retrieval now accepts common shorthand forms, emits
  variant-scoped recovery guidance on misses, and accepts long-form protein
  notation such as `AKT2 p.Pro50Thr`. (033, 039)
- `search all` cross-routing now stays inside controlled typed fallback rules,
  reducing noisy drill-downs and duplicate follow-up commands. (035)
- Typed retrievals now expose compact answer-bearing summaries for approvals,
  population frequencies, and disease/variant evidence without removing the
  detailed sections. (036)
- Deepened OpenTargets integration with tractability and
  genetic-association sections.
- Added JATS-aware PMC full-text extraction for article detail views.
- MCP chart responses can now return SVG inline as a base64 image payload.
- Polished Kuva chart output with human-readable labels and KM curve
  improvements.
- Suppressed retry-middleware warnings from stderr.
- Pathway output now keeps section availability truthful and gives better
  guidance when a source lacks a section.

### Fixes

- Fixed drug interaction output so supported drugs no longer return empty
  interaction sections.
- Fixed g:Profiler enrichment timeouts.
- Fixed the DisGeNET template crash on sparse or null payloads such as
  `biomcp get gene KYNU disgenet`. (031)
- Fixed default article retraction filtering so only confirmed retractions are
  excluded; PubTator3 and Semantic Scholar rows with unknown retraction status
  remain visible. (038)

## 0.8.16 — 2026-03-17

- Adopted the shared `bin/lint` script and wired `make lint` / `make check`
  to the March repo convention for Rust repos.
- Refreshed the README, homepage, CLI reference, and helper docs so the public
  release docs match the current BioMCP command surface.
- Expanded `skills/SKILL.md` for article graph helpers, `enrich`, `batch`,
  chart discovery, chart flags, and updated drug interaction guidance.
- Updated the Kuva chart blog post with checked-in SVG outputs for all nine
  worked examples plus placeholder references for terminal screenshots.
- Added Semantic Scholar article enrichment and helpers for TLDR, citations,
  references, and recommendations.
- Added Kuva-backed study chart rendering plus chart discovery/docs for SVG
  study outputs.
- Trial search now accepts fractional ages such as `0.5 years`.
- Generated `_meta.next_commands` now pass a parser-level validity gate, and
  article JSON guidance is cleaner and more truthful.
- Fixed federated article search offset and sort semantics with correctness
  coverage.
- Breaking MCP runtime change: renamed the MCP execution tool from `shell` to
  `biomcp`; update MCP clients and demos to call `biomcp` after `tools/list`.
- Added `CITATION.cff` and citation pointers in the README and public docs.
- Fixed Semantic Scholar references to use cited papers instead of referenced
  papers.
- Fixed article search timeouts on `--exclude-retracted`.
- Fixed article retraction filtering and unary `NOT` parsing.
- Stabilized CTGov trial pagination and age-filtered totals: improved cursor
  behavior, aligned count/search params, clarified trial-filter guidance, and
  exposed approximate age-only counts when exact totals are too expensive.
- Expanded release-quality gates with contracts CI, version-sync checks, a
  PR-gated spec suite, stable/smoke lane splitting, and protein entity spec
  coverage.
- Updated lockfiles for the `quinn-proto` and `PyJWT` vulnerability
  advisories.
- Refined the Streamable HTTP demo into a tested end-to-end BRAF/melanoma
  workflow with a focused README and Python 3.11 compatibility.

## 0.8.15 — 2026-03-11

- Fixed the planning-docs CI path regression so release validation uses the
  repo-local planning fixtures by default instead of an Ian-local absolute
  path. This is the fix from PR #191 that unblocks release packaging on
  GitHub Actions.
- Refreshed the public discovery docs so `search all` is taught as the unified
  cross-entity entry point in the README and docs index. This is the docs
  alignment from PR #190.

## 0.8.14 — 2026-03-10

- Promoted remote Streamable HTTP in newcomer docs with a dedicated getting-started page for `biomcp serve-http`.
- Added a runnable PEP 723 demo client at `demo/streamable_http_client.py` for `/mcp`, including a `tools/list` flow and `biomcp version` tool call.
- Surfaced the canonical remote HTTP routes more prominently across public docs: `/mcp`, `/health`, `/readyz`, and `/`.
- Kept `serve-sse` migration guidance visible while shipping the Streamable HTTP release/docs/demo verification package together.
- Reranked disease search results so exact-match canonical names surface first.
- `search article` now rejects unsupported identifiers with explicit contract guidance instead of falling through ambiguously.
- Defaulted article search sorting to relevance across CLI entry points.
- Normalized search aliases, identifiers, and trial phase semantics across entity searches.
- Broadened trial mutation matching across title, summary, keyword, and eligibility fields.
- Removed stale skill-discovery UX and refreshed `list` / help output to match the shipped CLI surface.

## 0.8.13 — 2026-03-09

- `study survival` now reports Kaplan-Meier median survival, 1/3/5-year survival landmarks, and two-group log-rank p-values.
- Fixed survival median calculation: replaced raw follow-up median with Kaplan-Meier median survival.
- `study compare --type expression` now reports Mann-Whitney U and p-value.
- Added `study download <study_id>` and `study download --list` for local cBioPortal datahub installs.
- Hardened `study download`: no total body timeout for large archives, path-like study ID rejection, stream-to-disk download, and cleanup on failed extraction.
- Refreshed `skills/SKILL.md` with real study command usage, including `cohort`, `filter`, and `survival --endpoint` examples.

## 0.8.12 — 2026-03-07

- Added `study` subcommand: local cBioPortal study adapters with CLI query surface (`download`, `cohort`, `survival`, `compare`).
- Added `study filter`: multi-omics cross-table sample joins across mutation, CNA, expression, and clinical data.
- Added Fisher exact p-values to co-occurrence pair analysis.
- Added skill installer auto-discovery: scans existing agent config directories in priority order and supports the `.agents/skills/` cross-tool standard.
- Refreshed skill guidance and reference material: adopted the measured `SKILL.md` refresh and added drift validation for examples, schemas, and jq snippets.
- CLI quality pass: positional search arguments, multi-token input handling, variant fallback, NCT ID validation, bare-entity shortcuts, staged trial fill, article request timeout, and `--until` alias.

## 0.8.11 — 2026-03-06

- Added `expression` section to gene output (GTEx tissue-specific TPM data).
- Added `druggability` section to gene output (DGIdb drug-gene interactions and categories).
- Added `clingen` section to gene output (gene-disease validity and dosage sensitivity).
- Added evidence URLs (`_meta.evidence_urls`) to all entity output — includes Ensembl, OMIM, NCBI Gene, and UniProt links where available.
- Added `spec/` BDD documentation suite with 54 passing executable specifications.
- Unified article search: fan-out across PubTator3 and Europe PMC in parallel with PMID deduplication, source-grouped rendering, and `--source <all|pubtator|europepmc>`.
- Added infinite cache mode via `BIOMCP_CACHE_MODE=infinite` for offline/demo workflows.
- Consolidated CI into a single job; streamlined release pipeline.
- Added reusable presentations infrastructure with an intro deck, branded theme assets, and slide templates for BioMCP talks.
- Hardened PyPI release packaging for arm64 and portable version syncing in the release workflow.

## 0.8.10 — 2026-03-04

- Polished `search all` output: GWAS trait relevance filtering, clinical significance sorting, uninformative variant suppression.
- Gene-anchored article search via `GENE_PROTEIN` field for higher precision.
- Added trial search improvements (`--mutation` boolean handling and `--criteria` support).
- Added PyPI trusted publisher setup and release workflow updates.

## 0.8.9 — 2026-03-03

- Added `search all` cross-entity command — parallel fan-out across major entities (genes, variants, diseases, drugs, trials, articles, pathways, PGx, GWAS, and adverse events) with counts-first display and HATEOAS deep-dive links.

## 0.8.8 — 2026-03-02

- Output cleanup across entity renderers for consistent markdown formatting.
- Improved help discoverability and completeness across all subcommands.
- Rewrote footer output with streamlined next-step suggestions.

## 0.8.7 — 2026-02-27

- Improved help text discoverability and completeness for all CLI subcommands.

## 0.8.6 — 2026-02-27

- Fixed trial filter accuracy: age-based post-filtering, exclusion-criteria detection, and pathway source correctness.

## 0.8.5 — 2026-02-26

- Fixed `install.sh` to resolve `latest` tag to a release that has downloadable assets.
- Hardened input safety with SSRF prevention on user-supplied URLs.
- Enforced pagination and validation contracts across search endpoints.

## 0.8.4

- Simplified CI/CD workflow triggers to avoid redundant post-merge runs.
- Added runner-aware `actions/cache@v4` protoc caching in CI, docs deploy, and release workflows.
- Pinned workflow protoc installation to `28.3` with cache-aware conditional setup.

## 0.8.3

- Added inclusion-first eligibility post-filtering to reduce exclusion-only false positives in trial matching.
- Applied cryptography dependency security updates and refreshed release artifacts.
- Tightened CLI/docs consistency around trial filter behavior and output contracts.

## 0.8.2

- Added location-level facility and geo post-filtering for more accurate trial site matching.
- Embedded git tag version metadata in the binary for reliable `biomcp version` output.
- Improved pagination and result-window handling across search command families.

## 0.8.1

- Expanded clinical-grade trial filtering with ~45 new search flags and validation rules.
- Added MCP tool auto-generation to reduce drift between CLI capabilities and server tool surface.
- Upgraded help/discoverability content and list-based guidance for agent-driven workflows.

## 0.8.0

Complete rewrite from Python to Rust. Single static binary, no runtime dependencies.

### Highlights

- Single-binary CLI and MCP server — no Python, no pip, no virtual environments
- 15 biomedical data sources, unified command grammar, compact markdown output
- 14 embedded skills (guided investigation workflows)
- MCP server (stdio + SSE) with tool and resource support
- HTTP proxy (`serve-http`) for multi-worker shared rate limiting
- Production installer with SHA256 verification (5 platforms)
- Progressive disclosure: search returns summaries, get returns full detail with selectable sections
- NCBI API key support for improved rate limits
- 429 retry with Retry-After backoff

### Data sources

MyGene.info, MyVariant.info (ClinVar, gnomAD, CIViC, OncoKB), ClinicalTrials.gov,
NCI CTS API, PubMed/PubTator3, MyChem.info, Monarch/MONDO, Reactome, UniProt,
OpenFDA FAERS, PharmGKB/CPIC, GWAS Catalog, Monarch (phenotypes), AlphaGenome (gRPC).

### Breaking changes from Python BioMCP

- Python package `biomcp-python` is no longer maintained
- MCP tool names and signatures have changed
- Configuration via environment variables only

## 0.7.3

Legacy Python BioMCP. See branch `python-0.7.3` for source code.
