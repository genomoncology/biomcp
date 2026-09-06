---
flow: record
priority: 8
deps: [1167, 1173]
---

# Return variant literature within one invocation-wide work deadline

## Goal

One `variant articles` invocation has one monotonic 60-second provider-work
deadline and returns every article page or provider leg completed before that
deadline. The same absolute deadline covers a single CLI query and the complete
1-10 item batch used by CLI `--input` and the typed `variant_articles` MCP tool.
It is never reset per item, route, provider, retry, page, enrichment, or
verification phase.

## Result

Implemented one task-local monotonic deadline created after syntax and option
validation and shared by a single request or all ordered batch items. The
deadline is copied into request extensions, clips the complete middleware send,
retry-after and rate-limit waits, bounds cache trait operations, and adds
deadline checks to recursive security, snapshot, cleanup, estimate, and lock
paths. Variant cache locks use deadline-aware try-lock polling and do not enter
the ordinary detached `spawn_blocking` path. One shared ten-permit provider
semaphore and the existing two-item buffered executor preserve the invocation
and item concurrency ceilings. Completed alias futures are consumed
incrementally rather than held behind `join_all`.

The public response now includes the additive single-item `error`, `timed_out`
source state, terminal work counts/reason codes, and invocation deadline debug
metadata. Incomplete pagination uses an unknown total and retryable `has_more`;
healthy empty and known complete pages remain usable. Mixed batches succeed
when any item is usable. CLI Markdown names incomplete routes, JSON keeps the
structured envelope, typed MCP marks only all-error batches as errors, and raw
MCP carries an internal variant-only disposition without reinterpreting other
nonzero command outcomes. CLI, MCP, how-to, provider documentation, and the
executable healthy-empty fixture were updated.

Focused evidence passed for the original implementation: `cargo check
--no-default-features`; Clippy with warnings denied; `make lint` including
quality/source-size ratchets; the isolated deadline, source-precedence,
provider-cap, task-local-isolation, raw-MCP, mixed-batch, CLI parsing,
cache-manager, JSON hard-failure, documentation, MCP-catalog, package-boundary,
healthy-empty, and union fixture lanes. Independent review subsequently ran all
47 variant-search module tests, including the 119-second real-capture case;
all passed, confirming the earlier 45/46 report was only shared compile/cache
contention.

Review remediation now keeps the provider permit around the entire downstream
cache/retry/transport/body/commit chain while retaining per-attempt rate
pacing; uses explicit async deadline-aware constructors for every
variant-literature provider; finishes cache metadata/security hardening after
an atomic put even if the clock expires; rebuilds source status only after
identity, enrichment, and LDH events; records real MyVariant hydration and LDH
incompleteness; and distinguishes invocation deadline, provider timeout,
configuration, identity-inapplicable, and logical-cap reasons without a
synthetic `internal` provider. CLI Markdown now distinguishes complete
offset-beyond-total pages from healthy-empty searches and gives incomplete-zero
items an actionable route/state failure. Added deterministic actual-HTTP
ten-provider concurrency, cache safe-return, terminal-reason, and executable
offset Markdown coverage. Remediation evidence includes `cargo check
--no-default-features`, the actual-HTTP provider concurrency regression, and 31
focused Python documentation contracts; two additional Python tests required a
prepared `target/release/biomcp` and were not run as passing claims. Full
`make test` and `make spec` are not claimed.

A second independent review rejected four remaining implementation gaps. This
remediation moves provider admission out of request middleware and into the
execution ledger: the unit is reserved before cold client construction, owns
one invocation-shared permit through cache/retry/body decoding and accumulator
commit, and records cancellation as the actual source's provider timeout or
invocation timeout. It removes the synthetic `internal` source entirely.
Deadline-aware cold construction now polls the cache-epoch lock instead of
blocking on `lock_exclusive`. The cache manager arms a publication guard before
CACache can expose an entry; production request middleware observes that guard,
drives metadata/security finalization to its stable success or fail-closed
result, and then resumes normal deadline cancellation. Full repository gates
remain unclaimed.

