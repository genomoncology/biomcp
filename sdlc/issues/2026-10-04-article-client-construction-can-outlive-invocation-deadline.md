# plain article provider-client construction blocks on an unbounded cache epoch lock

Status: open. Filed from source review at the named revision. No runtime reproduction is claimed.

Donor revision: `d8c6ce8ba495639b8a798e69e58d379d3762b92a` (main, 2026-10-04, "Clear the lane table: tickets 1290 and 1293 landed and their worktrees are gone").

Priority: P2.

## Symptom

A plain `search article` invocation is scoped to a 60-second deadline, yet the synchronous provider-client construction inside that invocation takes an exclusive file lock on the HTTP cache epoch with no deadline bound. While another process holds that lock (a concurrent article search in a second terminal, a dataset sync, or a cache migration), construction blocks on the async worker thread and can outlive the invocation deadline, because a blocking lock acquisition has no await point the deadline can cancel. The enrichment stages of the same invocation construct their clients the same synchronous way.

## Source paths (at the donor revision, relative to the repository root)

Invocation deadline scope:

- `src/entities/article/search.rs` — `search_page` sets one task-local `VariantArticleDeadline` per invocation when no caller already owns one, and maps deadline-class errors to an invocation failure.
- `src/entities/article/search/deadline.rs` — `ARTICLE_SEARCH_DEADLINE` is 60 seconds; the module comment records ticket 1293's budget.

Plain-path synchronous construction (the `execution == None` branch):

- `src/entities/article/backends.rs` — the plain branch of each backend builds its client with the synchronous constructor: `PubMedClient::new()` (esearch loop), `EuropePmcClient::new()`, `PubTatorClient::new()`, and `SemanticScholarClient::new()`; the variant-article branch already uses the `new_with_deadline` forms.
- `src/entities/article/enrichment.rs` — the enrichment batch builds `SemanticScholarClient::new()` on the plain branch, and the per-row metadata fallback builds `PubTatorClient::new()` and `EuropePmcClient::new()`.

Synchronous constructor chain that takes the unconditional lock:

- `src/sources/pubtator.rs`, `src/sources/pubmed.rs`, `src/sources/europepmc.rs` — each `Client::new()` calls `crate::sources::shared_client()`.
- `src/sources/semantic_scholar.rs` — `SemanticScholarClient::new()` calls `crate::sources::semantic_scholar_provider_client(...)`.
- `src/sources/mod.rs` — `shared_client()` and `semantic_scholar_provider_client()` call `build_http_client_with_config`, which calls `crate::cache::ensure_body_limited_cache_epoch`.
- `src/cache/migration.rs` — `ensure_body_limited_cache_epoch` opens the epoch lock file and calls `lock.lock_exclusive()` unconditionally: no deadline consultation, no try-lock loop.

Existing deadline-aware alternatives at the same seams:

- `src/sources/mod.rs` — `shared_client_with_deadline`, `provider_url_client_with_deadline`, `semantic_scholar_provider_client_with_deadline`, and `build_http_client_with_config_deadline`.
- `src/cache/migration.rs` — `ensure_body_limited_cache_epoch_until` polls `try_lock_exclusive` under the deadline and returns `TimedOut` on expiry.
- `src/cache/clear.rs` — the synchronous `lock_cache_shared` already honors the task-local deadline by polling `try_lock_shared`; the epoch lock acquisition is the one unconditional wait left on this path.
- Per-client async constructors: `new_with_deadline` in `src/sources/pubtator.rs`, `src/sources/pubmed.rs`, `src/sources/europepmc.rs`, `src/sources/semantic_scholar.rs`.

## Invariant

Every lock wait inside a deadline-scoped article search invocation must be cancellable by that deadline. Provider-client construction happens inside the invocation, so its cache lock acquisition must bound itself by the invocation deadline rather than block without limit.

## Evidence (source reading only)

The code paths above are quoted from the donor revision. The blocking `lock_exclusive()` in `ensure_body_limited_cache_epoch` runs inside the deadline-scoped `search_page` task, and nothing between `search_page` and that call consults the deadline on the plain path. A generic consumer running two concurrent article searches, or a search alongside a cache migration or dataset sync in another process, holds the epoch lock on one side while the other side constructs its client; the constructor's lock wait has no bound the invocation deadline can enforce. No runtime reproduction was performed by this job.

## Bounded remediation (proposed)

On the plain article path (no variant-article execution context), read the task-local deadline (`current_variant_article_deadline()`) and use the existing deadline-aware constructors when one is scoped, keeping the synchronous constructors only for callers with no deadline. No cache format, lock file, or lock-semantics change; only which constructor the plain article path selects. The variant-article path, which explicitly owns its deadline, stays unchanged, as do non-article callers of the synchronous constructors.

## Finite future offline proof (to be written by the fixing ticket; not run here)

1. Hold the real cache epoch lock with a deterministic lock-acquired signal. Scope a controlled invocation clock around the plain constructor and enrichment, then explicitly advance it to expiry. Assert the retryable deadline error or degraded enrichment with retained rows. No sleep or elapsed-time threshold.
2. Release the same held lock under an unexpired controlled invocation and assert successful empty search and null enrichment. Keep the existing explicit variant ownership and ordinary-error controls.

## Relationship to existing records

Ticket 1293 ("bound article search time and report partial sources", complete, landed `6bf7ca55`) introduced the invocation deadline and the deadline-aware seams; its deferral list names only NCBI rate-limit handling, and no open issue in `sdlc/issues/` records this lock-acquisition gap. The GenCC store lock deadline work (ticket 1187) is a different subsystem with the opposite direction of change and is not affected.

## Reproduced 2026-10-05

Experiment 439, binary at main a877443f. With an exclusive flock held on `~/.cache/biomcp/.body-limit-cache-v1.lock` and `BIOMCP_TEST_ARTICLE_SEARCH_DEADLINE_MS=3000`, `search article --source pubmed "BRAF melanoma" --limit 3 -j` produced no output for the full 90 s harness timeout: the deadline cannot cancel the block. Lock-free control: the same command exits 0 with 3 rows in 35.0 s wall (10.2 s user, 5.9 s sys), so the plain path's own floor is high independently. Root cause in code confirmed: the plain branch constructs clients synchronously (`src/entities/article/backends.rs:244, 410, 574, 673`; `src/entities/article/enrichment.rs:178, 334, 345`) through `shared_client()` → `ensure_body_limited_cache_epoch` → `lock.lock_exclusive()` (`src/cache/migration.rs:130-132`), a blocking lock with no await point; the deadline-aware `ensure_body_limited_cache_epoch_until` (`migration.rs:150-176`) exists and is used only by the variant-article path.
