---
flow: build
priority: 8
deps: []
---

# Give article batching one canonical command and contract

## Goal

Make `batch article` the one grammar BioMCP teaches for multi-article lookup,
while preserving the working `article batch` compact route for existing
callers. The canonical command makes compact shortlist cards versus ordinary
article detail an explicit mode instead of hiding the distinction in word
order. At the same boundary, replace the route's eager `join_all` fan-out with
at most ten live item futures, reject compatibility raw positional values over
512 bytes, and reject canonical trimmed identifiers over 512 bytes. These are
intentional public safety changes, not incidental implementation details.

## Current facts

Reconfirmed at current main `7c45f6298cdc970e1aa9b06ea899ac4707701029`:

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
  article, then attempts optional Semantic Scholar enrichment. After the
  Semantic Scholar client is constructed successfully, enrichment
  request/response failure is fail-open: it emits the existing provider
  warning and leaves optional TLDR/citation fields absent without changing an
  otherwise successful item into an error. Client construction or
  configuration failure is instead a settled item error and remains so.
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
ignore sections. Existing section behavior for non-article batch entities is
unchanged.

The compatibility grammar remains exactly:

```text
biomcp article batch <id1> <id2> ...
```

It is compact mode and gains no `--mode`, `--sections`, pagination, or comma
list interpretation. For every positional value of 512 raw UTF-8 bytes or
fewer, including empty, whitespace-only, and literal-comma values, every
previously valid call must preserve stdout bytes, JSON formatting, input echo,
exit status, and stderr behavior exactly. Deterministic stderr comparisons run
with tracing disabled; timestamped provider warnings are proved separately by
normalized event fields and count. A raw positional value over 512 bytes is
the one intentional compatibility break: reject the whole command with the
existing invalid-argument envelope and exit 2 before provider/client/cache
construction. Execution prints no deprecation or migration warning. Only
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
Optional compact Semantic Scholar post-construction enrichment warnings retain
their existing stderr/log behavior and do not affect item status or exit
status. Client construction/configuration failure remains an item error. Raw
MCP retains its current mapping: a settled report is successful tool content
even when the CLI outcome carries exit 1; parse/preflight rejection is an MCP
tool error.

## Bounds, ordering, and work ownership

All preflight validation completes before constructing a provider client,
opening the HTTP cache, starting a retry sleep, or polling an item future:

- canonical compact accepts 1–20 surviving comma-separated IDs;
- canonical detail accepts 1–10 surviving comma-separated IDs;
- canonical parsing trims each component and ignores empty components; every
  surviving value must be nonempty and at most 512 UTF-8 bytes after trimming,
  and an empty surviving list or one overlength value rejects the whole command
  with exit 2 and zero provider work;
- compatibility compact accepts 1–20 raw space-separated positional values as
  it does today, including empty, whitespace-only, and literal-comma values;
  measure each untouched positional value before any trimming and reject only
  a value over 512 UTF-8 bytes with exit 2 and zero provider work; and
- unsupported `--mode`, `--sections`, pagination-like flags, and `--source` on
  an article batch are rejected before providers. Existing valid and invalid
  `batch trial ... --source ...` behavior is unchanged.

Canonical comma parsing preserves the current `batch` behavior for valid
lists: trim each component and ignore empty components, then enforce the
post-parse lower and upper count and trimmed byte limit. Compatibility passes
each positional value through exactly as today; the raw-byte length preflight
is its only added input safety check and must not trim or rewrite the echoed
value.

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
scheduler tests prove the ten-item ceiling, queue order, settlement after
failure, and the dropping of queued and active futures. A separate integration
test through the real injected provider middleware proves that dropping a CLI
or raw-MCP batch while an item is in retry backoff prevents the later retry
request and leaves no cache operation, task, or permit alive.

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
   `--source`. Article `--source`, limit, and length failures must expose
   request count zero; the matrix also freezes unchanged valid and invalid
   trial-source behavior. Canonical cases measure trimmed surviving values;
   compatibility cases measure raw positional bytes and preserve empty,
   whitespace-only, and literal-comma inputs through 512 bytes.
2. With tracing disabled, canonical compact and compatibility compact have
   identical full stdout, stderr, JSON bytes, and exit codes for ordered
   success, duplicates, mixed failure, all failure, optional Semantic Scholar
   success, post-construction fail-open Semantic Scholar request/response
   failure, and Semantic Scholar client construction/configuration failure as
   a settled item error. A separate in-process tracing subscriber proves one
   normalized post-construction fail-open warning event with the existing
   fields and no item failure. The committed baseline goldens make
   compatibility independent of shared implementation.
