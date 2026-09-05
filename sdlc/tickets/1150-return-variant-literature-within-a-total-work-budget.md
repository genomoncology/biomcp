---
flow: build
priority: 8
deps: [1167]
---

# Return variant literature within one invocation-wide work deadline

## Goal

One `variant articles` invocation has one monotonic 60-second provider-work
deadline and returns every article page or provider leg completed before that
deadline. The same absolute deadline covers a single CLI query and the complete
1-10 item batch used by CLI `--input` and the typed `variant_articles` MCP tool.
It is never reset per item, route, provider, retry, page, enrichment, or
verification phase.

## Current facts

Ticket 1167 has landed. It removed recursive whole-cache repair after every
write but intentionally retained cold-construction repair, cleanup, and size
scans. The exact post-1167 command was rerun:

```bash
biomcp --json variant articles "ODC1 c.1342A>T" --limit 10
```

| Cache | Wall | User | System | Exit |
|---|---:|---:|---:|---:|
| Representative accumulated | 99.73 s | 14.50 s | 29.47 s | 0 |
| Fresh | 31.61 s | 0.11 s | 0.06 s | 0 |

Both runs returned 10 rows only at command completion, with `complete: false`,
`truncated: true`, `pagination.total: null`, and `has_more: true`. Route debug
timings below are milliseconds/calls:

| Cache | Route | Europe PMC | PubMed | PubTator | Semantic Scholar |
|---|---|---:|---:|---:|---:|
| Accumulated | strict | 1,756/7 | 1,358/7 | 970/7 | 40,262/7 |
| Accumulated | pubtator_variant | - | - | 119/1 | - |
| Accumulated | exact_lexical | 39,127/7 | 42,495/12 | 37,045/7 | - |
| Fresh | strict | 2,400/7 | 897/7 | 1,677/7 | 14,219/7 |
| Fresh | pubtator_variant | - | - | 101/1 | - |
| Fresh | exact_lexical | 3,137/7 | 11,636/12 | 5,310/7 | 26,439/3 |

`source_citation` was skipped in both runs. The representative accumulated
cache still exceeds 60 seconds, so the premise survives 1167; the fresh result
also shows that cache state materially changes which route dominates. The
original pre-1167 evidence remains in
`sdlc/issues/2026-09-04-variant-article-union-can-exceed-two-minutes.md` at
commit `f8ff2a78`, but it is no longer the acceptance baseline.

Today `strict_provider_candidates` waits for provider plans serially, while
federated alias work can hold a completed provider leg inside `join_all` until
slower siblings settle. The outer response therefore cannot retain all work
that was already complete at an earlier deadline. Typed MCP always turns a
settled batch outcome into a successful tool result, and raw MCP discards the
CLI exit code in `outcome_to_mcp_output`; both need an explicit terminal
contract rather than inference from transport behavior.

## Deadline ownership

Local command option validation, input reading and JSON decoding, the 1-10 item
bound, request-ID checks, and item identity syntax validation happen before the
clock starts. Immediately afterward, before variant resolution or any source
client construction, the owning entry point creates one
`VariantArticleDeadline` with an absolute `tokio::time::Instant` 60 seconds in
the future. The duration and monotonic clock are injectable for tests. Every
single-item and batch `VariantArticleExecutionContext` holds the same deadline.
Invalid syntax still fails before provider contact and is never reclassified as
a timeout.

The invocation deadline covers resolution, canonical equivalence, client and
cache-manager construction, every route/alias/page, response-body parsing,
transport retries and backoff, provider rate-limit waits, cache reads and
writes performed for those requests, enrichment, PubTator identity
verification, and ClinGen LDH verification. Once it expires, no new work is
admitted and all invocation-owned futures are dropped and settled. Only the
already bounded in-memory merge, deduplication, ranking, pagination, status
construction, serialization, and rendering may follow. Work initiated solely
by this invocation in the provider or construction path must not continue in a
detached task after its response. The cache manager's existing independently
owned background eviction lifecycle is not provider work and remains unchanged.

This deadline is distinct from an inner provider timeout. Existing provider
connect/request/source timeouts and retry counts remain. Each provider attempt,
retry sleep, and rate-limit wait is clipped to `min(its existing limit,
deadline.remaining())`:

- If an existing provider timeout expires while invocation time remains, the
  request unit is a non-deadline provider failure. The route/source aggregate
  is `unavailable` when no unit succeeded, or `degraded` when earlier pages
  from that provider were committed. Its reason is `provider_timeout`.
- If the absolute invocation deadline wins, including an equal-boundary race,
  the unit contributes `timed_out` with reason `invocation_deadline`.
- A unit never admitted because no invocation time remains contributes
  `not_attempted` with reason `invocation_deadline`. A logical-call cap uses
  `not_attempted` with reason `logical_work_cap`; it is not a timeout.

An outer `tokio::time::timeout(Duration::from_secs(60), whole_request)` alone
does not satisfy this contract: it would lose completed sibling results, blur
the two timeout causes, and cannot preempt the synchronous cache construction
path.

## Bounded execution and incremental commitment

Keep `ITEM_CONCURRENCY_LIMIT` at two. Add one invocation-wide
`PROVIDER_CONCURRENCY_LIMIT` of 10, shared by all active items. A permit covers
client acquisition, any rate-limit wait, request/retry/body parse, cache work,
and commitment of that provider unit. Alias and provider counts cannot enlarge
the cap. Eligible units are admitted in input-item order, then stable
route/provider/query-plan order. Existing provider rate limiters remain the
lower effective limit.

Replace whole-route `join_all` settlement with a bounded completion stream.
Each fully received and decoded provider leg or page is committed immediately
to the item accumulator before any later page, alias, provider, enrichment, or
verification wait. A partial response body or failed decode is never committed.
At deadline, ranking runs once over the committed candidate pool. Every row
keeps its existing plan position and provenance, so varying completion order
cannot change item order, status order, deduplication, ranks, pagination, or
candidate trace. Enrichment and identity verification are overlays: timing out
an overlay retains the base row and records that overlay route as incomplete.
This is internal incremental commitment, not streaming; JSON and Markdown are
still emitted once after the invocation settles.

## Source status aggregation

`source_status` remains the public route/source ledger and adds `timed_out` to
the closed enum. Keep the existing `route`, `source`, `status`, and sanitized
`detail` fields. Add an additive `work` object with nonnegative counts
`planned`, `ok`, `degraded`, `unavailable`, `timed_out`, and `not_attempted`;
the six terminal counts sum to `planned`. Add a sorted, deduplicated
`reason_codes` array drawn from `provider_timeout`, `provider_error`,
`invocation_deadline`, `logical_work_cap`, `identity_inapplicable`, and
`configuration`. Do not expose raw provider errors, URLs, cache paths, or
unbounded text.

A work unit is one planned provider request/page for one query alias; a
continuation page becomes planned when the completed response exposes that
continuation. Each unit receives exactly one terminal count. Local overlays
that perform provider work are units under their enrichment or verification
route.

Aggregate one row for each planned route/source in stable route/source order.
Build the strategy's route/source skeleton before resolution; if a prerequisite
times out before query text can be constructed, its dependent entries still
appear as `not_attempted` with `invocation_deadline`. Provider-query text is
never fabricated. Apply this precedence exactly:

1. `skipped` when the strategy, identity result, or configuration makes the
   entire route/source inapplicable; its work counts are zero.
2. `not_attempted` when work was eligible but no unit started. The reason codes
   distinguish invocation deadline from logical cap.
3. `timed_out` when at least one unit hit the invocation deadline, or when some
   units completed but the invocation deadline left later eligible units
   unstarted. Previously committed rows do not lower this status.
4. `degraded` when no invocation deadline was involved and at least one unit
   completed successfully, but another unit was degraded, unavailable, or
   omitted by a logical cap. A successful healthy-empty unit counts as a
   successful unit for this aggregation.
5. `unavailable` when no unit completed successfully and at least one admitted
   unit ended in a non-deadline failure. Logical-cap omissions, if any, remain
   visible in `work` and `reason_codes`.
6. `ok` only when every planned unit completed successfully before both its
   provider timeout and the invocation deadline. Successful zero-row units are
   `ok`.

Thus a route with rows plus an invocation timeout is `timed_out`; rows plus an
inner provider timeout is `degraded`; only inner failures and no successful
unit is `unavailable`; and a wholly unstarted route is `not_attempted`.
Deadline involvement makes the affected item and batch `complete: false` and
`truncated: true`, and keeps `pagination.total: null`.

