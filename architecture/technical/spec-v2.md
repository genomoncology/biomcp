# BioMCP spec-v2 target state

Ticket 297 freezes the replacement architecture for the executable spec corpus.
The current `spec/*.md` tree is structurally wrong for the shipped surface: it
mixes entity contracts with tutorials and junk drawers, repeats env-isolation
boilerplate in every bash block, depends on cold live APIs in CI, and pins
copy-edit trivia instead of durable behavior. The target state keeps the parts
that work (CLI/MCP/user-facing contracts) and replaces only the broken corpus
shape, execution seam, and gate mechanics.

## Current problems to preserve in writing

The survey identified six root-cause issues that this target architecture must
eliminate without changing BioMCP's shipped product surface:

1. **No env-isolation seam** - spec blocks reimplement `BIOMCP_BIN`, `XDG_*`,
   and API-key handling independently.
2. **Cold live-API CI gate** - `spec-stable` restores no HTTP cache and reruns
   a large live-network suite from a fresh checkout every PR.
3. **Wrong unit of organization** - numbered files mix entity behavior,
   tutorial prose, cross-entity pivots, and rendering trivia.
4. **Trivia pinning** - exact counts, exact prose, and exact positions fail on
   copy edits rather than runtime regressions.
5. **Heading/node-ID coupling** - `SPEC_PR_DESELECT_ARGS`, `SPEC_SMOKE_ARGS`,
   and `spec/conftest.py` depend on exact current file names and headings.
6. **Serial fixture coupling** - drug/study/cross-entity specs share mutable
   repo-local fixture state and force an extra serial pytest leg.

## Decisions already made

These decisions come from the 2026-04-24 deep-dive and ticket 297 and are not
up for redesign in this document:

1. Hard-delete the current numbered corpus once slice 1 lands; Git is the
   archive.
2. Replace it with **13 entity files** under `spec/entity/` and **3 surface
   files** under `spec/surface/`.
3. Keep each file at **3-7 semantic assertions**; **12 is the hard cap** for
   dense entities.
4. Introduce `tools/biomcp-ci` as the only spec invocation seam.
5. Use the existing BioMCP disk cache plus CI cache restore as the replay
   mechanism; no committed tarball, no separate cassette format, no nightly
   record job.
6. Keep the PR gate budget at **<=5 minutes warm** and allow a **<=15 minute
   cold** first run per release key.
7. Fold `discover`, `suggest`, and `skill` into one `spec/surface/discover.md`.
8. Keep `ema`, `who`, `cvx`, `gtr`, and `who-ivd` out of entity coverage; they
   get one `--help` assertion each in `spec/surface/cli.md`.

## Target architecture

### Layout and ownership

| Path | Owns | Must not own |
|---|---|---|
| `spec/entity/{gene,variant,article,trial,drug,disease,protein,pathway,study,pgx,phenotype,diagnostic,vaers}.md` | Search/get behavior, progressive disclosure, typed pivots, truthful error/fallback behavior for one entity | Global CLI envelope, MCP transport, installer/admin sync flows |
| `spec/surface/cli.md` | `--help`, `--version`, `--json`, `list`, `health`, `batch`, `enrich`, cache commands, admin-command help | Entity-specific data assertions, MCP transport |
| `spec/surface/mcp.md` | `serve`, `mcp`, `serve-http`, `/mcp`, `/health`, `/readyz`, `/`, read-only MCP boundaries | CLI-local cache/admin commands |
| `spec/surface/discover.md` | Free-text concept resolution, skill routing, no-match behavior, ambiguity handling | Entity detail cards, transport/runtime handshakes |

### Invariants

The new corpus enforces these invariants:

1. **One behavioral ownership zone per file.**
2. **No hand-rolled env isolation inside spec blocks.**
3. **No required live API keys in the PR gate.**
4. **No exact counts, exact prose, or line-qualified node IDs in shipped
   assertions.**
5. **No `SPEC_PR_DESELECT_ARGS` or `SPEC_SERIAL_FILES` dependence on the old
   corpus shape.**
6. **Every migrated ticket leaves `make check` healthy and the active spec lane
   executable.**

## Per-file outlines

Each file below names the section headings the rewrite should land. The headings
are intentionally semantic: they describe the behavior the block proves, not the
literal `mustmatch` bytes.

