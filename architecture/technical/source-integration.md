# BioMCP Source Integration Architecture

This document is the durable contract for adding a new upstream source to
BioMCP or deepening an existing integration. It is for implementers and
reviewers working in `src/`, `docs/`, `spec/`, and `scripts/`.

The goal is consistency without pretending every source follows one rigid
template. Some conventions are required; others are preferred when they fit the
source's transport, authentication, and payload shape.

## New Source vs Existing Source

- Add one module per upstream provider under `src/sources/<source>.rs` when the
  repo does not already have a client for that upstream.
- Extend the existing module when the work deepens an already integrated
  provider instead of creating a sibling client for the same API surface.
- Current examples of distinct upstream modules include
  `src/sources/hpa.rs`, `src/sources/gnomad.rs`, and
  `src/sources/complexportal.rs`.
- Current examples of extension work include `src/sources/opentargets.rs`,
  which already owns multiple OpenTargets query paths.
- Every source module must be declared from `src/sources/mod.rs`.

This prevents duplicated auth handling, base URL overrides, rate limiting, and
error behavior for the same provider.

## Shared Source Client Conventions

BioMCP source clients should reuse shared helpers from `src/sources/mod.rs`
when they apply:

- Use `shared_client()` for ordinary JSON/HTTP request flows that fit the
  middleware stack.
- Use `streaming_http_client()` when middleware-compatible request cloning or
  streaming is not workable.
- Use `env_base(default, ENV_VAR)` when a source needs a testable or
  operator-overridable base URL.
- Use `read_limited_body()` and `body_excerpt()` for bounded error handling and
  readable upstream failure messages.
- Use `retry_send()` when explicit retry handling is needed outside the shared
  middleware path, especially for streaming or provider-specific request
  builders.
- Reuse provider-specific rate limiting already present in the repo instead of
  inventing a second limiter for the same source.

These are conventions, not a fake one-size-fits-all constructor contract. The
current repo does not require every client to share one name, one constructor
shape, or one exact error-variant mix.

## Local Runtime Sources and File-Backed Assets

Not every file-backed dependency participates in the same runtime lifecycle.

### Local runtime sources

EMA, WHO Prequalification, CDC CVX/MVX, GTR, and WHO IVD are local runtime sources.

- Runtime resolution is owned by the source module, not hard-coded in docs.
- EMA resolves `BIOMCP_EMA_DIR` first, then the platform data directory.
- WHO resolves `BIOMCP_WHO_DIR` first, then the platform data directory.
- CDC CVX/MVX resolves `BIOMCP_CVX_DIR` first, then the platform data directory.
- GTR resolves `BIOMCP_GTR_DIR` first, then the platform data directory.
- WHO IVD resolves `BIOMCP_WHO_IVD_DIR` first, then the platform data directory.
- first-use auto-download prepares missing local runtime files for commands
  that need those sources, while the sync commands force-refresh the same
  local bundles for operator workflows.
- Full `biomcp health` includes the EMA, WHO Prequalification, CDC CVX/MVX, GTR, and WHO IVD local-data readiness rows.
- `biomcp health --apis-only` excludes those rows because local runtime data is
  not an upstream API.
- The row status contract is `configured`, `configured (stale)`,
  `available (default path)`, `available (default path, stale)`,
  `not configured`, and `error (missing: ...)`.
- The canonical provider terms live in `docs/reference/source-licensing.md`
  and the machine-readable source inventory in `docs/reference/sources.json`.
- Operator-facing setup details live in `docs/user-guide/drug.md` for the drug
  local data roots and `docs/user-guide/diagnostic.md` for GTR and WHO IVD.
- CDC CVX/MVX only augments the EMA plain-name vaccine identity path after
  MyChem misses; it is not a WHO alias source and pure `--region us` searches
  do not use the CVX root.

| Source | Env root | Required files | Stale window | Sync command | Health/API-only | Terms |
|---|---|---|---|---|---|---|
| EMA | `BIOMCP_EMA_DIR` | `medicines.json`, `post_authorisation.json`, `referrals.json`, `psusas.json`, `dhpcs.json`, `shortages.json` | 72 hours | `biomcp ema sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |
| WHO Prequalification | `BIOMCP_WHO_DIR` | `who_pq.csv`, `who_api.csv`, `who_vaccines.csv` | 72 hours | `biomcp who sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |
| CDC CVX/MVX | `BIOMCP_CVX_DIR` | `cvx.txt`, `TRADENAME.txt`, `mvx.txt` | 30 days | `biomcp cvx sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |
| GTR | `BIOMCP_GTR_DIR` | `test_version.gz`, `test_condition_gene.txt` | 7 days | `biomcp gtr sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |
| WHO IVD | `BIOMCP_WHO_IVD_DIR` | `who_ivd.csv` | 72 hours | `biomcp who-ivd sync` | full health row; omitted from `--apis-only` | `docs/reference/source-licensing.md` / `docs/reference/sources.json` |