For every non-`ok`, non-`skipped` aggregate, `detail` is the deterministic
bounded sentence
`N planned: A ok, B degraded, C unavailable, D timed out, E not attempted
(reason_codes)` with every count present and sorted reason codes. `skipped`
retains its bounded applicability explanation; `ok` needs no detail.

## Cache construction contract

The deadline-aware variant path must use these explicit APIs, while existing
non-variant callers may retain the current wrappers:

- Async `shared_client_with_deadline`,
  `provider_url_client_with_deadline`, and
  `semantic_scholar_provider_client_with_deadline` feed an async
  `build_http_client_with_config_deadline`. Every variant-literature source
  constructor uses one of these APIs; none may fall back to the synchronous
  constructor after the clock starts.
- `build_http_client_with_config_deadline` passes the same deadline through
  `migrate_http_cache_with_deadline`,
  `ensure_body_limited_cache_epoch_with_deadline`,
  `lock_cache_shared_until`, `secure_managed_tree_until`, and
  `SizeAwareCacheManager::new_with_deadline`. A timed-out construction is not
  published into the shared-client cell and is attributed to the source unit
  that requested it. Deadline expiry propagates past the existing non-fatal
  migration warning seam instead of being logged and ignored.
- `lock_cache_shared_until`, `lock_cache_maintenance_until`, the epoch-file
  lock, and deadline-scoped `lock_cache_key_until` use `fs2::try_lock_shared`
  or `try_lock_exclusive` plus deadline-clipped async polling of at most 10 ms.
  They preserve 1167's cache-wide-before-key lock order and release every guard
  on timeout. The opportunistic constructor cleanup still uses immediate
  try-lock semantics when contention is not itself the awaited operation.
- `secure_managed_tree_until` checks the deadline before opening the root,
  before each `read_dir`, and before each entry is validated or repaired.
  `snapshot_cache_until` checks before each `cacache::list_sync` result and
  content-tree entry. `execute_cache_clean_until` checks before planning and
  before each key/blob removal. `estimate_cache_bytes_fast_until` checks before
  each content-tree entry. Deadline expiry is a distinct error and must not be
  swallowed as an ordinary scan error or converted into a zero-byte estimate.
- `SizeAwareCacheManager::new_with_deadline` uses only those bounded cleanup and
  estimate paths. Cache `put`/`delete` reached from a scoped variant request use
  `lock_cache_key_until`; ordinary callers retain their existing behavior.

Do not implement these APIs by wrapping current blocking locks or recursive
scans in `spawn_blocking`: dropping a timed-out join handle detaches the work,
allows it to retain a 1167 lock, and violates the invocation deadline. Checks
occur between filesystem entries and atomic filesystem operations; on expiry,
the current operation reaches its safe return boundary, all guards drop, no
partially constructed client is published, and no subsequent scan, cleanup, or
provider operation begins. Cache permission, symlink/reparse-point, hard-link,
atomic-write, and lock-order guarantees from 1167 remain unchanged.

## Surface semantics

A usable item is either one with at least one article row after existing
identity/filter/pagination rules, or a fully complete healthy-empty search. An
incomplete zero-row item is not usable. Add the same optional bounded `error`
shape used by batch items to the single JSON response and always serialize the
key; it is `null` for a usable row set and for healthy empty,
`deadline_exceeded` when deadline involvement leaves no row, and
`source_unavailable` for wholly unavailable non-deadline work.

- A partial single item with one or more rows returns those rows with
  `error: null`, `complete: false`, `truncated: true`, total null, named source
  statuses, and success. CLI JSON writes that object to stdout and exits zero.
  CLI Markdown writes rows plus an `Incomplete coverage` list of every
  non-`ok`/non-`skipped` route/source and exits zero.
- A complete healthy-empty search at offset zero returns `results: []`,
  `error: null`, `complete: true`, `truncated: false`, `pagination.total: 0`,
  `has_more: false`, and `ok`/`skipped` statuses. CLI JSON writes it to stdout
  and exits zero. CLI Markdown writes the existing no-articles message to
  stdout and exits zero; it must not say unavailable or timed out.
