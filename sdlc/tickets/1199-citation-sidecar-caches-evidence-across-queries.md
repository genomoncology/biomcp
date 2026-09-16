---
flow: build
priority: 2
deps: []
---

# 1199: Citation evidence persists across queries in a sidecar cache

## Goal

Repeated `biomcp article citation-evidence <citing-id> <cited-id>` calls for
the same directed edge are served from a local record under the existing
`BIOMCP_CACHE_DIR` tree: the second call makes no graph, full-text, or bridge
request and returns the assembled result byte-identically.

## Current Facts

The command re-runs its whole pipeline on every call: seed resolution (two
Semantic Scholar batch requests), the directed reference traversal, and — when
provider contexts are empty — the Europe PMC JATS fetch plus the blocking
parse. The HTTP cache already deduplicates network traffic for warm responses,
but the assembled outcome (including the parse work) is recomputed, the
effective freshness is limited by each HTTP entry's own policy, and HTTP
entries are subject to size-LRU eviction and `biomcp cache clean` without any
durable record of which edges were already recovered. The original observation
is `sdlc/issues/2026-08-27-a-citation-sidecar-would-make-synthesis-mechanical.md`
(that issue's `--cite` rendering idea is a different feature and is not this
ticket).

`ArticleCitationEvidenceResult` already derives `Serialize` and `Deserialize`
(`src/entities/article/graph/citation_evidence.rs:109`), so the frozen result
shape round-trips without a new wire format. Ticket 1145's deadlines and
budgets stay authoritative: the cache never extends what a call may fetch; it
only skips fetching.

## Design

### Store

New module `src/cache/citation_evidence.rs`, owned by the cache layer and
following `src/cache/provider_capture.rs` conventions: private managed tree,
atomic publish, explicit schema version, best-effort failure handling.

- Layout: `<cache root>/citation-evidence/v1/<sha256-hex>.json`, where the
  hash input is the exact key string described below. The cache root comes
  from the existing `resolve_cache_config()`.
- Key string: `"v1|{citing_paper_id}|{cited_paper_id}"` with both IDs the
  lowercase 40-hex resolved Semantic Scholar paper IDs. The cache step runs
  only after both `valid_paper_id` checks pass, so the key space contains only
  validated IDs.
- File body (compact JSON, one line):
  `{"schema_version":1,"citing_paper_id":"...","cited_paper_id":"...","stored_at_unix_ms":N,"expires_at_unix_ms":N,"result":{...}}`
  where `result` is the `ArticleCitationEvidenceResult` verbatim.
- TTL: 30 days, fixed (`CITATION_EVIDENCE_CACHE_TTL_MS = 2_592_000_000`).
  A debug-only seam `BIOMCP_TEST_CITATION_CACHE_TTL_MS` (parsed exactly like
  the citation deadline seams, `cfg(debug_assertions)` only) overrides it for
  tests.
- Modes, resolved from the existing environment exactly as the HTTP cache
  resolves them: `crate::sources::cache_is_bypassed()` (the `--no-cache` /
  `BIOMCP_CACHE_MODE=off` path) bypasses both read and write;
  `BIOMCP_CACHE_MODE=infinite` reads entries regardless of expiry;
  default reads only unexpired entries. No new environment variable and no
  new flag.

### Read-before-fetch placement

The read happens after seed resolution and after both `valid_paper_id` checks,
and before `directed_edge_contexts`. A hit returns the stored `result`
verbatim; the stored `citing`/`cited` are the resolved related-paper objects,
so the answer is spelling-independent and a hit for any caller spelling of the
same resolved pair is correct.

A hit is treated as a miss when the file is absent, unreadable, not valid
JSON, carries a different `schema_version`, carries a key mismatch, or is
expired (unless infinite mode). A `--fulltext` call serves a stored entry
exactly when its status is not `context_from_provider`: the forced attempt's
point is the JATS path, and every other cacheable status reflects a completed
JATS attempt. With today's five-state enum that reads
`context_from_fulltext`; the six-state extension (ticket 1200) fits the same
rule without amendment because its status is also a completed-JATS-attempt
evidence state. A stored `context_from_provider` result is a miss for a forced
call.

The two-level alternative — a spelling-keyed breadcrumb mapping caller inputs
to the resolved pair, so a repeat call could skip seed resolution — was
rejected: a stale breadcrumb could serve one paper's evidence for a spelling
that now resolves elsewhere, which is a false evidence claim. Read after
resolution keeps every hit paired with a live resolution.

### Write

