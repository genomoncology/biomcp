# T010 Design: Infinite Cache Mode for Deterministic Spec Testing

## Problem

BioMCP hits 28+ live APIs. Spec tests need deterministic, network-independent
replay. The existing cache (`http-cache-reqwest` + `cacache`) respects HTTP
semantics with a 24h max-stale window, which is insufficient for reproducible
test runs.

## Architecture

### Cache Mode Enum

Introduce a `BIOMCP_CACHE_MODE` environment variable read once at startup,
mapped to an internal enum that controls per-request `CacheMode` extensions.

```
BIOMCP_CACHE_MODE  ->  Internal behavior
─────────────────────────────────────────
(unset) / "default"  ->  Current behavior (HTTP semantics, 24h max-stale)
"infinite"           ->  CacheMode::ForceCache — use cached response if
                         available regardless of staleness; fetch + store
                         on cache miss
"off"                ->  CacheMode::NoStore — equivalent to --no-cache
```

### Precedence Rules

```
Priority (highest first):
1. --no-cache CLI flag          (explicit user intent, always wins)
2. Authenticated request        (security: never cache auth'd responses)
3. BIOMCP_CACHE_MODE env var    (ambient config)
4. Default (HTTP semantics)
```

The `--no-cache` flag sets the `NO_CACHE` task-local to `true`, which already
forces `NoStore`. This override remains at the top of the precedence chain.

Authenticated requests (`apply_cache_mode_with_auth` with `authenticated=true`)
continue to force `NoStore` regardless of env var — this preserves security.

### Design Decision: GWAS NoStore Override

`gwas.rs` hard-codes `CacheMode::NoStore` via `request_no_store()` to work
around cache decode failures. In infinite mode, `apply_cache_mode` will
override this to `ForceCache`. This is intentional: infinite mode is for
deterministic replay, and GWAS responses that are already cached should replay
like any other source. If GWAS cache entries are corrupted, the user can clear
the cache and re-populate.

Flow: `request_no_store()` sets `NoStore` → `get_json_optional()` passes to
`apply_cache_mode()` → `with_extension(ForceCache)` replaces the `NoStore`.

### Design Decision: MCP Server Path

The MCP server dispatches through `cli::execute()` which calls `run(cli)`.
The parsed `Cli` struct defaults `no_cache` to `false`. The MCP path does not
pass `--no-cache`, so `BIOMCP_CACHE_MODE` will apply to MCP requests too. This
is correct — the env var is an ambient configuration that should affect all
request paths.

### Design Decision: Sources Not Using Cache Middleware

Four sources (or specific code paths) bypass `apply_cache_mode` and are
unaffected by `BIOMCP_CACHE_MODE`:

- **AlphaGenome** (`alphagenome.rs`) — uses gRPC (tonic), not HTTP cache
  middleware; completely outside the reqwest cache stack.
- **UniProt** (`uniprot.rs`) — uses `streaming_http_client()` (raw
  `reqwest::Client`, no middleware stack); all UniProt requests bypass cache.
- **MyDisease get-by-ID** (`mydisease.rs:~173`) — the bulk-fetch / get-by-ID
  path calls `.send()` directly without going through `apply_cache_mode`.
  The search path (`get_json` helper at line ~69) is covered. Follow-up
  ticket recommended to route the direct-send path through the cache helper.
