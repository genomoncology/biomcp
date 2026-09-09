---
flow: build
priority: 8
deps: []
---

# Give article batching one canonical command and contract

## Goal

Make `batch article` the one grammar BioMCP teaches for multi-article lookup,
while preserving the working `article batch` compact route byte-for-byte for
existing callers. The canonical command makes compact shortlist cards versus
ordinary article detail an explicit mode instead of hiding the distinction in
word order.

## Current facts

Reconfirmed on current `main` at
`2b3129f0e52fcc56f5b92d571be6b7c1956344c1` after BioMCP regained local
ownership of its 0.9 trial code:

- `src/cli/system/mod.rs::BatchArgs` parses
  `biomcp batch <entity> <one-comma-separated-argument> [--sections ...]`.
  `src/cli/system/dispatch.rs::handle_batch` caps every entity at ten and the
  article arm runs the ordinary `get article` path. Successful JSON items are
  the ordinary flattened `Article` projection plus `_meta.evidence_urls`,
  `_meta.next_commands`, `_meta.section_sources`, and the other ordinary
  entity metadata.
- `src/cli/article/mod.rs::ArticleCommand::Batch` separately parses
  `biomcp article batch <one-or-more-space-separated-IDs>`. Its dispatcher caps
  input at `ARTICLE_BATCH_MAX_IDS == 20`, calls `get_compact` once per input,
  and settles `ArticleBatchItem` values.
- Both routes use `src/cli/system/batch.rs::settle_batch`. JSON is an object
  containing `summary` and ordered `items`; it is not a bare array. Markdown
  has the shared `# Batch: article (N)` wrapper and summary. The compact item
  renderer nests one `# Article Batch (1)` card inside each successful item.
  `docs/user-guide/article.md` and `src/cli/list_reference.md` currently claim
  a bare compact array and must be corrected rather than treated as the wire
  contract. `spec/entity/article.md` exercises the executable envelope.
- `get_compact` derives each card from the ordinary PubTator/Europe PMC base
  article, then attempts optional Semantic Scholar enrichment. Semantic
  Scholar failure is fail-open: it emits the existing provider warning and
  leaves optional TLDR/citation fields absent without changing an otherwise
  successful item into an error.
- Ordinary article detail carries the typed `section_outcomes` map. Semantic
  Scholar is represented by the `tldr` outcome (`data`, `empty`, or
  `unavailable`) and corresponding `_meta.section_sources`; detail must keep
  that contract rather than adopting compact omission semantics.
- `settle_batch` currently starts every item future with `join_all`, preserves
  input order and duplicates, returns exit 1 when any item fails, and returns
  the complete mixed-success report on stdout. It adds no batch retry.
  Provider middleware owns the existing request timeouts, at-most-three retry
  policy, retry sleep, cache, and rate limiting.
- Raw MCP allows both command families and projects the CLI result through the
  `biomcp` escape hatch. The seven-tool typed MCP catalog has no general or
  article batch tool. Article-search session suggestions, CLI/list embedded
  reference strings, packaged skills, use cases, public docs, and executable
  specs predominantly teach `article batch`, while generic batch help teaches
  `batch article`.

## Settled public grammar

The canonical grammar is exactly:

```text
biomcp batch article <id1,id2,...> [--mode compact|detail] [--sections <s1,s2,...>]
```

`--mode` defaults to `detail`. It is accepted only when the normalized entity
is exactly `article`; supplying either mode for gene, variant, trial, drug,
disease, pgx, pathway, protein, or adverse-event is an invalid-argument error
before any provider/client/cache construction. Other entities retain their
current grammar and behavior.

`--sections` is accepted for article batches only in `detail` mode. It uses the
existing comma-separated article section vocabulary and ordinary article
section semantics. Supplying `--sections` with `--mode compact`, including an
empty value, is rejected before provider work. Compact mode does not silently
ignore sections. Canonical detail validates every normalized section against
that closed vocabulary during batch preflight; an unknown or malformed section
is an exit-2 invalid argument with zero item polling, provider/client/cache
construction, or filesystem work. Existing section behavior for non-article
batch entities is unchanged.