After outcome assembly, exactly when the final status is one of
`context_from_provider`, `context_from_fulltext`, or
`reference_confirmed_without_passage`. The non-evidence statuses
(`fulltext_unavailable`, `reference_unresolved`, `citation_marker_unlinked`)
are never cached: they usually mean transient provider unavailability or a
document version detail, and pinning them for the TTL would present a stale
failure as current. The write is atomic (temporary file in the same directory
plus rename), best-effort, and never changes the command outcome; a failure is
a `tracing::warn`. No file locks are used: one file per key with atomic rename
makes the last writer win and readers never observe a partial file.

### Clear

`execute_managed_clear` (`src/cli/cache.rs:360`) additionally removes
`<cache root>/citation-evidence`, summing bytes and entries into the existing
`ClearReport` exactly as the `sessions` directory is summed today.
`biomcp cache path` and `biomcp cache stats` are unchanged.

## Fixtures

Focused tests, all under `src/entities/article/graph/tests.rs` or a new
`src/entities/article/graph/tests/cache.rs` using the existing
`spawn_citation_fixture` loopback pattern and a fresh `TempDirGuard` cache per
test:

1. Repeat call: call once (populates); delete the `<cache>/http` directory;
   call again → output equals the first call byte-for-byte and the fixture
   request log contains only the two `s2:seed` lines (no `s2:graph` and no
   `fullTextXML`).
2. Cross-spelling: first call with the PMID spelling, second with the DOI
   spelling of the same resolved pair → second log has only `s2:seed` lines.
3. TTL seam: with `BIOMCP_TEST_CITATION_CACHE_TTL_MS=0`, the second call
   refetches (log shows `s2:graph`).
4. Bypass: with `BIOMCP_CACHE_MODE=off`, the second call refetches and no
   file is created under the sidecar directory.
5. Infinite: entry written with the TTL seam at 0, second call with
   `BIOMCP_CACHE_MODE=infinite` is served from the cache.
6. Corruption: a garbage file at the key path is a miss, and the call
   refetches and replaces it.
7. Future schema: a file with `"schema_version":2` is a miss.
8. Non-evidence statuses are not cached: a `fulltext_unavailable` outcome
   leaves no file.
9. Forced miss: a cached `context_from_provider` result plus `--fulltext`
   performs the JATS attempt (log shows `fullTextXML`).
10. Forced hit: a cached `context_from_fulltext` result plus `--fulltext` is
    served with only `s2:seed` lines in the log.
11. Write failure: a regular file at the sidecar directory path makes the
    write fail and the outcome is unchanged.
12. Clear: `execute_managed_clear` removes the sidecar directory (extend the
    existing clear tests to assert the directory is gone and the report sums
    include its bytes).

Executable spec (`spec/entity/article.md`, the existing article fixture
family): one block runs the same `--json` citation-evidence command twice and
asserts byte-identical output, proving the cache is invisible in the folded
surface. The request-count proofs live in the focused tier because a warm HTTP
cache already suppresses fixture traffic on the second call.

## Acceptance

All twelve focused tests pass under the existing `make test` lane; the spec
block passes; `make lint`, `make test`, and `make spec` pass on the gate host
at the pushed SHA. The package path count gains exactly the one new module
(following the established authorized-baseline pattern if any source-size
baseline moves, with the measured delta and a removal condition recorded). No
existing test changes behavior.

## Dependencies

None.

## Boundaries

Only `article citation-evidence` uses the sidecar; no other command gains a
cache level. No new CLI flag, no new environment variable (the debug TTL seam
is test-only and is classified in the environment-docs contract like the
citation deadline seams), and no cache-provenance member in the output: the
cache is invisible, exactly like the HTTP cache. No `--cite` sidecar output
(the 2026-08-27 issue's rendering idea is a separate feature). No corpus
listing, export, or query surface over stored entries. No backfill or
migration of any kind. No change to ticket 1145's request budgets, outcome
states, or JSON shapes.

## Complexity

- Contract score: 1 (several explicit cases: hit, miss, expiry, bypass,
  infinite, forced semantics, clear)
- State and timing score: 2 (persistent state with TTL, invalidation, and
  atomic publication)
- Reach score: 1 (one command path plus the cache layer and its clear path)
- Proof score: 2 (failure injection for corruption and write failure, exact
  request-log proofs, byte-identity)
- Cost of error score: 1 (stale or wrong evidence served as current is
  user-visible; recoverable by clear/TTL, no data loss)
- Total: 7
- Minimum level floor: level 3 (cache invalidation and shared durable state)
- Final level: 3
- Reasons: the cache-invalidation floor sets the level; the surface is one
  command but the state and failure modes are real
- Selected model: gpt-5.6-sol, medium reasoning (level 3 implementer)

## Review

- Design review: pending
- Code review: pending
