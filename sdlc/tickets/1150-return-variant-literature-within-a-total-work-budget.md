---
flow: build
priority: 8
deps: [1167]
---

# Return variant literature within one invocation-wide work deadline

## Goal

One `variant articles` invocation has one monotonic 60-second provider-work
deadline and returns every usable row completed before that deadline. The
deadline applies to a single CLI query and to the complete 1-10 item batch used
by `--input` and the typed `variant_articles` MCP tool; it is not reset per
item, route, provider, retry, or phase.

The original evidence predates dependency 1167. On 2026-09-04,
`biomcp --json variant articles "ODC1 c.1342A>T" --limit 10` produced no JSON
before both 55-second and 130-second process limits expired, while the isolated
annotation strategy completed in 21.26 seconds. The reproduction and code
evidence are in
`sdlc/issues/2026-09-04-variant-article-union-can-exceed-two-minutes.md` at
commit `f8ff2a78`.

## Deadline contract

BioMCP validates local command options, input JSON, the 1-10 item bound,
request IDs, and item identity syntax before starting the clock. Immediately
after that validation, and before variant resolution or construction of any
provider client, it creates one deadline from a monotonic clock. Production
uses 60 seconds; tests can inject a shorter duration and controlled clock.
Invalid invocation syntax still fails before provider contact and is not
reclassified as a timeout.

The shared deadline covers every subsequent operation that can wait on a
provider or provider substrate: variant resolution and canonical equivalence,
client and cache-manager construction, discovery and citation routes, all
pages and aliases, transport retries and backoff, provider rate-limit waits,
visible-row enrichment, PubTator identity verification, and ClinGen LDH
verification. Existing per-request timeouts are ceilings inside this deadline;
each attempt, retry, and wait receives at most the remaining invocation time.
No provider work is launched once no time remains.

At expiry, outstanding provider work is cancelled and settled without waiting
for its original provider timeout. Only bounded in-memory merge, deduplication,
ranking, pagination, status construction, serialization, and rendering may run
after the deadline. No retry, rate-limit sleep, enrichment, verification,
cache write, or other provider work may continue detached after the response is
returned.

Concurrency is bounded and deterministic. The existing maximum of two active
variant items remains. Provider futures use a fixed request-scoped in-flight
cap, declared as a constant and reported by `--debug-plan`; the cap must not
grow with the number of items, aliases, or providers. Eligible work is admitted
in input-item order and then stable route/provider/query-plan order. Completion
order cannot change output item order, source-status order, candidate ordering,
deduplication, or ranking. Independent work may run concurrently only through
that scheduler and still passes through the existing provider rate limiters.

## Terminal states

`source_status` remains the caller-visible route/source ledger and adds
`timed_out` to its closed status vocabulary. One aggregate terminal status is
reported for each planned route/source, in stable route/source order:

- `ok`: all admitted work for the route/source settled before the deadline.
- `degraded`: it settled before the deadline with usable data, but a
  non-deadline provider failure made that data incomplete.
- `unavailable`: it was attempted and settled before the deadline with no
  usable data because of a non-deadline provider or construction failure.
- `timed_out`: some work for that route/source started but did not settle by the
  invocation deadline. Rows it had already completed remain usable.
- `not_attempted`: eligible work never started because the invocation deadline
  or an existing logical work cap was exhausted. Its bounded detail identifies
  which budget stopped it.
- `skipped`: the route was deliberately inapplicable because of the selected
  strategy, identity result, or existing configuration; it is not a timeout.

For an aggregate that completed rows before later work timed out, `timed_out`
takes precedence over `degraded`; for a non-timeout partial provider result,
`degraded` remains correct. A provider timeout must not be collapsed into
`unavailable`, and work that never started must not be called `timed_out`.
Deadline exhaustion makes the affected item and batch `complete: false` and
`truncated: true`, leaves `pagination.total` unknown, and names the unfinished
route/source entries. Existing non-deadline incomplete states retain their
current meanings.

## Partial and failure semantics

A usable result is an article row that survives the existing identity checks,
deduplication, filtering, and requested pagination. The same rule applies at
every surface:

- If at least one usable row exists in the invocation, completed rows are
  returned, partial metadata is preserved, and the CLI exits zero. JSON is
  written to stdout. Markdown includes the rows plus an explicit incomplete
  coverage warning listing each `timed_out`, `not_attempted`, `unavailable`, or
  `degraded` route/source instead of the current generic warning.
- In a batch, an item with rows has `error: null` even when incomplete. An item
  with no row because its started work expired receives a bounded actionable
  `deadline_exceeded` item error; an item never admitted before expiry receives
  the same code and `not_attempted` status; a non-deadline provider failure
  retains `source_unavailable`. Other existing per-item validation errors are
  unchanged. Batch item order remains input order.
- If no usable row exists anywhere in the invocation, JSON still returns the
  structured response and statuses on stdout but the CLI exits one. Markdown
  exits one with a bounded actionable explanation that distinguishes deadline
  exhaustion from provider unavailability and names the affected routes.
- The typed MCP tool returns an ordinary successful tool result with the same
  structured partial response when at least one row exists. When no usable row
  exists, it returns the structured response as an MCP tool error rather than a
  transport/protocol error, so callers retain item errors and source statuses.
  Invalid tool parameters remain MCP `invalid_params` before the clock starts.

A healthy completed search with no matching article therefore has no usable
row and follows the no-result failure rule with an actionable no-results
message; it is not mislabeled as unavailable or timed out.

## Completion evidence

- After 1167 lands, and before implementation begins, rebuild from its landing
  commit and rerun the exact ODC1 union reproduction against both a fresh cache
  and a representative accumulated cache. Record route timings and the first
  JSON emission. The pre-1167 timing is not sufficient confirmation. If the
  post-1167 implementation no longer threatens the 60-second contract, return
  this ticket to design review instead of implementing a speculative deadline.
- Deterministic tests use injected short deadlines and controlled provider
  futures or paused Tokio time, never public services or production-length
  sleeps. A fast route returns a row while a started slow route becomes
  `timed_out`, a queued route becomes `not_attempted`, and a separately failed
  route remains `unavailable`; the row survives with incomplete metadata.
- A 10-item fixture proves there is one shared deadline rather than ten item
  deadlines, at most two items are active, the provider in-flight cap is never
  exceeded, no work starts after expiry, and items and statuses remain stable
  when completion order is varied.
- Separate short-deadline fixtures expire during resolution/client
  construction, retry/backoff or a rate-limit wait, enrichment, and identity
  verification. Each settles promptly, records the owning route/source
  truthfully, and performs no post-response provider work.
- CLI Markdown, CLI JSON, mixed and wholly failed batch JSON, and typed MCP
  tests pin the success/error behavior above, including `complete`,
  `truncated`, unknown totals, exit status or MCP `isError`, item errors, and
  the four distinct `timed_out`, `not_attempted`, `unavailable`, and
  `degraded` outcomes.
- Existing annotation and lexical strategy tests remain independently usable,
  and the existing identity, confirmed-only, logical budget, provider-rate,
  pagination, candidate-order, and ranking regressions keep passing.

## Cache construction caveat

Ticket 1167 removes recursive whole-cache repair after every write, but
deliberately retains recursive repair and size initialization during cold HTTP
client construction. Those operations are synchronous today. Wrapping only
the later async provider future in `tokio::time::timeout` cannot enforce this
ticket: cold construction must participate in the deadline through a safe,
deadline-aware seam. The implementation must neither skip required cache
permission validation nor detach a blocking scan that continues after the
caller has received a timeout result. If that cannot be achieved without a
broader cache-lifecycle change, stop and revise the ticket rather than claiming
the 60-second guarantee.

## Boundaries and dependencies

Ticket 1167 must land and the premise must be reconfirmed before implementation.
Ticket 1164 remains downstream of this ticket.

This deadline is an additional ceiling, not a replacement for existing logical
caps: the 10 exact-alias cap, item/request call budgets, fetch and result caps,
identity-verification reservations, and LDH limits remain. Provider retry
counts and rate limits are not weakened or bypassed. Variant identity,
canonical-equivalence, `--verify-identity`, `--confirmed-only`, deduplication,
ranking inputs and order, relevance rules, and explicit annotation/lexical
strategy purposes do not change. Cache security, layout, eviction, and
maintenance behavior remain owned by the cache tickets except for making cold
construction honor this invocation deadline.