EMA and WHO Prequalification use a 72-hour stale window; CDC CVX/MVX uses a
30-day refresh window; GTR uses a 7-day stale window; WHO IVD uses a 72-hour
stale window.

This keeps local runtime readiness grounded in `src/cli/health.rs` and
`src/sources/ema.rs` / `src/sources/who_pq.rs` / `src/sources/cvx.rs` /
`src/sources/gtr.rs` / `src/sources/who_ivd.rs` while leaving operator setup
details in the user guide.

### File-backed non-runtime assets

BioASQ is the canonical file-backed non-runtime asset.

- File-backed benchmark and evaluation assets are repo-local artifacts, not
  runtime query providers.
- They do not join the runtime source inventory, `biomcp health`, or the
  source-readiness checklist unless a future ticket makes them runtime-visible.
- The authoritative operator/reference home remains
  `docs/reference/bioasq-benchmark.md`.
- The repo-local grounding surface is `benchmarks/bioasq/`.

## Section-First Entity Integration

BioMCP prefers entity-section integration over ad hoc command sprawl.

- New upstream data should usually extend an existing entity in `src/entities/`
  rather than adding a new top-level command family.
- The default card should stay concise and reliable. New network-backed data
  belongs behind named sections unless there is a strong reason to put it on
  the default path.
- Section names must fit the existing `get <entity> <id> [section...]`
  contract, where default `get` output stays concise and optional sections
  expand on demand.
- Keep the user-facing command grammar aligned with code changes by updating
  `src/cli/commands.rs` for top-level command families and `after_help`
  examples, the owning entity CLI module such as `src/cli/drug/mod.rs` for
  entity-specific clap/help text, `src/cli/list.rs`, and
  `docs/user-guide/cli-reference.md` when the public CLI surface changes.
- The progressive-disclosure behavior described in
  `architecture/functional/overview.md` and `docs/concepts/progressive-disclosure.md`
  remains the governing UX rule.

Entity integration shapes differ by entity, but common patterns include:

- adding new optional fields or section structs to the owning entity type;
- gating a section on prerequisite identifiers already present on the base
  entity card;
- keeping helper commands for true cross-entity pivots rather than routine
  upstream enrichment.

## Entity-Specific Command Modifiers

Some entities need named modifiers in addition to section names. The durable
contract is:

- The base grammar remains `get <entity> <id> [section...]`.
- Entity-specific modifiers are named options that sit beside the positional
  `sections` list; they are not new positional arguments.
- The canonical example is `get drug <name> ... --region <us|eu|who|all>`.
- This modifier pattern is distinct from unrelated search filters that happen
  to reuse the same flag name on other entities.

Implementers must keep every alignment surface in sync:

- top-level command families and `after_help` examples in
  `src/cli/commands.rs`
- entity-specific clap argument/help text in the owning CLI module, such as
  `src/cli/drug/mod.rs`
- concise list/help output in `src/cli/list.rs` and `src/cli/list_reference.md`
- user-facing docs in `docs/user-guide/cli-reference.md` and the owning entity
  guide such as `docs/user-guide/drug.md`
- executable CLI contract coverage in the owning canary or surface spec when
  that command participates in the active tree

Runtime validation belongs in the owning entity or CLI path, not only in
docs/help text.

The current drug contract is the model:

- `--region` only changes the data plane for `regulatory`, `safety`,
  `shortage`, or `all`
- `--region who` is valid for `regulatory` and `all`, but not for `safety` or
  `shortage`
- `approvals` remains U.S.-only
- invalid flag/section combinations fail fast before data fetches

## Source-Aware Section Capability Contract

Multi-source entities must define one authoritative capability model for their
section surface.

The current pathway implementation is the concrete example:

- section constants and parsing live in `src/entities/pathway.rs`
- source-specific capability lists live there as
  `REACTOME_PATHWAY_SECTIONS` and `KEGG_PATHWAY_SECTIONS`
- runtime validation resolves against the capability list for the resolved
  source
- `all` expands to the sections supported by the resolved source, not the
  union of all sections across every source for that entity

For source-aware entities, "all" means all sections available for the resolved
source, not all sections ever defined for that entity.

This contract defines three user-visible states:

- **unsupported**: the resolved source does not offer that section. Fail fast
  with a source-aware `InvalidArgument` error and a recovery hint. Current
  pathway example: KEGG `events` and `enrichment`.
- **empty**: the section is supported and the upstream call succeeded, but no
  data came back. Return the entity's truthful empty shape.