The final remediation gives each production middleware send one shared cache
publication marker. Both rate-limit layers and the request boundary observe
that same marker, so expiry remains cancellable before publication but awaits
cache commit, fail-closed finalization, and key-lock release after it is armed.
A paused-time production-middleware test uses `Notify` at the real writer
boundary and proves an unrelated unarmed deadline still expires. Cold-client
migration and epoch initialization now acquire the maintenance lock by
deadline-clipped async polling; deterministic contention proves expiry leaves
the legacy tree untouched, releases the guard, and permits a complete retry.
The strategy route skeleton is now selected before resolution, and a
zero-deadline production-path matrix pins every union, annotation, and lexical
row to exact `planned: 1`, `not_attempted: 1`, `invocation_deadline` state.
Focused Rust tests for those regressions passed; full `make test` and `make
spec` remain integration gates and are not claimed here.

## Review

- Design review: accepted before implementation.
- Code review: pending independent review of this commit.

## Second remediation evidence

- `cargo check --no-default-features`: passed without warnings.
- `cargo clippy --no-default-features --lib -- -D warnings`: passed.
- Production-shaped post-publication fail-closed test: passed.
- Contended cold cache-epoch deadline test: passed.
- Actual-HTTP provider-unit concurrency test: passed with an HTTP peak of ten,
  a post-body commit peak of ten, and twenty matching terminal ledger events.
- Backend fifty-call bound, invocation-deadline settlement, source attribution,
  terminal precedence, cache-manager safe-return, and fail-closed client matrix:
  passed.
- `tools/check-quality-ratchet.sh`, formatting, and diff whitespace: passed.
  No tracked package file was added, preserving the 1,300-file boundary.

## Prerequisite integration evidence

Rebased onto `ecaeb921` after ticket 1173 landed. The cache integration keeps
1173's injected manager factory, get/write observers, and stable fail-closed
post-write metadata context. Deadline cancellation covers cache work before
CACache publishes an entry; after publication, metadata hardening completes
inside the existing safe-return boundary before the operation returns. The
unused synchronous ClinGen LDH constructor was removed rather than adding a
new dead-code exception, while variant-literature uses the required
deadline-aware constructor.

Focused integration validation passed all 24 cache-manager tests, both cached
client post-write and bypass tests, all 47 variant-search tests, the task-local
deadline isolation test, the real downstream HTTP provider-concurrency test,
and the raw-MCP disposition seam test. Formatting, diff whitespace, the
quality/source-size ratchets, and the exact 1,300-file package inventory pass.
The source-size inventory now records the exact post-integration counts rather
than the stale pre-remediation counts. Full repository gates were not run.

## Final review remediation

The prior downstream-HTTP concurrency test proves the shared ten-permit cap,
but by itself does not prove production accumulator settlement. The final
remediation returns zero-query CAR equivalence as `inapplicable` before budget
reservation or client construction, adds provider-unit commit callbacks that
retain the permit until synchronous decoded-result mutation completes, and
uses them for CAR observation/item insertion and PubTator token-plan commits.
MyVariant citation hydration now retains its unit through PMID candidate and
provenance construction. Existing backend request commits continue to include
provider decoding, filtering, transformation, deduplication, and page-vector
mutation before their one terminal event.

Focused evidence covers the zero-query zero-unit CAR boundary, but an initial
generic route-label test did not prove those production paths and was removed.
The follow-up production tests prove two expired, planned CAR identities yield
exactly two deadline-omitted events rather than an inapplicable result or three
events; prove a real PubTator search page commits its decoded page; and prove a
completed PubTator visible-row enrichment remains returned when the later
Europe PMC leg fails, with exactly one success and one unavailable event. The
existing real CAR/LDH capture path remains extended through MyVariant citation
hydration. Full `make test`, `make spec`, and release gates remain unclaimed.

The coordinating full `make test` later stopped after 68 passing Rust tests
because `put_finishes_security_boundary_when_deadline_expires_after_publish`
timed out. Its 30-millisecond wall-clock setup did not establish whether the
CACache index entry had been published, and it exposed that the first
remediation armed the per-put marker before CACache serialization and content
writing rather than at publication. The interrupted full gate remains failed
evidence and was not rerun here.