### `spec/entity/gene.md`

1. **Symbol-Based Search** - canonical HGNC symbol search returns ranked rows
   with stable identity columns.
2. **Search Table Contract** - markdown/JSON search shape stays stable enough
   for human scanning and `_meta.next_commands`.
3. **Identity Card** - `get gene <symbol>` returns persistent identifiers,
   summary text, and progressive-disclosure guidance.
4. **Tissue-Expression Context** - HPA-backed tissue/subcellular context appears
   only when requested.
5. **Druggability & Targets** - actionable targetability data stays available
   without flooding the default card.
6. **Funding & Diagnostics Cross-Pivot** - NIH funding and GTR diagnostics stay
   discoverable as opt-in deepen paths.

### `spec/entity/variant.md`

1. **Gene-Scoped Variant Search** - search by gene returns canonical variant
   identity with legacy-name fallback.
2. **Search Table Contract** - results expose stable variant identity columns
   and query echo.
3. **Protein-Filter Narrowing** - short- and long-form protein filters normalize
   to one canonical mutation spelling.
4. **Residue-Alias Search** - residue phrases route through alias resolution,
   not free-text fallback.
5. **Clinical Significance & Frequency** - ClinVar and population-frequency
   sections stay opt-in and truthful.
6. **Variant-to-Trial & Article Pivots** - variant context exposes trial and
   literature follow-ups.
7. **ID Normalization** - rsID and c.HGVS inputs resolve to the same canonical
   record without losing user intent.

### `spec/entity/article.md`

1. **Gene & Keyword Search** - typed and free-text article search keep their
   own planning and routing semantics.
2. **Search Table & Source Ranking** - federated article results preserve source
   identity and honest totals.
3. **PubTator Annotations** - entity extraction remains available as a
   first-class article deepen path.
4. **Full-Text Access & PDF Fallback** - fulltext is opt-in and PDF fallback is
   only legal with the fulltext path.
5. **Semantic Scholar Graph** - TLDR, citations, references, and
   recommendations degrade truthfully when the optional key is absent.
6. **Batch & Entity Extraction** - article batch/entity helpers stay available
   as compact multi-record follow-ups.

### `spec/entity/trial.md`

1. **Condition-First Search** - disease/condition search returns stable trial
   rows with the expected identity columns.
2. **Status & Phase Filters** - recruitment and phase filters narrow results
   deterministically.
3. **Mutation-Centric Search** - mutation and drug alias paths normalize
   consistently in search and help surfaces.
4. **Age & Enrollment Constraints** - age filtering keeps its bounded-count
   behavior and transparent guidance.
5. **Trial Detail & Eligibility** - detail cards expose eligibility, locations,
   and NCI-specific help semantics.

### `spec/entity/drug.md`

1. **Multi-Region Search** - FDA, EMA, and WHO drug search surfaces share one
   durable contract with honest region handling.
2. **Brand-Name Bridge** - brand names route to the expected generic/vaccine
   identity without silent misses.
3. **Indication Structured Search** - structured indication search stays
   truthful on sparse or empty evidence.
4. **Mechanism & Targets** - target/mechanism data remains first-class and flat
   enough for CLI consumption.
5. **Regulatory Approval Timeline** - approval/regulatory summaries remain
   human-readable and opt-in deep sections stay reachable.
6. **Trial & Adverse-Event Pivots** - trial and safety follow-ups stay exposed
   from drug context.
7. **Health-Readiness Indicators** - local EMA/WHO readiness remains operator
   visible through `health`, not entity defaults.

### `spec/entity/disease.md`

1. **Disease Normalization & Search** - ontology-backed disease search returns
   MONDO-driven identity with synonym handling.
2. **Canonical Disease Card** - default disease cards expose persistent
   identifiers, definitions, and deepen hints.
3. **Survival & Epidemiology** - SEER-backed survival stays truthful on missing
   or inapplicable data.
4. **NIH Funding Context** - NIH funding remains opt-in and does not pollute the
   default summary.
5. **Disease-to-Gene & Phenotype** - genes and phenotype associations stay
   discoverable as deepen paths.
6. **Diagnostic & Treatment Pivots** - disease context keeps explicit pivots to
   diagnostics, trials, articles, and drugs.

### `spec/entity/protein.md`