- **unavailable**: the section is supported, but the upstream failed or timed
  out. Treat this as unavailable, not unsupported, and degrade in the owning
  entity's established shape rather than inventing a new hard failure path.

Downstream surfaces must stay in lockstep with the capability model:

- top-level command/help examples in `src/cli/commands.rs` and entity-specific
  option/help text in the owning CLI module, such as `src/cli/drug/mod.rs`,
  must describe source-specific limits accurately
- `src/cli/list.rs` must not advertise unsupported sections as universally
  valid
- renderer follow-on commands and section suggestions must derive from the
  resolved source capability list
- docs that teach sections, including `architecture/functional/overview.md`,
  `docs/concepts/progressive-disclosure.md`, and user-guide pages, must either
  describe the source-aware constraint inline or show examples valid for the
  named source

## Multi-Source Search Ranking

When an entity search fans out across multiple upstreams, ranking happens after
the combined fetch, not by hard-coding a preferred source.

The current pathway search contract is the model:

1. Normalize the query once for upstream search and ranking.
2. Fetch Reactome and KEGG results in parallel when both sources are enabled.
3. Score title matches globally across both source result sets:
   - Tier 3: exact normalized title match
   - Tier 2: normalized title starts with the normalized query
   - Tier 1: normalized title contains the normalized query
   - Tier 0: no title match
4. Sort by tier descending, then upstream position ascending, then stable ID
   ascending as the final tiebreaker.
5. Preserve source identity on every row.
6. Truncate after ranking.

Two semantics are non-negotiable:

- there is no fixed source-priority rule within the same title-match tier
- federated totals are source-aware:
  - if rows from both sources are merged, the total is not presented as an
     exact combined count
  - if only one source contributes rows and that source has an authoritative
     total, that source total may be preserved

### Article-search ranking contract

For federated article search, the article contract is more specific than the
generic multi-source rule above:

1. Build ranking concepts from the typed article filters in
   `src/entities/article/mod.rs`. Structured filters remain one concept each, while
   `--keyword` is decomposed into independently matchable concepts instead of
   one exact phrase blob.
2. Apply one shared normalization strategy in `src/transform/article.rs` to
   both anchor creation and result normalization, including compact matching for
   compound-name punctuation variants that PubMed commonly resolves through MeSH
   or author keywords.
3. Capture source-local backend position before cross-source merge and preserve
   it through dedup on the merged `ArticleSearchResult`. Cross-source append
   order is not part of the relevance contract.
4. Resolve the effective relevance mode after filters are finalized:
   keyword-bearing queries default to `hybrid`, while entity-only queries
   default to `lexical`.
5. Sort federated article rows with explicit ranking signals rather than hidden
   source identity:
   - `lexical` keeps the existing calibrated PubMed rescue plus lexical
     directness comparator;
   - `semantic` sorts LitSense2 score descending, then falls back to lexical;
   - `hybrid` scores
     `semantic + lexical + citations + position` with configurable weights and
     stable-ID tiebreaking.
6. Preserve provenance on the result row and, when ranking metadata is
   serialized, expose the effective mode plus the normalized component scores
   needed to explain hybrid and semantic ordering in JSON output.

The article-specific invariants are:

- the same query and document normalization must be used on both sides of
  anchor matching;
- source-local rank is comparable within a source, not a proxy for source
  priority across the whole federation;
- merge order must not be observable as a relevance penalty; and
- calibrated PubMed rescue remains part of lexical fallback behavior, but it is
  one explicit ranking signal rather than an implicit source-priority rule.

## Non-JSON Transport Guidance

Source integrations do not have to be JSON to fit the repo's architecture.

- Keep using the repo's bounded-body and readable-error conventions.
- Name the response media type and parsing approach in module-level docs or
  nearby architectural prose so reviewers can orient quickly.
- Prefer the shared HTTP client when it fits; transport format alone does not
  justify a new client.
- Inline parsing is acceptable for small line-oriented text payloads.
- Move synchronous parsing to `tokio::task::spawn_blocking` when parser cost or
  payload size would block the async runtime inappropriately.
- Reject obviously wrong content types when an upstream commonly falls back to
  HTML or another human-facing error surface.

Current concrete examples:

- KEGG uses plain-text flat-file / tab-separated style responses and parses
  them inline in `src/sources/kegg.rs`.
- HPA uses XML and parses it with `roxmltree` behind `spawn_blocking` in
  `src/sources/hpa.rs`.

## Provenance and Rendering

Source identity must remain visible in output.

- Preserve provenance in markdown and JSON rather than normalizing it away.
- Use the entity's existing rendering shape, such as per-row `source`,
  `source_label`, stable source identifiers, or source-specific notes.
- Do not merge facts from different upstreams into one unlabeled result when
  the user needs to understand where the data came from.