The compatibility grammar remains exactly:

```text
biomcp article batch <id1> <id2> ...
```

It is compact mode and gains no `--mode`, `--sections`, pagination, or comma
list interpretation. Except for its help text and the intentional rejection of
an ID longer than 512 UTF-8 bytes, every previously valid call must preserve
stdout bytes, deterministic diagnostic bytes, JSON formatting, input echo, and
exit status exactly. Timestamped tracing output is not a byte-stable interface;
its provider identity, sanitized warning text, count, and ordering relative to
other warnings remain stable. Execution prints no deprecation or migration warning. Only
`biomcp article batch --help` identifies the route as compatibility syntax and
shows the copyable replacement
`biomcp batch article <id1,id2,...> --mode compact`.

Top-level help, `batch --help`, `list batch`, `list article`, generated next
commands, and current workflow guidance teach canonical syntax. Durable
reference material may document the compatibility route in one migration note
but must not recommend or generate it.

## Settlement and rendering contract

Both canonical modes use the existing settlement envelope. There is no second
batch response type.

```json
{
  "summary": {"total": 2, "succeeded": 1, "failed": 1},
  "items": [
    {"input": "first", "status": "ok", "result": {}},
    {"input": "second", "status": "error", "error": {}}
  ]
}
```

The real objects are not the empty examples above:

- Compact success `result` is exactly the current serialized
  `ArticleBatchItem`: `requested_id`, resolved identities, title, complete
  authorship fields, optional journal/year/entity summary, and optional
  Semantic Scholar TLDR/citation fields. It does not gain ordinary article
  `_meta` or `section_outcomes`.
- Detail success `result` is exactly the JSON projection produced today by
  the `batch article` arm: the ordinary serialized `Article` flattened beside
  its ordinary `_meta`, including evidence URLs, next commands, and section
  provenance. Requested section outcomes retain their typed states. It does
  not collapse to `ArticleBatchItem`.
- Error items retain the current `input`, `status: "error"`, and the public
  structured `error` projection. `summary.total` counts inputs, including
  duplicates; succeeded and failed partition total.

JSON remains pretty-printed by the current renderer. Items occur once in
request order, including duplicate IDs and mixtures of success and error.
There is no batch-level `_meta`, mode field, pagination token, offset, or
truncation marker.

Markdown is byte-defined by the existing settlement composition:

```text
# Batch: article (N)\n
[for each item, in input order]
\n---\n\n
## <input> — ok\n\n<mode renderer bytes>
or
## <input> — error\n\n<public error message>\n
[after all items]
\n## Summary\n\nTotal: N; succeeded: S; failed: F.\n
```

For compact mode, `<mode renderer bytes>` is the unchanged one-element
`article_batch_markdown` result beginning `# Article Batch (1)`. For detail it
is the unchanged `article_markdown` result for the requested sections,
including ordinary source-state, related-command, and evidence blocks. The
outer renderer must not strip, merge, or renumber inner headings. Existing
human-output sanitization remains the final boundary for CLI and raw MCP.

Validation errors exit 2 through the existing CLI error path. A completely
successful settlement exits 0. A mixed or all-error settlement prints the
complete report on stdout, leaves settlement errors off stderr, and exits 1.
Optional compact Semantic Scholar warnings retain their existing stderr/log
behavior and do not affect item status or exit status. Raw MCP retains its
current mapping: a settled report is successful tool content even when the CLI
outcome carries exit 1; parse/preflight rejection is an MCP tool error.

## Bounds, ordering, and work ownership

All preflight validation completes before constructing a provider client,
opening the HTTP cache, starting a retry sleep, or polling an item future:

- canonical compact accepts 1–20 comma-separated IDs;
- canonical detail accepts 1–10 comma-separated IDs;
- compatibility compact accepts 1–20 space-separated IDs as it does today;
- every canonical ID retained after comma parsing is nonempty and at most 512
  UTF-8 bytes; one overlong canonical ID rejects the whole command with exit 2
  and zero provider requests;