1. **Positional Search & Table** - protein search keeps its positional argument
   contract and stable result columns.
2. **UniProt Identity Card** - protein detail cards preserve identity, summary,
   and evidence URLs.
3. **Protein Complexes** - complexes remain bounded, truthful, and free of table
   bloat.
4. **Evidence URLs & JSON Contract** - provenance metadata remains stable in the
   JSON path.
5. **Deepen Commands** - protein output keeps explicit typed follow-up commands.

### `spec/entity/pathway.md`

1. **Long-Form Alias Normalization** - supported long-form pathway aliases
   normalize before query execution.
2. **Query-Required Contract** - empty-query behavior stays explicit and
   recoverable.
3. **Exact-Title Ranking** - exact title matches rank first across KEGG and
   WikiPathways.
4. **Concise KEGG Default** - default cards stay summary-first while explicit
   gene sections remain reachable.
5. **Unsupported Section Rejection** - unsupported sections fail cleanly with
   guidance instead of blank success.

### `spec/entity/study.md`

1. **Local Study Discovery** - `study list` makes the local cBioPortal catalog
   contract explicit.
2. **Gene-Frequency Queries** - mutation/CNA/expression queries require and
   validate the expected study/gene inputs.
3. **Multi-Omics Filtering** - filter workflows stay explicit about missing data
   sources and intersected constraints.
4. **Survival & Cohort Analysis** - cohort/survival commands keep their
   split-by-status contract.
5. **Expression & Co-Occurrence Comparison** - comparison and co-occurrence
   summaries stay available without hidden data-shape changes.
6. **Chart Export & Visualization** - charted study calls keep their terminal
   and SVG/MCP response contract.

### `spec/entity/pgx.md`

1. **Gene & Drug Search** - pharmacogene and drug entrypoints return CPIC-style
   interaction rows.
2. **CPIC Evidence Levels** - evidence-level filters remain first-class and
   documented.
3. **Drug-Gene Recommendations** - dosing/recommendation sections stay opt-in
   and typed.
4. **Population Allele Frequencies** - population frequency data stays truthful
   on sparse coverage.

### `spec/entity/phenotype.md`

1. **HPO & Symptom-Phrase Input** - HPO IDs and symptom text resolve through one
   transparent phenotype path.
2. **Monarch Similarity Ranking** - disease ranking remains similarity-based and
   explicitly typed.
3. **Symptom Text Resolution** - symptom normalization stays visible enough for
   users to understand the follow-up path.

### `spec/entity/diagnostic.md`

1. **Multi-Source Search** - default diagnostic search merges GTR and WHO IVD
   results with per-row provenance.
2. **Source-Native Filters** - supported filters differ by source and fail fast
   when users ask for unsupported combinations.
3. **Gene-First GTR Workflows** - gene-centric diagnostic search remains a GTR
   path, not a WHO path.
4. **WHO IVD Local Data** - WHO IVD readiness stays explicit and `who-ivd sync`
   remains CLI-only.
5. **Deterministic Deduplication** - dedupe and row compaction preserve readable
   output without losing source intent.
6. **FDA Regulatory Overlay** - the GTR/FDA overlay stays opt-in and truthful.

### `spec/entity/vaers.md`

1. **Vaccine-First Search** - vaccine queries normalize to the expected VAERS
   aggregation path.
2. **Aggregate-Only Reporting** - VAERS remains explicitly aggregate-only.
3. **Source-Specific Limitations** - unsupported FAERS-style filters degrade
   truthfully rather than silently disappearing.
4. **Combined Default & Source Selection** - combined FAERS+VAERS behavior stays
   explicit and scoped by `--source`.

### `spec/surface/cli.md`

1. **Search & Get Envelope** - core command grammar (`search`, `get`, sections,
   `--json`, `--no-cache`) remains stable.
2. **Workflow Metadata** - `_meta.workflow`, `_meta.ladder[]`, and
   `_meta.next_commands` keep their distinct roles.
3. **Evidence & Attribution** - `_meta.evidence_urls` and `_meta.section_sources`
   remain first-class auditability surfaces.
4. **Installer/Admin Help** - `ema`, `who`, `cvx`, `gtr`, and `who-ivd` prove
   `--help` only; they stay CLI-only installer utilities.
5. **Cache & Operator Commands** - cache-family commands and `health` stay
   explicit CLI-only operational surfaces.

