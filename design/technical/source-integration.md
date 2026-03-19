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
  `src/cli/mod.rs`, `src/cli/list.rs`, and
  `docs/user-guide/cli-reference.md` when the public CLI surface changes.
- The progressive-disclosure behavior described in
  `design/functional/overview.md` and `docs/concepts/progressive-disclosure.md`
  remains the governing UX rule.

Entity integration shapes differ by entity, but common patterns include:

- adding new optional fields or section structs to the owning entity type;
- gating a section on prerequisite identifiers already present on the base
  entity card;
- keeping helper commands for true cross-entity pivots rather than routine
  upstream enrichment.

## Provenance and Rendering

Source identity must remain visible in output.

- Preserve provenance in markdown and JSON rather than normalizing it away.
- Use the entity's existing rendering shape, such as per-row `source`,
  `source_label`, stable source identifiers, or source-specific notes.
- Do not merge facts from different upstreams into one unlabeled result when
  the user needs to understand where the data came from.
- Rendering work may require changes in `src/render/markdown.rs`,
  `src/render/json.rs`, or both.

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
- On upstream failure or timeout, warn and degrade gracefully in the shape that
  matches the entity: default section structs, empty collections, omitted
  optional fields, or explanatory notes are all used in the current repo.
- Returned output must stay truthful about missing or unavailable data.

Default-path integrations can still return hard errors when the source is
required for the base command, but those failures should use clear
`BioMcpError` variants with useful recovery suggestions.

## Rate Limits and Operational Constraints

Source additions must preserve BioMCP's runtime boundaries.

- Keep slow or failure-prone upstream calls off the default `get` path unless
  the latency and failure profile are already acceptable there.
- Respect the process-local rate limiting model described in
  `design/technical/overview.md`.
- When many workers need one shared limiter budget, the operational answer is
  `biomcp serve-http`, not a per-ticket custom coordination layer.
- Reuse source-specific rate limiting already present in `src/sources/` when a
  provider has special throughput rules.
- Document source-specific enforced limits, practical ceilings, or payload
  constraints in `docs/reference/data-sources.md` when a new integration adds
  them.

## Source Addition Checklist

Every new source or source-deepening ticket should evaluate and update the
following surfaces when applicable:

- `src/sources/<source>.rs`
- `src/sources/mod.rs`
- the owning entity module(s) in `src/entities/`
- rendering surfaces in `src/render/`
- `src/cli/mod.rs`
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
