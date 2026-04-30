# Semantic Scholar Runtime Contract

## Status

Target state from architect ticket 364 (2026-04-30). This document records the
current cross-process failure mode found during Project 38 and the incremental
architecture BioMCP should preserve while follow-up tickets ship.

## Current Problems

### BotAssembly strips optional API-key environment variables

Project 38 BioASQ runs start with `S2_API_KEY` present in the shell, March
worker, and pi harness, but BotAssembly's `bash` tool spawns child commands with
only `PATH` in the subprocess environment. BioMCP reads Semantic Scholar
credentials from the `biomcp` process environment via
`src/sources/mod.rs::s2_api_key()`. When BotAssembly launches `biomcp` with a
PATH-only environment, `SemanticScholarClient::new()` in
`src/sources/semantic_scholar.rs` selects the unauthenticated shared-pool client
and emits the "Set S2_API_KEY" rate-limit guidance.

This is a process-boundary contract gap rather than a BioMCP auth bug: BioMCP's
credential source is the subprocess environment; BotAssembly's bash policy is a
stripped environment with no safe allowlist escape hatch.

### Semantic Scholar traffic is process-local

BioMCP's rate limiter is intentionally process-local:
`src/sources/rate_limit.rs::GLOBAL_RATE_LIMITER` is a process-local `OnceLock`.
A single `biomcp` process enforces the Semantic Scholar interval selected by
`RateLimiter::from_env()` (1 request/sec with `S2_API_KEY`, 1 request/2sec
without it). Multiple concurrent `biomcp` subprocesses do not share that budget.

One `search article --source all` invocation can issue both a Semantic Scholar
search request and one or more batch enrichment requests through
`src/entities/article/backends.rs::search_semantic_scholar_candidates()` and
`src/entities/article/enrichment.rs::enrich_article_search_rows_with_semantic_scholar()`.
Parallel benchmark agents can therefore exceed an authenticated key's aggregate
practical quota even when every subprocess is individually configured correctly.

### Search output lacks a structured auth/degraded signal

`biomcp health` already reports Semantic Scholar as authenticated vs.
unauthenticated without printing secrets. `search article` currently reports only
whether Semantic Scholar is enabled in the plan; it does not expose whether the
Semantic Scholar leg ran authenticated, shared-pool, unavailable, or degraded.
Warnings are emitted on stderr, which is easy for agents and benchmark reports to
miss when stdout is the primary evidence channel.

### Authenticated retries ignore Retry-After on the shared-client path

The shared HTTP client in `src/sources/mod.rs::build_http_client()` uses
`reqwest_retry::policies::ExponentialBackoff` with three retries. The current
shared-client policy does not opt into honoring `Retry-After` for 429 responses.
The separate `retry_send()` helper parses `Retry-After`, but Semantic Scholar's
standard authenticated request path uses the shared middleware client instead.
Under authenticated aggregate pressure, short retries can amplify a 429 storm.

## Target Architecture

### Credential boundary: BotAssembly owns env forwarding policy

BioMCP continues to read optional API keys from its own process environment.
BotAssembly must provide the explicit policy bridge for tools it launches:

- Add a bot-folder manifest such as `tools/bash-env.json` under the BotAssembly
  bash tool policy surface, beside `tools/bash-allowlist.json`.
- The manifest declares environment variable names that may be copied from the
  runner host into bash child processes. It stores names only, never values.
- `packages/basm/src/bash-tool.ts` loads and validates this allowlist, then
  builds the spawn environment as `{ PATH, ...allowedPresentHostVars }`.
- Environment variable names are validated with a conservative identifier rule
  (for example `^[A-Z_][A-Z0-9_]*$`), duplicate names are rejected or
  canonicalized, and `PATH` remains runner-owned rather than manifest-owned.
- Tool evidence may record allowed variable names and redacted presence/absence,
  but must never record values, partial values, or secret-derived strings.

For the BioASQ bot, the follow-on agent repo change should declare at least
`S2_API_KEY`; other BioMCP optional keys can be added only as named allowlist
entries with the same redaction rule.

### BioMCP Semantic Scholar client contract