The final cache-boundary remediation preserves CACache's exact bincode response
and policy envelope but uses its public async `Writer`: serialization, writer
creation, and body writing remain deadline-cancellable; the per-put marker is
armed only when entering `Writer::commit`, whose public API owns content close,
size/integrity validation, and the atomic index insertion. Metadata lookup,
security hardening, accounting, and eviction scheduling still settle under the
same per-put shield. A real manager-put transition regression expires once
before publication and proves an error with no index entry, then expires at
commit and proves successful settlement plus compatible readback. That test
passed individually and in 20 repeated runs. The paused-clock concurrent test
still proves an armed put does not shield unrelated or unarmed operations.

The subsequent full test gate passed the cache transition regression but found
two existing CLI execution tests overflowing the fixed 8 MiB
`biomcp-cli-execute` worker stack. An immutable A/B reproduced the disease test
failure at pre-Writer commit `ecfbc1bb` while current main passed, proving this
was not caused by the explicit CACache writer. Ticket 1150 had enlarged article
and variant async handlers retained inside the shared CLI dispatcher state.
Boxing those handler arms and the fallback `run` boundary restores bounded
dispatcher state without changing the worker stack or cache behavior. The
exact disease-card and ticket-1120 raw-MCP reproductions now pass on the normal
worker stack. The coordinating full gate remains unclaimed pending rerun.

## Current facts

Ticket 1167 has landed. It removed recursive whole-cache repair after every
write but intentionally retained cold-construction repair, cleanup, and size
scans. These measurements used the prepared release-style binary built from
code commit `8afd56cf` after ticket 1154; ticket 1167 is in that commit's
ancestry. Commit `e09b0393` only recorded the measurements in documentation and
is not the code revision that was executed. The accumulated invocation was:

```bash
/usr/bin/time -p biomcp --json variant articles "ODC1 c.1342A>T" --limit 10 --debug-plan
```

The fresh invocation used the same prepared binary and arguments with a newly
created empty cache root:

```bash
fresh_cache="$(mktemp -d)"
BIOMCP_CACHE_DIR="$fresh_cache" /usr/bin/time -p biomcp --json variant articles "ODC1 c.1342A>T" --limit 10 --debug-plan
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

`source_citation` was skipped in both runs. The accumulated run used the
default cache, the same cache population previously audited at 636 MB and
75,853 index files; those size/count figures are the prior audit, not a
recount taken at the instant of this run. The fresh run set `BIOMCP_CACHE_DIR`
to an empty directory created by `mktemp`. Both commands emitted their first
JSON only after all work completed. The representative accumulated run still
exceeds 60 seconds, so the premise survives 1167 even though a 60-second
implementation would intentionally cut off its late work; the fresh result
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
  from that provider were committed, absent any invocation-deadline condition.
  Its reason is `provider_timeout`.
- If the absolute invocation deadline wins, including an equal-boundary race,
  the unit contributes `timed_out` with reason `invocation_deadline`.
- A unit never admitted because no invocation time remains contributes
  `not_attempted` with reason `invocation_deadline`. A logical-call cap uses
  `not_attempted` with reason `logical_work_cap`; it is not a timeout.

An outer `tokio::time::timeout(Duration::from_secs(60), whole_request)` alone
does not satisfy this contract: it would lose completed sibling results, blur
the two timeout causes, and cannot preempt the synchronous cache construction
path.

### Deadline propagation seam

Implement one Tokio `task_local!` value, `VARIANT_ARTICLE_DEADLINE`, and enter
it exactly once around the complete post-validation single or batch future via
`with_variant_article_deadline(deadline, future)`. Cache operations read
`current_variant_article_deadline()` when each operation is called. They must
never store an invocation deadline in `HTTP_CLIENT`,
`SizeAwareCacheManager`, or any other global/client field. The task-local is
required because `http_cache::CacheManager::{get, put, delete}` receives only
the cache key/response/policy and cannot see `http::Extensions`.

Every request built for this command also carries the same cloned
`VariantArticleDeadline` in `http::Extensions` via
`RequestBuilder::with_extension`. BioMCP's rate-limit,
Retry-After/retry, and provider-pool middleware read that extension and clip
their waits. The extension handles request middleware; the task-local handles
the third-party cache-manager interface. Neither is a second clock or a reset.
Provider work stays in the scoped future (a bounded `FuturesUnordered` or
equivalent), not in unscoped `tokio::spawn` tasks. If an existing abstraction
unavoidably spawns, it must explicitly re-enter the same task-local scope and
be joined/dropped before settlement; no invocation-owned detached task is
permitted.

`shared_client_with_deadline` first checks `HTTP_CLIENT`. If already
initialized, it returns the clone without construction or cold scans; its
reused cache manager still observes the current invocation's task-local value
on every `get`, `put`, and `delete`. “Without cold scans” here means it skips
the constructor's migration, cleanup snapshot, and initial size estimate; a
warm cache `get` still performs the existing `secure_managed_tree` security
traversal and that traversal is deadline-aware below. If uninitialized, the
client runs the bounded constructor below and publishes only a fully
completed, deadline-free client; a losing concurrent initializer discards its
extra client. Provider-URL and Semantic Scholar client construction receive
the explicit deadline too. The global provider `RateLimiter` remains shared
and unchanged, while each wait on it is clipped by the calling request's
extension. This preserves shared rate limiting without stale deadlines or
cross-invocation cancellation.

Tests run two overlapping invocations with different injected deadlines
through one already initialized client and cache manager. They prove that the
short invocation expires independently, the long invocation continues, and a
later invocation does not inherit either deadline. A separate cold-client test
proves construction scans consume the caller's deadline; a warm reused-client
test proves those scans are not repeated and cache `get`/`put`/`delete` still
honor the new caller's deadline.

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

## Route skeleton and work planning

After syntax validation and deadline creation, create each item's stable
route/source skeleton before resolution. A skeleton row is ordering and
applicability metadata, not automatically a work unit. Materialize units at the
planning points below, before admitting any unit from that group:

The immediate base units are the one initial `resolution/myvariant` unit and
the zero-to-two syntactically derived `canonical_equivalence/clingen_car`
units. Every other row is a conditional skeleton row whose exact units are
materialized only at the stated prerequisite boundary.

| Route/source skeleton row | Applicability and planning point | Exact units |
|---|---|---|
| `resolution/myvariant` | Always; choose the shape from the parsed requested identity before client construction | One `get_all` for a genomic-accession identity, otherwise the first 50-hit search page. Each later search page is added only after the committed prior page proves the scan is not exhaustive, up to the existing 1,000-candidate cap. |
| `canonical_equivalence/clingen_car` | Always present; derive its queries from the parsed identity before resolution starts | Zero, one, or two normalize requests: one versioned RefSeq transcript/coding query when valid and one authoritative genomic query when valid. Zero queries is inapplicable. |
| `strict/{pubmed,europepmc,pubtator,semanticscholar}` | Union when validation is not contradictory, and lexical after a resolved identity; after resolution/CAR produces the final selected exact aliases | Per selected alias/provider: Europe PMC first search page; Semantic Scholar one bulk-phrase search; PubMed one ESearch followed, only when IDs were returned, by one ESummary; PubTator one entity-token resolution followed, only when a token exists, by the first search page for that first token. Provider continuation pages/exchanges are added after the prior decoded response requires them. |
| `pubtator_variant/pubtator` | Union or annotation after a resolved compatible identity | One entity-token resolution, then one first search page for each returned token; each continuation page is added after the preceding decoded page requires it. |
| `exact_lexical/{pubmed,europepmc,pubtator,semanticscholar}` | Union or lexical after a resolved compatible identity; plan from the final selected exact aliases | Per alias, the same federated source legs as above: Europe PMC first page, PubTator first page, Semantic Scholar one search, and PubMed ESearch plus conditional ESummary. Continuations are added from decoded page metadata. |
| `best_effort_free_text/{pubmed,europepmc,pubtator,semanticscholar}` | Union only, after resolution concludes unresolved or contradictory; plan from the bounded fallback aliases | Per fallback alias, the same four federated source legs and conditional PubMed/page expansion as `exact_lexical`. Existing debug `provider_queries.route: "discovery"` remains the query-plan descriptor; the executed/status route remains `best_effort_free_text`. |
| `source_citation/myvariant` | Union only, after resolved provider validation is confirmed and a retained MyVariant hit exists | One `get_all` hydration only if the retained hit lacks loaded CIViC data. If CIViC data is already loaded, local PMID extraction is applicable healthy work with zero provider units. |
| `enrichment/semanticscholar` | After ranking selects visible base rows and their unique lookup IDs are known | One batch lookup per existing `SEMANTIC_SCHOLAR_BATCH_LOOKUP_MAX_IDS` chunk. No lookup IDs means applicable healthy work with zero units. |
| `enrichment/pubtator` and `enrichment/europepmc` | After Semantic Scholar enrichment is committed, for each still-eligible visible PubMed row | One PubTator export per row. Add one Europe PMC PMID lookup after a successful PubTator result needing metadata or a PubTator 400/404 lag fallback; do not add it after another terminal PubTator failure. |
| `identity_verification/pubtator` | Only with `--verify-identity`; after local abstract verification identifies the bounded candidates still requiring fetched verification | One export for each such candidate: visible-page candidates normally, or the existing bounded candidate set for `--confirmed-only`. |
| `clingen_ldh_medium/clingen_ldh` | Only with `--verify-identity` and an eligible conclusively resolved CAid, after PubTator verification | Exactly one medium request. |
| `clingen_ldh_direct/clingen_ldh` | Only after the committed medium result identifies eligible matching PMCIDs/IRIs | One request per selected IRI, preserving at most two IRIs for each of at most five candidates, the existing ten-request cap, and the existing aggregate body-byte limit. |

The variant command does not expose an explicit LitSense2 source selection, and
the current `all` federated plan does not enable LitSense2, so no LitSense2 row
or unit is added. A provider exchange is one unit; thus PubMed ESearch and
ESummary are separate units, as are PubTator token resolution and article
search. Client/cache construction is charged to the first unit that needs that
client and does not create a synthetic extra unit.

At terminalization, a skeleton row whose completed prerequisite proves it
inapplicable is `skipped` with `planned: 0` and all five terminal counts zero.
An applicable row that conclusively needs no provider exchange (for example,
already hydrated CIViC data or no enrichment lookup IDs) is `ok` with the same
zero counts; `ok` rather than `skipped` records that its prerequisite completed
and healthy zero-work was the answer. If the invocation deadline prevents the
prerequisite from deciding applicability or constructing query text, materialize
one dependency-blocked leg for that row (`planned: 1`, `not_attempted: 1`,
reason `invocation_deadline`) without fabricating a query. Logical caps likewise
materialize each known omitted leg as `not_attempted`; unknown continuation
pages are never invented after a failed or timed-out page.

## Source status aggregation

`source_status` remains the public route/source ledger and adds `timed_out` to
the closed enum. Keep the existing `route`, `source`, `status`, and sanitized
`detail` fields. Add an additive `work` object with nonnegative counts
`planned`, `ok`, `degraded`, `unavailable`, `timed_out`, and `not_attempted`;
the five terminal counts sum to `planned`. Add a sorted, deduplicated
`reason_codes` array drawn from `provider_timeout`, `provider_error`,
`invocation_deadline`, `logical_work_cap`, `identity_inapplicable`, and
`configuration`. Do not expose raw provider errors, URLs, cache paths, or
unbounded text.

A work unit is one planned provider exchange/leg from the table above. A fully
decoded, semantically complete response is `ok`, including a healthy zero-row
response. A decoded response that the provider contract explicitly marks
usable but non-exhaustive is `degraded`. A non-deadline failure with no
committable response is `unavailable`; an active unit cancelled by the
invocation clock is `timed_out`; and a materialized unit never admitted is
`not_attempted`. Each unit receives exactly one terminal count.

Emit one aggregate for every row in the applicable strategy skeleton, even
when it has zero units, in stable route/source order. Let `started = ok +
degraded + unavailable + timed_out` and let a deadline omission mean a
`not_attempted` unit carrying `invocation_deadline`. Apply this exhaustive
precedence exactly:

1. `skipped` if a completed prerequisite makes the entire row inapplicable;
   `planned` and every terminal count are zero.
2. `ok` if it is applicable healthy zero-work (`planned: 0`), or if
   `ok == planned > 0`.
3. `not_attempted` if `planned > 0` and `started == 0`, regardless of whether
   the omission reason is the invocation deadline or a logical cap.
4. `timed_out` if `started > 0` and either `timed_out > 0` or any later unit was
   omitted for `invocation_deadline`. Previously committed rows do not lower
   this status.
5. `degraded` if no invocation-deadline condition applies, at least one unit
   is usable (`ok + degraded > 0`), and the row is not wholly `ok`. This covers
   any mixture with provider failure or logical-cap omission.
6. `unavailable` if no unit is usable and at least one admitted unit is
   `unavailable`; logical-cap omissions may coexist and remain visible.

These rules settle every mixture: only deadline-omitted work with nothing
started is `not_attempted`; any active/late deadline involvement is
`timed_out`; usable work plus only non-deadline defects is `degraded`; and
non-deadline failures with no usable work are `unavailable`. A partial response
body or failed decode is never usable or `degraded`.

Thus a route with rows plus an invocation timeout is `timed_out`; rows plus an
inner provider timeout is `degraded`; only inner failures and no usable unit is
`unavailable`; and a wholly unstarted route is `not_attempted`.
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
  constructor after the clock starts. These APIs accept the explicit deadline
  for construction, but the published client does not retain it; request
  builders attach the current deadline extension and cache operations load the
  task-local deadline at call time as specified above.
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
  estimate paths. In the reused manager, all three trait operations dispatch at
  call time: `get` uses `secure_managed_tree_until` and a deadline-clipped
  `CACacheManager::get`; `put` uses `lock_cache_key_until`,
  `prepare_write_paths_until`, a deadline-clipped inner put/metadata read, and
  `secure_written_content_until`; `delete` uses `lock_cache_key_until`,
  `secure_managed_tree_until`, and a deadline-clipped inner delete. Each helper
  obtains the task-local deadline only for the current call; when no scope is
  active, ordinary callers retain their current behavior. Filesystem-space
  inspection and eviction-trigger bookkeeping check the deadline before and
  after their bounded synchronous operations.

Do not implement these APIs by wrapping current blocking locks or recursive
scans in `spawn_blocking`: dropping a timed-out join handle detaches the work,
allows it to retain a 1167 lock, and violates the invocation deadline. Checks
occur between filesystem entries and atomic filesystem operations; on expiry,
the current operation reaches its safe return boundary, all guards drop, no
partially constructed client is published, and no subsequent scan, cleanup, or
provider operation begins. Deadline cancellation of an inner cacache future
must leave only its existing private temporary/atomic-write state; required
permission/integrity finalization that has crossed an atomic commit is the safe
return boundary and must complete before its guard is released. Cache
permission, symlink/reparse-point, hard-link, atomic-write, and lock-order
guarantees from 1167 remain unchanged.

## Surface semantics

A usable item is one with at least one article row after existing
identity/filter/pagination rules, a fully complete healthy-empty search, or a
fully settled empty page whose requested offset is beyond a known nonzero
total. An incomplete zero-row item is not usable. Add the same optional bounded
`error` shape used by batch items to the single JSON response and always
serialize the key; it is `null` for every usable result,
`deadline_exceeded` when deadline involvement leaves no row, and
`source_unavailable` for wholly unavailable non-deadline work.

Pagination is derived only after terminal coverage is known. Every incomplete
item, whether it has partial rows or zero rows and regardless of requested
offset, has `complete: false`, `truncated: true`, `pagination.total: null`, and
`has_more: true`; here `has_more` means a retry may discover more, not that a
cursor is known. Every complete item has a known total and computes `has_more`
as `offset + returned < total`. `next_page_token` remains null.

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
- A fully settled request with known `total: N > 0` and `offset >= N` returns
  `results: []`, `error: null`, `complete: true`, `truncated: true`,
  `pagination.total: N`, and `has_more: false`. CLI JSON exits zero. CLI
  Markdown writes `No articles on this page` plus the known total and requested
  offset to stdout and exits zero; it does not claim that the search itself had
  no articles. Raw and typed MCP treat this as success, and a batch item counts
  as usable.
- An incomplete zero-row single item returns the structured JSON object on
  stdout and exits one. Markdown writes a bounded actionable failure to stderr,
  names each unfinished route/source and state, distinguishes invocation
  deadline from provider failure, and exits one. It does not print a false
  no-articles conclusion.
- A batch item follows the same four cases. Row-bearing partial, complete
  healthy-empty, and settled offset-beyond-total items have `error: null`;
  incomplete zero-row items have the corresponding bounded item error. Batch
  `complete` is true only when every item is complete, and `truncated` is true
  when any item is truncated. The CLI batch exits zero when at least one item
  is usable and exits one only when all items are errors; per-item errors remain
  visible either way. Item order is input order.
- Raw MCP `biomcp` mirrors the CLI single-item result despite having no process
  exit: JSON or Markdown success is `isError: false`; incomplete zero-row
  failure is `isError: true` while preserving the same structured JSON or
  actionable Markdown content. Add an internal, optional
  `VariantArticlesMcpDisposition::{Success, StructuredError}` signal to
  `CommandOutcome` and copy it through `CliOutput`; only the `variant articles`
  handlers set it, and the raw MCP adapter selects `CallToolResult::error` only
  for `StructuredError`. Do not reinterpret arbitrary nonzero CLI exit codes or
  alter any other raw MCP command. Raw MCP continues to reject server-local
  `--input`.
- Typed `variant_articles` returns the same batch JSON text with
  `isError: false` when at least one item is usable and `isError: true` when all
  items are errors. It does not replace a structured failed batch with a plain
  `Error:` string. Schema/argument failures remain MCP `invalid_params` before
  the deadline starts.

## Public compatibility and completion evidence

The public change is additive: retain existing response, pagination,
`source_status`, item-error, debug-plan, provenance, identity, and ranking
fields and meanings. Add only the single-response `error`, `timed_out` enum
member, status `work`/`reason_codes`, and debug deadline metadata. The internal
raw-MCP disposition is not serialized. Update every checked JSON Schema/OpenAPI
or generated contract enum in the same change; document that consumers must
tolerate new `source_status.status` enum members and additive object fields.
Do not rename/remove an existing value, route, field, or debug-plan key. Both
single and batch `--debug-plan` expose `deadline.scope: "invocation"`,
`deadline.limit_ms: 60000`, `deadline.exhausted`, and
`deadline.provider_concurrency_limit: 10`; batch item plans reference that same
invocation deadline rather than presenting per-item budgets. Existing
`calls`, `pages`, `latency_ms`, logical budgets, `stopped_routes`, and candidate
trace remain, with `calls` counting admitted provider exchanges and `pages`
counting only fully decoded, committed pages. Keep the existing debug
`provider_queries.route: "discovery"` descriptor while reporting execution
status under `best_effort_free_text`, as today.

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
  rows, settled offset beyond a nonzero total, mixed batch, and all-failed batch
  across CLI JSON, CLI Markdown, raw MCP JSON/Markdown, and typed MCP, including
  stdout/stderr, exit code, `isError`, `complete`, `truncated`, every
  `has_more` rule, item errors, totals, and source-status aggregation. A raw MCP
  regression proves an unrelated nonzero `CommandOutcome` retains its current
  behavior and is not reclassified by the variant-only disposition.
- Planning fixtures cover every skeleton row, both canonical query shapes,
  resolved/unresolved/contradictory identities, all three strategies,
  zero-work `ok`, zero-work `skipped`, prerequisite-blocked units, PubMed's
  two-exchange leg, continuation discovery, Semantic Scholar chunks,
  article-base fallback ordering, PubTator verification, and both LDH stages.
  They assert exact `planned` counts and the aggregate precedence for every
  mixed terminal-state combination.
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