- Rendering work may require changes in `src/render/markdown/`,
  `src/render/json.rs`, or shared provenance helpers such as
  `src/render/provenance.rs`.

The exact representation is not universal across the repo. Some sections label
individual rows, some label source groups, and some preserve provenance through
source-specific notes and identifiers.

## Auth, Cache, and Secrets

Authenticated or key-gated integrations have extra requirements.

- Required credentials must fail clearly with `BioMcpError::ApiKeyRequired`.
- Optional credentials must improve quota or capability without breaking the
  baseline no-key workflow, unless the feature itself is intentionally
  key-gated.
- Authenticated requests must use the no-store cache path, for example
  `apply_cache_mode_with_auth(..., true)`, so private responses are not cached
  like shared public responses.
- Document new or changed keys in `docs/getting-started/api-keys.md` and
  `docs/reference/data-sources.md`.
- Keep secrets in environment variables and out of repository files.
- User-facing errors may name the required env var and docs page, but must not
  echo the credential value.
- Do not log secrets.

## Graceful Degradation and Timeouts

Optional enrichment is best-effort across the repo, but the exact fallback
shape is entity-specific.

- Optional enrichments must not take down the whole command.
- Use bounded async enrichment with the entity-local timeout style instead of
  inventing unrelated latency budgets.
- Follow the owning entity's established timeout constant and section pattern.
  Current entities use values such as 8 seconds for gene, disease, and variant
  enrichments, and 10 seconds for PGx enrichment.
- If a prerequisite identifier is missing, prefer an empty/default/note result
  for optional sections over a hard failure.
- On upstream failure or timeout, treat supported sections as unavailable, not
  unsupported, and warn and degrade gracefully in the shape that matches the
  entity: default section structs, empty collections, omitted optional fields,
  or explanatory notes are all used in the current repo.
- Returned output must stay truthful about missing or unavailable data.

Default-path integrations can still return hard errors when the source is
required for the base command, but those failures should use clear
`BioMcpError` variants with useful recovery suggestions.

## Rate Limits and Operational Constraints

Source additions must preserve BioMCP's runtime boundaries.

- Keep slow or failure-prone upstream calls off the default `get` path unless
  the latency and failure profile are already acceptable there.
- Respect the process-local rate limiting model described in
  `architecture/technical/overview.md`.
- When many workers need one shared limiter budget, the operational answer is
  `biomcp serve-http`, not a per-ticket custom coordination layer.
- Reuse source-specific rate limiting already present in `src/sources/` when a
  provider has special throughput rules.
- Document source-specific enforced limits, practical ceilings, or payload
  constraints in `docs/reference/data-sources.md` when a new integration adds
  them.

## Source Addition Checklist

Minimum required proof surface:

| Surface | Required when | Contract |
|---------|---------------|----------|
| Targeted Rust tests near `src/sources/`, `src/entities/`, or `src/render/` | All source additions and source-deepening work | Verify behavior and edge handling for the new integration contract |
| `spec/` BDD update | Any user-visible CLI contract change | Cover stable user-visible behavior such as new sections, new flags, or changed output structure |
| `scripts/contract-smoke.sh` | Stable public endpoints with deterministic probe shapes | Add or update live probes when operationally suitable; skip or reduce secret-gated or volatile sources explicitly |
| `src/cli/health.rs` | Readiness-significant sources that operators should inspect directly | Include the source when it materially affects baseline availability; key-gated sources may appear as excluded when unconfigured |
| `CHANGELOG.md` | User-visible source additions or major deepening | Record the shipped surface change |

Every new source or source-deepening ticket should then evaluate the following
surfaces when applicable:

- `src/sources/<source>.rs`
- `src/sources/mod.rs`
- the owning entity module(s) in `src/entities/`
- rendering surfaces in `src/render/`
- the owning CLI modules in `src/cli/`, including `src/cli/commands.rs` for
  top-level grammar/help and entity modules such as `src/cli/drug/mod.rs` for
  modifier-specific text
- `src/cli/list.rs`
- `docs/user-guide/cli-reference.md` when the public command surface changes
- `docs/reference/data-sources.md`
- `docs/getting-started/api-keys.md` when credentials are added or changed
- `docs/reference/source-versioning.md` when a new upstream endpoint or version
  pin is introduced
- `src/cli/health.rs` when the source should participate in operator health
  visibility
- `scripts/contract-smoke.sh` when the upstream is suitable for live contract
  probes
- `spec/` when the stable CLI contract changes in a user-visible, assertable
  way
- targeted Rust tests near the new source, entity, and rendering behavior
- `CHANGELOG.md` for user-visible source additions or major deepening work

Not every item changes on every ticket. The contract is to evaluate each
surface deliberately and update the ones the new source actually touches.