3. Canonical detail default and explicit-detail JSON contain the ordinary
   article projection and `_meta`; Markdown contains the exact wrapper and
   unmodified ordinary article rendering. Section fixtures prove Semantic
   Scholar `tldr` data, empty, and unavailable outcomes and matching section
   provenance. Compact fixtures prove post-construction enrichment failures
   only omit optional fields and retain item success, while client
   construction/configuration failures retain item-error settlement.
4. Raw stdio and Streamable HTTP MCP execute canonical compact, canonical
   detail, and compatibility compact in Markdown and JSON. Assertions cover
   the exact single sanitized text content item and absence of protocol
   `structuredContent`, mixed-failure tool-success behavior, preflight tool
   errors, ordering, and redaction/sanitization. `tools/list` remains the same
   seven tools; typed `search` and `get` schemas gain no batch, IDs, or mode
   field, and no typed article-batch tool is added.
5. The article-search session loop-breaker, next-command validation, command
   help, `src/cli/list_reference.md`, `list batch`, and `list article` emit or
   teach `biomcp batch article <comma IDs> --mode compact`. No executable
   next command emits `article batch`.
6. Inventory and update packaged `skills/SKILL.md`, every article-batch use
   case/ladder, public article/CLI/how-to/reference pages, provider attribution,
   embedded Rust help/reference strings, Python docs/skill contracts, and
   `spec/entity/article.md`, `spec/surface/skills.md`, and the variant/article
   fixture assertion. A current-product assertion scans tracked content under
   `README.md`, `src/`, `skills/`, `spec/`, `tests/`, and `docs/` except
   `docs/blog/`; it deliberately excludes `sdlc/`, generated output, and other
   history. Within that domain the old spelling is allowed only in the
   compatibility help/migration paragraph and exact preservation tests.
7. Existing non-article batch CLI, JSON, Markdown, and raw-MCP contracts remain
   green. Package inventory/quality ratchets remain within their existing
   limits; implementation must split modules instead of weakening ratchets.

Run focused Rust CLI/entity/renderer/MCP tests, the affected Python
docs/skills/MCP contracts, and the affected article and skills mustmatch specs,
then the repository's standard `make lint`, `make test`, and `make spec` gates.

## Complexity

- Contract: 2 — this introduces one canonical article-batch grammar while
  freezing exact compatibility behavior for the existing grammar.
- State and timing: 2 — bounded concurrency, ordered settlement, provider
  retry backoff, and cancellation ownership are part of the contract.
- Reach: 1 — the change spans several modules and documented CLI/raw-MCP
  surfaces, but remains within BioMCP 0.9.
- Proof: 2 — acceptance requires hostile-input, failure-injection,
  cancellation, and exact-byte compatibility proofs across both output modes.
- Cost of error: 1 — a mistake would break user-visible commands or silently
  change batch results, but has no data-loss or external migration risk.
- Total: 8.
- Minimum level floor: Level 3 because concurrency and cancellation are
  explicit correctness boundaries.
- Final level: Level 3.
- Reasons: the public contract is settled, but safe implementation depends on
  coordinated parser, settlement, renderer, MCP, documentation, and fixture
  proofs.
- Selected model: `gpt-5.6-sol`, medium.

## Dependencies and independence

This ticket has no blocking dependency. Ticket 1146 changes recovery from
reversed search grammar and can land before or after this work; its correction
can use whatever canonical article-batch help exists at integration time.
Draft 1163 changes reserved-keyword diagnostic prose and is likewise
independent. Neither ticket changes the two article batch response contracts,
so neither is a prerequisite for 1147 and 1147 is not a prerequisite for them.

## Boundary

This ticket consolidates and documents article batch routing, limits article
settlement to ten live futures, and adds the stated 512-byte input boundary. It
does not remove the compatibility parser, change ordinary article detail or
compact `ArticleBatchItem`, add a typed MCP tool or protocol structured
content, add pagination, optimize or batch provider calls, redesign every
entity's batch interface, rename citation or reference pivots, change provider
retry/cache/rate-limit policy, or change historical records.

## Review

The 2026-09-11 re-review correction makes the two safety changes explicit in
the goal, preserves raw compatibility inputs through 512 bytes, isolates
timestamped warning semantics from byte goldens, freezes raw MCP as one text
item without structured content, scopes `--source` to article while preserving
trial behavior, and separates scheduler cancellation proof from provider retry
drop proof.

Implementation starts only after fresh design acceptance. Code review must
compare compatibility bytes to the pre-change goldens, inspect preflight
request-count proofs and cancellation ownership, and verify the complete
surface inventory rather than accepting contains-only smoke assertions.