- An incomplete zero-row single item returns the structured JSON object on
  stdout and exits one. Markdown writes a bounded actionable failure to stderr,
  names each unfinished route/source and state, distinguishes invocation
  deadline from provider failure, and exits one. It does not print a false
  no-articles conclusion.
- A batch item follows the same three cases. Row-bearing partial and complete
  healthy-empty items have `error: null`; incomplete zero-row items have the
  corresponding bounded item error. Batch `complete` is true only when every
  item is complete, and `truncated` is true when any item is truncated. The CLI
  batch exits zero when at least one item is usable and exits one only when all
  items are errors; per-item errors remain visible either way. Item order is
  input order.
- Raw MCP `biomcp` mirrors the CLI single-item result despite having no process
  exit: JSON or Markdown success is `isError: false`; incomplete zero-row
  failure is `isError: true` while preserving the same structured JSON or
  actionable Markdown content. The MCP execution seam therefore preserves the
  `CommandOutcome` error bit instead of discarding its exit code. Raw MCP
  continues to reject server-local `--input`.
- Typed `variant_articles` returns the same batch JSON text with
  `isError: false` when at least one item is usable and `isError: true` when all
  items are errors. It does not replace a structured failed batch with a plain
  `Error:` string. Schema/argument failures remain MCP `invalid_params` before
  the deadline starts.

## Public compatibility and completion evidence

The public change is additive: retain existing response, pagination,
`source_status`, item-error, debug-plan, provenance, identity, and ranking
fields and meanings. Add only the single-response `error`, `timed_out` enum
member, status `work`/`reason_codes`, and debug deadline metadata. Both single
and batch `--debug-plan` expose `deadline.scope: "invocation"`,
`deadline.limit_ms: 60000`, `deadline.exhausted`, and
`deadline.provider_concurrency_limit: 10`; batch item plans reference that same
invocation deadline rather than presenting per-item budgets. Existing
`calls`, `pages`, `latency_ms`, logical budgets, `stopped_routes`, and candidate
trace remain, with `calls` counting admitted provider calls and `pages`
counting only fully decoded, committed pages.

- Update the serialized public contract tests, typed MCP schema/surface tests,
  CLI reference, MCP server reference, `docs/how-to/find-articles.md`, and the
  relevant source pages to document 60 seconds, healthy empty, partial/error
  behavior, the expanded status enum, and raw versus typed MCP behavior.
- Deterministic tests inject short deadlines and controlled futures or paused
  Tokio time, never public providers or production-length sleeps. A fast page
  is committed before a sibling invocation timeout; queued work is
  `not_attempted`; an inner provider timeout stays `unavailable` or `degraded`;
  completion-order permutations produce identical rows, ranks, statuses, and
  traces.
- A 10-item fixture proves one shared deadline, the two-item and ten-provider
  concurrency limits, input-order settlement, no admission after expiry, and
  mixed usable/error batch behavior. Separate fixtures expire in resolution,
  each cache construction scan/lock API, retry/backoff, rate-limit wait,
  enrichment, PubTator verification, and LDH verification. They prove prompt
  settlement, released cache locks, no published partial client, and no
  invocation-owned work after response.
- Surface fixtures pin partial rows, complete healthy empty, incomplete zero
  rows, mixed batch, and all-failed batch across CLI JSON, CLI Markdown, raw MCP
  JSON/Markdown, and typed MCP, including stdout/stderr, exit code, `isError`,
  item errors, totals, and source-status aggregation.
- Existing annotation and lexical strategies remain independently usable, and
  all identity, canonical-equivalence, `--verify-identity`, `--confirmed-only`,
  provider-rate, cache-security, pagination, provenance, candidate-order,
  deduplication, and ranking regressions keep passing.

## Boundaries and dependencies

Keep dependency 1167 recorded; its landed lock and traversal guarantees are
prerequisites to the deadline-aware cache variants above. Ticket 1164 remains
downstream of this ticket.

The deadline is an additional ceiling, not a replacement for the 10
exact-alias cap, item/request call budgets, fetch/result caps,
identity-verification reservations, LDH limits, provider timeouts/retry counts,
or provider rate limits. This ticket does not change variant identity,
canonical-equivalence, confirmed-only filtering, deduplication, ranking inputs
or order, relevance, cache layout/eviction/security, or the purpose of explicit
annotation and lexical strategies. It does not add streaming output or promise
that every provider finishes.