### `spec/surface/mcp.md`

1. **Stdio & Streamable HTTP Entry Points** - `serve`, `mcp`, and `serve-http`
   remain the authoritative MCP entrypoints.
2. **Health & Readiness Probes** - `/health`, `/readyz`, `/`, and `/mcp` keep
   their stable route meanings.
3. **Tool Capability Advertising** - the MCP shell keeps tools/resources
   enabled and the description read-only.
4. **Response Content Contract** - charted study calls keep the text+SVG
   multi-block contract; other calls stay text-only.
5. **Read-Only Boundaries** - sync/admin/cache/mutating study flows stay blocked
   over MCP.

### `spec/surface/discover.md`

1. **Free-Text Entry Point** - `discover` resolves open text into typed BioMCP
   follow-ups.
2. **Entity Alias Resolution** - gene aliases, drug brands, and symptom concepts
   normalize into typed command stubs.
3. **Symptom-Safe Suggestions** - phenotype/disease symptom routing stays
   truthful and clinically modest.
4. **Ambiguous Query Guidance** - ambiguous input produces explicit
   disambiguation hints rather than silent one-path guesses.
5. **Suggest & Skill Routing** - `suggest` and `skill` stay part of one
   agent-onboarding surface with a durable JSON shape.

## `tools/biomcp-ci` wrapper contract

`tools/biomcp-ci` is the single execution seam for every v2 spec bash block.
The wrapper is intentionally plain shell; it is test infrastructure, not a new
product command.

### Invocation

- Form: `tools/biomcp-ci <biomcp args...>`
- Argument policy: pass `"$@"` through unchanged.
- Binary resolution: use `BIOMCP_BIN` when set; otherwise fail closed. The
  wrapper may resolve `biomcp` from PATH only for diagnostic context, must
  refuse to execute from PATH, and names the rejected PATH candidate when one
  exists.

### Environment handling

The wrapper owns these environment decisions:

1. **Unset auth/rate-limit keys** so the PR gate proves the public,
   unauthenticated surface:
   - `NCBI_API_KEY`
   - `S2_API_KEY`
   - `OPENFDA_API_KEY`
   - `NCI_API_KEY`
   - `ONCOKB_TOKEN`
   - `DISGENET_API_KEY`
   - `ALPHAGENOME_API_KEY`
   - `UMLS_API_KEY`
2. **Pin the managed BioMCP cache root** with
   `BIOMCP_CACHE_DIR="$REPO/.cache/biomcp-specs"`.
3. **Redirect XDG config/cache side effects** into the same workspace-owned
   tree:
   - `XDG_CACHE_HOME="$REPO/.cache/biomcp-specs/xdg-cache"`
   - `XDG_CONFIG_HOME="$REPO/.cache/biomcp-specs/config"`
4. **Default logging** to `RUST_LOG=error` unless the caller already set
   `RUST_LOG`.
5. **Warm-cache replay mode** is conditional:
   - if the caller exports `BIOMCP_SPEC_CACHE_HIT=1` and `BIOMCP_CACHE_MODE` is
     unset, the wrapper sets `BIOMCP_CACHE_MODE=infinite`;
   - otherwise it leaves `BIOMCP_CACHE_MODE` unchanged so a cold miss can refill
     the cache instead of failing the run.

### Failure modes

- Missing explicit `BIOMCP_BIN` target: wrapper exits with the underlying `exec`
  failure status and preserves stderr.
- Unset or empty `BIOMCP_BIN`: wrapper fails closed before invoking `biomcp`;
  any `PATH` candidate is rejected diagnostic context only.
- Directory creation failure: wrapper exits nonzero before invoking `biomcp`.
- Warm-cache replay miss (`BIOMCP_SPEC_CACHE_HIT=1` + `BIOMCP_CACHE_MODE=infinite`):
  BioMCP fails loudly on the cache miss; this is the desired signal that the CI
  cache key or schema version is wrong.
- Explicit `BIOMCP_BIN` output and exit behavior: stdout, stderr, and exit code
  pass through exactly.

## Cache-warm gate design

BioMCP already resolves cache roots through `resolve_cache_config()` and the
shared HTTP client already honors `BIOMCP_CACHE_MODE`. The spec rewrite should
reuse that mechanism rather than inventing cassette files.