- **Enrichr `addList` streaming** (`enrichr.rs:~78`) — the multipart streaming
  upload uses `retry_send` directly (comment: "bypass middleware because it
  uses a streaming body"). The read/lookup paths (`send_bytes` at line ~56)
  do use `apply_cache_mode` and are covered. Follow-up ticket recommended.

**Scope of T010:** `BIOMCP_CACHE_MODE=infinite` affects all requests that
flow through `apply_cache_mode` or `apply_cache_mode_with_auth`. The four
exceptions above are explicitly out of scope for this ticket and will not
change behavior. This is documented and acceptable for the initial
implementation — spec tests should avoid these sources or accept that they
will still hit the network when running with infinite cache mode.

These sources will always hit the network regardless of cache mode. This is
acceptable — they use streaming/gRPC patterns that cannot pass through the
retry/cache middleware, and their test coverage is a follow-up concern.

### Design Decision: Authenticated Sources in Infinite Mode

Sources that always pass `authenticated=true` will bypass infinite cache:

- **NCI CTS** (`nci_cts.rs`) — always authenticated (API key required)

Sources with optional API keys will use infinite cache only when no API key
is configured:

- **PubTator** (`pubtator.rs`) — `self.api_key.is_some()`
- **OpenFDA** (`openfda.rs`) — `self.api_key.is_some()`
- **PMC OA** (`pmc_oa.rs`) — `self.api_key.is_some()`
- **NCBI IDConv** (`ncbi_idconv.rs`) — `self.api_key.is_some()`
- **OncoKB** (`oncokb.rs`) — conditional on token presence

For spec testing: run without API keys to get full infinite caching across all
sources (except NCI CTS which requires a key, and AlphaGenome/UniProt which
use non-HTTP transports). Alternatively, accept that these sources will hit
the network on each spec run.

## Implementation Plan

### File: `src/sources/mod.rs` (modify)

**1. Add `parse_cache_mode` pure function**

Extract the parsing logic into a pure, testable function:

```rust
fn parse_cache_mode(value: Option<&str>) -> Option<CacheMode> {
    match value {
        Some("infinite") => Some(CacheMode::ForceCache),
        Some("off") => Some(CacheMode::NoStore),
        Some("default") | Some("") | None => None,
        Some(other) => {
            warn!("Unknown BIOMCP_CACHE_MODE={other:?}, using default");
            None
        }
    }
}
```

**2. Add `env_cache_mode` with OnceLock**

Read `BIOMCP_CACHE_MODE` once on first access:

```rust
fn env_cache_mode() -> Option<CacheMode> {
    static MODE: OnceLock<Option<CacheMode>> = OnceLock::new();
    *MODE.get_or_init(|| {
        parse_cache_mode(
            std::env::var("BIOMCP_CACHE_MODE")
                .ok()
                .map(|s| s.trim().to_ascii_lowercase())
                .as_deref(),
        )
    })
}
```

**3. Modify `apply_cache_mode`**

Current (lines 67-72):
```rust
pub(crate) fn apply_cache_mode(req: RequestBuilder) -> RequestBuilder {
    match NO_CACHE.try_with(|v| *v) {
        Ok(true) => req.with_extension(CacheMode::NoStore),
        _ => req,
    }
}
```

New:
```rust
pub(crate) fn apply_cache_mode(req: RequestBuilder) -> RequestBuilder {
    // --no-cache flag takes highest priority
    if let Ok(true) = NO_CACHE.try_with(|v| *v) {
        return req.with_extension(CacheMode::NoStore);
    }
    // Then env var
    if let Some(mode) = env_cache_mode() {
        return req.with_extension(mode);
    }
    req
}
```

**4. `apply_cache_mode_with_auth` — no changes needed**

The auth check already takes priority over `apply_cache_mode`:
```rust
pub(crate) fn apply_cache_mode_with_auth(
    req: RequestBuilder,
    authenticated: bool,
) -> RequestBuilder {
    if authenticated {
        return req.with_extension(CacheMode::NoStore);
    }
    apply_cache_mode(req)
}
```

### File: `src/sources/mod.rs` — Tests (add)

Add unit tests in the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn parse_cache_mode_returns_none_for_default() {
    assert!(parse_cache_mode(None).is_none());
    assert!(parse_cache_mode(Some("default")).is_none());
    assert!(parse_cache_mode(Some("")).is_none());
}

#[test]
fn parse_cache_mode_returns_force_cache_for_infinite() {
    assert!(matches!(
        parse_cache_mode(Some("infinite")),
        Some(CacheMode::ForceCache)
    ));
}

#[test]
fn parse_cache_mode_returns_no_store_for_off() {
    assert!(matches!(
        parse_cache_mode(Some("off")),
        Some(CacheMode::NoStore)
    ));
}

#[test]
fn parse_cache_mode_returns_none_for_unknown() {
    assert!(parse_cache_mode(Some("bogus")).is_none());
}
```

### File: `Makefile` (modify)

Add a `spec` target to `.PHONY` and define it:

```makefile
.PHONY: build test check run clean spec

spec:
	BIOMCP_CACHE_MODE=infinite uv run pytest spec/ --mustmatch-lang bash -v
```

### No other files need modification

The change is entirely contained in `src/sources/mod.rs` (and `Makefile`).
Of 29 source clients, the majority call `apply_cache_mode` or
`apply_cache_mode_with_auth` and inherit infinite mode automatically.
**Out-of-scope exceptions** (documented above): AlphaGenome (gRPC), UniProt
(streaming client), MyDisease get-by-ID direct send, and Enrichr addList
streaming path. These four paths bypass cache middleware and are explicitly
out of scope for T010.

## File Disposition

| File | Action | Description |
|------|--------|-------------|
| `src/sources/mod.rs` | **Modify** | Add `parse_cache_mode`, `env_cache_mode`, update `apply_cache_mode`, add tests |
| `Makefile` | **Modify** | Add `spec` target with `BIOMCP_CACHE_MODE=infinite` |

## Acceptance Criteria

1. **`BIOMCP_CACHE_MODE=infinite` uses ForceCache for middleware-backed requests** —
   When env var is set to `infinite`, all requests flowing through
   `apply_cache_mode` or `apply_cache_mode_with_auth` use `CacheMode::ForceCache`.
   Out-of-scope sources (AlphaGenome gRPC, UniProt streaming, MyDisease
   get-by-ID, Enrichr addList) are explicitly excluded and will still hit the
   network. Verified by: unit test on `parse_cache_mode("infinite")` returns
   `Some(ForceCache)`, and code inspection confirming the env var path in
   `apply_cache_mode`.

2. **`BIOMCP_CACHE_MODE=off` equivalent to `--no-cache`** — When env var is
   set to `off`, all requests use `CacheMode::NoStore`. Verified by: unit
   test on `parse_cache_mode("off")` returns `Some(NoStore)`.

3. **`BIOMCP_CACHE_MODE=default` or unset preserves current behavior** —
   No `CacheMode` extension is set by the env var path; existing 24h max-stale
   behavior continues. Verified by: unit test on `parse_cache_mode(None)` and
   `parse_cache_mode(Some("default"))` both return `None`.

4. **`--no-cache` flag overrides env var** — The `NO_CACHE` task-local check
   occurs before the env var check in `apply_cache_mode`. Verified by: code
   inspection (the `if let Ok(true)` return is first) and unit test combining
   both.

5. **Authenticated requests still skip cache** — `apply_cache_mode_with_auth`
   returns `NoStore` when `authenticated=true`, regardless of env var. Verified
   by: existing behavior unchanged, code inspection.

6. **Unknown env var values warn and fall back to default** — Verified by:
   unit test on `parse_cache_mode(Some("bogus"))` returns `None` (and logs a
   warning).

7. **Spec tests pass deterministically with cached responses** — After
   `make spec` runs once with network access, subsequent `make spec` runs
   produce identical results without network. Verified by: running spec suite
   twice in CI.

## Edge Cases

- **Empty string**: `BIOMCP_CACHE_MODE=""` treated as unset via `Some("") =>`
  in the `parse_cache_mode` match arm (alongside `None` and `"default"`).

- **Case sensitivity**: Env var values are lowercased before matching.
  `BIOMCP_CACHE_MODE=INFINITE` works.

- **OnceLock initialization race**: `OnceLock::get_or_init` is thread-safe.
  Multiple tasks reading concurrently is fine.

- **GWAS NoStore in infinite mode**: Overridden to ForceCache. This is correct
  for deterministic replay. If GWAS cache corruption is an issue, clear the
  cache directory.

- **NCI CTS in infinite mode**: Always bypasses cache due to required API key
  and `authenticated=true`. Spec tests for NCI CTS will hit the network.

- **AlphaGenome**: gRPC transport — completely outside HTTP cache middleware.
  Unaffected by `BIOMCP_CACHE_MODE`.

- **UniProt**: Uses `streaming_http_client()` (raw reqwest::Client) which
  has no cache middleware. Unaffected by `BIOMCP_CACHE_MODE`.