- compatibility preserves every positional value exactly as today, including
  an explicitly supplied empty value; its only new per-ID validation is the
  512-byte limit, applied without trimming or rewriting the echoed value; and
- unsupported `--mode`, `--sections`, `--source`, pagination-like flags, and
  over-limit input are rejected before providers.

Canonical comma parsing preserves the current `batch` behavior: trim each
component, ignore empty components, then enforce the post-parse lower and upper
count and the 512-byte limit. Compatibility passes each positional value
through exactly as today; the new all-ID byte-length preflight is its only
added input safety check. Do not make canonical normalization retroactively
reject, trim, drop, or rewrite a compatibility value.

The compatibility length guard is an explicit, narrow safety break: a
513-byte positional value that previously reached settlement and produced an
exit-1 item error now fails preflight with exit 2 and zero work. Baseline and
post-change goldens must pin both sides of that intentional change without
generalizing it to shorter, empty, whitespace-only, or comma-containing
positional values.

Settlement owns at most ten live item futures at once in either mode. Pending
items enter in request order; completion order never changes output order.
Duplicates are neither removed nor coalesced: each position creates and
settles its own logical get, although the existing HTTP cache may naturally
serve a repeated request. The command waits for all items after ordinary item
failure; it does not fail fast.

The batch layer adds no retry, deadline, detached task, or background work.
Each item retains the exact provider-owned timeout/retry/rate-limit policy of
the underlying compact or ordinary get. An item occupies its concurrency slot
through retry sleeps. Dropping the CLI/MCP batch future drops queued and active
item futures, including a pending provider retry sleep; no spawned item may
continue provider work after cancellation. Deterministic controlled-future
tests prove the ten-item ceiling, queue order, settlement after failure, and
cancellation while work and retry sleep are pending.

This ticket explicitly does not optimize provider traffic. Do not introduce a
PubTator, Europe PMC, Semantic Scholar, or cross-provider bulk request; do not
deduplicate IDs; and do not change cache keys, provider selection, retry
counts, or Semantic Scholar authentication/rate limiting. Such optimization
needs its own measurements and contract.

## Surfaces and executable proof

Use provider-faithful local fixtures; no acceptance test contacts a public
provider. Before changing dispatch, capture exact compatibility golden outputs
from the baseline executable. Afterward prove:

1. CLI parsing and zero-request preflight matrices cover default detail,
   explicit detail/compact, invalid modes, mode on every non-article entity,
   sections in both modes, 0/1/10/11/20/21 IDs, a 512-byte ID, a 513-byte ID,
   commas/whitespace/empty components, unsupported pagination flags, and
   `--source`. Limit and length failures must expose request count zero.
2. Canonical compact and compatibility compact have identical full stdout,
   deterministic diagnostic bytes, JSON bytes, and exit codes for ordered
   success, duplicates, mixed failure, all failure, and optional Semantic
   Scholar success. For fail-open Semantic Scholar failure, compare exact
   stdout/JSON/exit behavior and assert the same sanitized provider warning,
   warning count, and relative order while ignoring formatter timestamps. The
   committed baseline goldens make the compatibility claim independent of
   shared implementation.
3. Canonical detail default and explicit-detail JSON contain the ordinary
   article projection and `_meta`; Markdown contains the exact wrapper and
   unmodified ordinary article rendering. Section fixtures prove Semantic
   Scholar `tldr` data, empty, and unavailable outcomes and matching section
   provenance. Compact fixtures prove those failures only omit optional
   enrichment and retain item success.
4. Raw stdio and Streamable HTTP MCP execute canonical compact, canonical
   detail, and compatibility compact in Markdown and JSON. Assertions cover
   exact text/structured content, mixed-failure tool-success behavior,
   preflight tool errors, ordering, and redaction/sanitization. `tools/list`
   remains the same seven tools; typed `search` and `get` schemas gain no
   batch, IDs, or mode field, and no typed article-batch tool is added.