### Cache root

- Canonical spec cache path: `.cache/biomcp-specs/`
- BioMCP cache root inside that path: `.cache/biomcp-specs/http`
- CI cache restore path: the entire `.cache/biomcp-specs/` tree

### Cache key

The CI key must change when cached responses are expected to be incompatible
with the rewritten corpus:

```text
spec-http-${runner.os}-${biomcp-version}-${spec-cache-schema-version}
```

- `biomcp-version`: parse from `Cargo.toml` / the built release binary version.
- `spec-cache-schema-version`: one explicit constant introduced in slice 1 and
  bumped when the wrapper contract, cache layout, or chosen canary accessions
  change incompatibly.

### CI behavior

1. Restore `.cache/biomcp-specs/` with `actions/cache`.
2. If the cache step reports a hit, export `BIOMCP_SPEC_CACHE_HIT=1` so
   `biomcp-ci` switches to `BIOMCP_CACHE_MODE=infinite`.
3. On a miss, do **not** force infinite mode; accept the cold run and let the
   job repopulate the cache under the new key.
4. Release-tag CI naturally refreshes the cache because the version component of
   the key changes on each tag.

### Why this replaces ticket 296

The PR gate no longer needs a second HTTP replay system. It needs:

1. one invocation seam,
2. one workspace-owned cache root,
3. one CI cache key,
4. a strict replay mode only when CI already restored a known-good cache.

That achieves deterministic warm runs without committed tarballs or a second
HTTP middleware layer.

## Assertion contract

The v2 corpus adopts the build2 prompt text directly.

### Assertion classes

> - **Semantic** — names user-visible behavior; fails on real regression.
>   Example:
>   `biomcp get gene BRAF | mustmatch like "Entrez ID: 673"`.
> - **Structural** — pins shape without exact counts of incidentals.
>   Example: `assert len(bullets) >= 4`; presence of named labels.
> - **Trivia — banned.** Exact counts (`== 6`), exact prose without a
>   regression target, line-qualified node IDs (`spec/x.md#L42`),
>   occurrence counts of incidental tokens. Breaks on copy edits,
>   catches no regressions.

Source: `planning/flows/build2/01-design.md` (`## Proof Matrix`) and identical
classification text in `planning/flows/build2/02-design-review.md`
(`## Phase 1 - Critique`).

### 5-question rejection rubric

> 1. What real user-visible regression would this catch that a looser
>    assertion wouldn't?
> 2. If the feature were silently broken, would this fail?
> 3. Does it name a behavior, or pin incidental output?
> 4. Would it survive a one-word copy edit?
> 5. Could `contains X` replace `exactly N of Z` or `exactly this
>    phrase`?
>
> If 1-2 have no answer -> trivia. If 5 honestly answers "yes, looser
> works" -> use the looser form.

Source: `planning/flows/build2/01-design.md` (`## Proof Matrix`) and
`planning/flows/build2/02-design-review.md` (`Apply the 5-question rejection
rubric`).

### Rubric sharpening needed for locally-backed entities

Two areas need explicit care when design authors write literal assertions:

1. **`study.md`** - local cBioPortal analytics are semantic, but exact row
   counts over local installs are not. Assertions should pin required failure
   messages, chart/summary labels, and required columns rather than downloaded
   row totals.
2. **`diagnostic.md`** - WHO IVD and GTR local-data footprints are semantic, but
   exact CSV or sync inventory counts are not. Assertions should pin readiness
   status, supported filter grammar, and truthful unsupported-section behavior.

## Blast-radius validation

The survey's dependency map is preserved and simplified as follows:

| Current dependency | v2 replacement | Why this is safe |
|---|---|---|
| `spec/*.md` numbered files | `spec/entity/` + `spec/surface/` | Keeps user-visible entities/surfaces, removes only numbering and junk drawers |
| Ad hoc `BIOMCP_BIN` / `XDG_*` / key handling per block | `tools/biomcp-ci` | Centralizes one contract without changing runtime behavior |
| `SPEC_PR_DESELECT_ARGS` / `SPEC_SMOKE_ARGS` tied to old headings | Shrunk corpus with semantic blocks only; no permanent deselect list target | Removes heading-coupled debt instead of renaming around it |
| `spec/conftest.py` keyed skip-node sets | Minimal or empty v2 conftest; no key-required PR-gate assertions | Public gate no longer depends on optional auth keys |
| `SPEC_SERIAL_FILES` and fixture setup scripts | No local-admin entity coverage in the active corpus | Eliminates shared mutable fixture state from the PR gate |
| `spec-stable` cold cache | Workflow cache restore keyed by version + schema | Preserves the same runtime HTTP client while stabilizing CI |