`SemanticScholarClient` remains the single source client for Semantic Scholar.
It should expose a small redacted auth-mode helper for in-process diagnostics:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticScholarAuthMode {
    Authenticated,
    SharedPool,
}
```

- `SemanticScholarClient::new()` still reads `s2_api_key()` once when the client
  is constructed.
- `SemanticScholarClient::auth_mode()` returns the enum above and never exposes
  key material.
- `maybe_with_auth()` remains the only place that attaches `x-api-key`.
- Authenticated requests continue to use `apply_cache_mode_with_auth(..., true)`
  so authenticated responses are not stored in the shared cache.

### Article-search source status contract

`search article` should carry source-level diagnostic state without changing the
row ranking or the primary success/failure semantics. Add an article-search
status structure owned by `src/entities/article/` and rendered by the CLI:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleSourceStatus {
    pub source: ArticleSource,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<SemanticScholarAuthMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ArticleSourceAvailability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArticleSourceAvailability {
    Ok,
    Degraded,
    Unavailable,
    Skipped,
}
```

The status vector should be attached to the article search page returned by
`src/entities/article/search.rs::search_page()` or an adjacent result wrapper,
then rendered as:

- `--json`: `_meta.source_status[]` for machine inspection.
- `--debug-plan`: a leg/source status entry that includes Semantic Scholar
  `auth_mode` and degraded/unavailable status.
- Markdown: keep normal output stable; add a concise source note only when a
  source is degraded or unavailable.

The structure is additive. Existing rows, pagination, ranking metadata,
`semantic_scholar_enabled`, and next-command behavior stay intact.

### Retry contract

The shared HTTP client retry policy should honor upstream `Retry-After` for 429
responses. The target code path is `src/sources/mod.rs::build_http_client()`:
construct `ExponentialBackoff` with Retry-After support, and add a focused test
that proves an authenticated Semantic Scholar 429 with `Retry-After` is not
retried on the short 100/200/400ms cadence.

The unauthenticated shared-pool fast-fail behavior remains unchanged:
`SemanticScholarSharedPoolRateLimitMiddleware` should continue converting
unauthenticated S2 429s into the explicit "Set S2_API_KEY" error instead of
retrying the shared pool.

### Aggregate parallelism contract

BioMCP does not add a custom cross-process lock in the source module. The
existing process-local limiter is kept because it is simple, testable, and works
for single CLI invocations and one shared server process. Multi-agent workloads
must choose one of these deployment shapes:

1. route concurrent agents through one `biomcp serve-http` process so the
   limiter is shared; or
2. keep CLI subprocesses but enforce a benchmark-level concurrency/rate policy
   outside BioMCP and prove the aggregate S2 request rate stays within quota.

Project 38 should first fix the missing-key boundary, then measure authenticated
parallel behavior before adding new substrate-level throttling. If authenticated
parallel runs still 429, the next change belongs to the benchmark runner or
BotAssembly policy layer, not to a source-specific cross-process lock inside
BioMCP.

## Invariants

- API-key values, partial values, hashes, and secret-derived strings never appear
  in logs, health output, search output, run artifacts, tickets, or docs.
- BotAssembly forwards secrets only when a bot-folder policy explicitly names the
  variable; model tool input cannot smuggle arbitrary env values into spawn.
- BioMCP's no-key Semantic Scholar path remains usable and degrades gracefully;
  authenticated mode improves quota but is not required for common commands.
- `search article` remains successful when optional Semantic Scholar search or
  enrichment is unavailable and at least one required article backend returns.
- Source-status metadata is additive and must not change ranking, pagination, or
  row serialization except for explicitly added metadata fields.
- `make check` remains the canonical gate for BioMCP changes; BotAssembly changes
  use that repo's `make -C packages/basm check` or repo-equivalent gate.

## Follow-up Work

1. BotAssembly: add bash env allowlisting and redacted evidence.
2. Agents BioMCP: declare the BioASQ bot's BioMCP key allowlist and prove the
   controlled PATH-only vs allowlisted subprocess behavior.
3. BioMCP: add redacted Semantic Scholar source-status metadata to article
   search output/debug plans.
4. BioMCP: honor `Retry-After` on authenticated shared-client retries while
   preserving unauthenticated shared-pool fast-fail behavior.
5. Agents BioMCP: after key propagation, measure authenticated parallel S2
   behavior and decide whether Project 38 should use `serve-http`, panel-level
   serialization, or a new BotAssembly generic tool-rate policy.