5. The article-search session loop-breaker, next-command validation, command
   help, `src/cli/list_reference.md`, `list batch`, and `list article` emit or
   teach `biomcp batch article <comma IDs> --mode compact`. No executable
   next command emits `article batch`.
6. Inventory and update packaged `skills/SKILL.md`, every article-batch use
   case/ladder, public article/CLI/how-to/reference pages, provider attribution,
   embedded Rust help/reference strings, Python docs/skill contracts, and
   `spec/entity/article.md`, `spec/surface/skills.md`, and the variant/article
   fixture assertion. A repository-wide current-content assertion allows the
   old spelling only in the compatibility help/migration paragraph, its exact
   preservation tests, and append-only records or historical blog prose.
7. Existing non-article batch CLI, JSON, Markdown, and raw-MCP contracts remain
   green. Package inventory/quality ratchets remain within their existing
   limits; implementation must split modules instead of weakening ratchets.

The preflight matrix separately freezes canonical `a,,b`, empty-only,
whitespace-only, 512-byte, and 513-byte components, plus compatibility empty,
whitespace-only, literal-comma, 512-byte, and 513-byte positional arguments.
For compatibility, compare baseline and new stdout/stderr/JSON/input/exit bytes
for every case up to 512 bytes and separately prove the intentional 513-byte
exit-1-to-exit-2 change with zero provider/cache work. For detail mode, include
unknown, empty, duplicated, and mixed-validity section lists and prove invalid
lists poll no item future.

Cancellation proof must use the actual injected provider middleware
retry/backoff future rather than an unrelated sleeping future: start a batch
whose item has entered retry backoff, drop the CLI or raw-MCP batch future,
advance paused time past the retry interval, and assert no later provider
request, cache operation, task, or permit remains.

On the current base, `src/cli/article/dispatch.rs` is 696 lines and the
preserved WIP would raise it to the 700-line CLI ceiling. Finish this ticket at
or below 696 lines by consolidating the added routing at its existing owner or
moving shared preflight/settlement logic into the existing system-batch owner;
do not consume the four-line headroom needed by the dependent article work.
The source package remains exactly 1,300 paths, and ticket 1183's external-
project coupling ratchet remains unchanged and green.

Run focused Rust CLI/entity/renderer/MCP tests, the affected Python
docs/skills/MCP contracts, and the affected article and skills mustmatch specs,
then the repository's standard `make lint`, `make test`, and `make spec` gates.

## Dependencies and independence

This ticket has no blocking dependency. Ticket 1146 changes recovery from
reversed search grammar and can land before or after this work; its correction
can use whatever canonical article-batch help exists at integration time.
Draft 1163 changes reserved-keyword diagnostic prose and is likewise
independent. Neither ticket changes the two article batch response contracts,
so neither is a prerequisite for 1147 and 1147 is not a prerequisite for them.

## Boundary

This ticket consolidates and documents article batch routing. It does not
remove the compatibility parser, change ordinary article detail or compact
`ArticleBatchItem`, add a typed MCP tool, add pagination, optimize or batch
provider calls, redesign every entity's batch interface, rename citation or
reference pivots, change provider retry/cache/rate-limit policy, or change
historical records.

## Review

The 2026-09-09 current-main refresh resolves the prior contradiction between a
global nonempty-ID statement and the stronger compatibility promise. Canonical
comma parsing ignores empty components; compatibility retains its existing
positional semantics through 512 bytes and gains one explicitly documented
overlength safety break. It also narrows timestamped warning equivalence,
requires route-specific normalization and length goldens, moves detail-section
validation into preflight, strengthens cancellation proof through real retry
middleware, and freezes the current dispatch and package ceilings. This
refresh requires fresh independent design acceptance before implementation
resumes.

Implementation starts only after fresh design acceptance. Code review must
compare compatibility bytes to the pre-change goldens, inspect preflight
request-count proofs and cancellation ownership, and verify the complete
surface inventory rather than accepting contains-only smoke assertions.