## Runtime budget

The outline above plans **85 sections** across 16 files. Budget against that
worst-case count instead of the aspirational "~80" average.

### Serial budget math

| Scenario | Assumption | Math | Result |
|---|---|---|---|
| Warm cache | 85 blocks x 2.5s/block | 85 x 2.5 | 212.5s (~3m33s) |
| Cold cache | 85 blocks x 10s/block | 85 x 10 | 850s (~14m10s) |

### Why these assumptions are acceptable

- **2.5s warm** is intentionally conservative: it includes release-binary
  startup, one cached BioMCP call, and `mustmatch` overhead.
- **10s cold** is intentionally conservative: it budgets one live upstream fanout
  plus parse/render time per block.
- `pytest-xdist -n auto --dist loadfile` only reduces elapsed wall-clock from
  these serial bounds; it is not required to make the warm budget work.

That keeps the **warm gate under five minutes** and the **cold miss path under
fifteen minutes** even before parallelism helps.

## Migration sequence

The target is reachable through four shippable slices. Each leaves the repo
working and keeps the active gate meaningful.

| Slice | Scope | Intermediate state after merge | Gate impact |
|---|---|---|---|
| 1 | Add `tools/biomcp-ci`, add CI cache restore/key/schema constant, delete the old corpus, land `gene.md`, `variant.md`, `article.md`, repoint `make spec-pr` to `spec/entity/` + `spec/surface/` | New layout exists, old numbered corpus is gone, three canary entity files prove the template, cache-warm path exists | PR gate runs only the canary v2 corpus |
| 2 | Land entity batch A: `trial.md`, `drug.md`, `disease.md`, `protein.md` | Core clinical entity coverage is in v2 format; no old entity files remain | Gate expands without reintroducing old fixture or deselect debt |
| 3 | Land entity batch B: `pathway.md`, `study.md`, `pgx.md`, `phenotype.md`, `diagnostic.md`, `vaers.md` | All 13 entities are on the v2 corpus | Gate covers the full entity surface in the new layout |
| 4 | Land `cli.md`, `mcp.md`, `discover.md`, remove any residual old-spec glue, and keep the already-shipped `make check`/`test-contracts` integration green while the final v2 corpus comes online | Full 16-file v2 corpus is active; stable PR gate and canonical `make check` gate align | Preserves 287 and makes the older spec tickets obsolete |

### Build-flow note

The intended discipline for slices 2-4 is **build2**, but BioMCP does **not**
currently declare `[profile.spec-only]` in `.march/validation-profiles.toml`.
Slice 1 must add that repo opt-in alongside the new spec lane. Until that lands,
the queue can only run under `build`; once slice 1 ships, the remaining tickets
should be reflowed or created under `build2` so design authors can land literal
red assertions first.

## Kill conditions

Pause the rewrite and escalate if any of these are proven true:

1. **Warm-cache budget miss** - measured warm wall-clock still exceeds five
   minutes and the bottleneck is not upstream I/O.
2. **Schema/key instability** - cache hits routinely produce misses after
   cosmetic/spec-only changes, implying the cache key or wrapper contract is too
   unstable to be a reliable gate.
3. **More than three entities resist semantic/structural coverage** - if the
   rewrite cannot express them without trivia, the runtime surface itself needs
   redesign first.
4. **Study/diagnostic local-data contracts cannot be made truthful without exact
   counts** - that would mean the current surface is too installer-state-driven
   for the new corpus discipline.

## Ticket disposition

- **287** - already done independently. Slice 4 must preserve the shipped
  `make check` -> `test-contracts` contract rather than re-implement it.
- **292** - untouched and orthogonal.
- **293** - closes as superseded once the v2 corpus and rubric land.
- **294** - closes as superseded once the old flaky corpus is gone.
- **295** - closes as superseded because `biomcp-ci` ships in slice 1.
- **296** - closes as superseded because the cache-warm gate replaces a second
  replay system.
