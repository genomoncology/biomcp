---
flow: build
priority: 3
---

# Article keyword and gene fields reject misplaced query syntax before search

Status: implemented; code-review remediation complete; re-review pending

## Outcome

Article search does not silently send BioMCP-style field expressions as
provider-neutral keyword text, and its gene field does not silently accept a
multi-concept phrase. The rejection identifies the owning field and gives both
CLI/raw-MCP and typed-MCP forms that an agent can use on its next call. Invalid
input reaches no article provider.

## Current facts

The 2026-09-01 MCP study remains useful field evidence, but its checkout does
not include a transcript or aggregate from which the 15 field-expression calls
or 6 multi-concept gene calls can be reconstructed. The retained report says
that BioMCP 0.8.25 served four tools over stdio, Claude Code 2.1.252 drove 31
tasks with `claude-sonnet-5`, and 52 of 179 BioMCP calls failed. Fifteen of the
52 put field-scoped syntax in a plain-text query field; six more put free text
where a symbol was required. `gene:RB1` and `TPMT mercaptopurine` are the two
retained examples. These numbers are dated observational evidence, not a
reproducible checkout benchmark. Together with ticket 1099's 19 wrong-section
calls, the 15 field-expression calls account for 34/52 failures (65%); the six
gene-field calls are additional and must not be folded into that percentage.

The code-level defect remains reproducible at HEAD
`eddcac9eb683ae58f68a0c1d8256b549f6e02ae3` (2026-09-04):

- `ArticleSearchArgs.keyword` is documented as provider-neutral free text and
  `ArticleSearchArgs.gene` as a gene-symbol filter. `article_search_request`
  turns both into `ArticleSearchFilters`.
- `validate_search_page_request` runs before `search_page` chooses or invokes a
  backend, and delegates value checks to
  `entities/article/filters.rs::validate_search_filter_values`. This is the
  owning validation boundary for normal article searches.
- That boundary currently rejects only recognized PubMed/Europe PMC author,
  affiliation, and journal forms (`AUTH:`, `AFFILIATION:`, `JOURNAL:`, and the
  documented bracket forms). Its positive unit corpus expressly permits other
  colon text. It therefore accepts `gene:RB1` as `keyword` and accepts
  `TPMT mercaptopurine` as `gene`; both continue into backend planning and
  provider query construction.
- Direct CLI article search and the raw MCP `biomcp` tool share the same Clap
  command and article entity path. Typed MCP `search` first maps JSON fields to
  that same CLI grammar in `mcp/shell.rs::search_args`, then executes it. The
  schema-valid but semantically invalid typed call consequently belongs at the
  shared article validation boundary, not in a second MCP-only parser.
- `search all --keyword ...` builds the same `ArticleSearchFilters`, but a
  section error is currently rendered inside a successful search-all card.
  `search all --gene ...` can fan the value out to gene, variant, drug, trial,
  article, pathway, and PGx legs. Its `PreparedInput::new` boundary must
  apply the same two user-input checks before any concurrent leg starts; an
  article-leg-only error is too late for this surface.
- `article batch` retrieves article identifiers and has no keyword or gene
  search field. Variant-article batch inputs describe a structured variant and
  already have their own per-item identity validation. Generic entity batches,
  gene cards/related commands, PGx/GWAS/diagnostic/variant search fields, trial
  biomarker text, and study gene arguments have distinct semantics and do not
  pass through `ArticleSearchFilters`. The study did not retain enough evidence
  to change all of those surfaces safely. They are not part of this ticket.

The original ticket incorrectly treated every plain-text query and every gene
slot as one grammar problem. This design narrows the observable examples to the
normal article-search family, where both inputs, one validation owner, all
transport paths, and provider non-execution can be proved together.

## Accepted behavior

### Misplaced field expressions

In article `keyword`, the case-insensitive labels `gene:`, `disease:`, and
`drug:` are reserved when the label begins the trimmed runtime value or the
label's first character follows Unicode whitespace or `(`. Existing
recognition of `AUTH:`, `AFFILIATION:`, `JOURNAL:`, `[author]`, `[au]`, `[ad]`,
`[journal]`, and `[jour]` remains. Recognition is diagnostic only: BioMCP
rejects the request and does not parse, move, or execute the supplied value as
another filter.

For `gene:RB1`, the stable diagnostic must convey all of the following without
echoing the whole untrusted query:

- `keyword` is provider-neutral and does not accept `gene:` filter syntax;
- use `--gene RB1` for CLI or raw MCP; and
- use the typed MCP `gene` field, for example `"gene":"RB1"`.

The disease and drug diagnostics analogously name `--disease`/`disease` and
`--drug`/`drug`. Existing author, affiliation, and journal guidance remains at
least as specific as it is now. No provider-specific query language is added.

The lexical boundary is part of the compatibility contract. These remain
ordinary keyword text and must reach the selected provider unchanged apart
from that provider's existing escaping/URL encoding:

- `NM_004333.6:c.1799T>A`;
- `protein:protein interaction`;
- `oncogene:RB1` and `MYGENE:RB1`;
- `ratio 1:2`; and
- `BRAF[variant]`.

A literal quote byte in the value before the label prevents a match. Thus a
runtime value whose first and last characters are quotes, such as
`"gene:gene interaction"`, remains ordinary keyword text. Shell quoting alone
does not put quote bytes in argv: a CLI caller that needs this compatibility
escape writes `-k '\"gene:gene interaction\"'`, and typed MCP sends the
keyword array value `{"keyword":["\"gene:gene interaction\""]}`. Those
literal quote bytes continue to the selected provider; BioMCP does not strip
them or implement a quote grammar. Conversely, runtime values `gene:RB1`,
`GENE:RB1`, `melanoma (gene:RB1)`, and `gene:"RB1"` are rejected. This is
deliberately a narrow recognizer, not a general `name:value` parser. A quote
elsewhere has no special stateful effect: only the exact lexical boundary above
controls recognition.

### Obvious non-symbol gene values

For normal article search, a supplied `gene` value is trimmed consistently
with search-all and the existing provider builders. The result must be nonempty
and contain no Unicode whitespace. All existing nonempty whitespace-free
spellings remain accepted, including case, digits, and punctuation such as
`BRAF`, `braf`, `PD-L1`, and `H3-3A`. Empty/whitespace-only input and any
internal space, tab, newline, or non-breaking space are rejected before
provider work. This ticket does not invent a complete HGNC-symbol regex or
perform a network identity lookup during validation.

For `TPMT mercaptopurine`, the stable diagnostic says that `gene` accepts one
symbol, gives `TPMT` as a valid example, and says to put the additional concept
in `keyword`; it includes `--gene TPMT --keyword mercaptopurine` and the typed
MCP `gene` plus `keyword` field shape. It does not echo arbitrary input into a
shell command.

### Errors and transport behavior

- Human CLI and `search all` reject at the request boundary, write one
  sanitized `Error: Invalid argument: ...` line to stderr, write no stdout, and
  exit 2. They do not render a successful empty/partial search card.
- CLI `--json` writes the existing structured `invalid_argument` error to
  stdout, writes no stderr, and exits 2.
- Raw and typed MCP return exactly one sanitized text content item with
  `isError: true` and the same actionable diagnostic. The typed article
  keyword input is the published array shape, for example
  `{"entity":"article","keyword":["gene:RB1"]}`. A schema-valid typed
  request remains a tool error rather than becoming a transport failure or a
  JSON-RPC schema error.
- Validation happens before cache lookup, alias/entity discovery, provider
  request planning that performs I/O, or any PubTator3, Europe PMC, PubMed,
  Semantic Scholar, or LitSense2 request.

Values are already passed as argv elements rather than through a shell, so the
current bug is not OS command injection. Automatic conversion would still turn
untrusted text into a different semantic query and would require ambiguous
quoting rules. Rejecting at the owned boundary avoids that query-injection
class, and fixed examples plus existing human-output sanitization prevent the
new error path from creating a command or terminal-control injection sink.

## Test-first implementation plan

1. Extend the existing article filter unit tests first. The red table must
   cover the reserved prefix positions/casing/quoted-value examples above,
   exact field-specific guidance, the full literal compatibility corpus, valid
   whitespace-free gene spellings, and empty plus internal
   space/tab/newline/non-breaking-space gene failures. Include reserved labels
   after ASCII and non-breaking whitespace, false-positive prefixes such as
   `oncogene:`/`MYGENE:`, and literal quote bytes. Test the classifier as a
   table, not one special-case string.
2. Extend `article_search_request` tests to prove that direct/positional keyword
   normalization reaches the shared validator and that both malformed shapes
   fail before a backend plan is returned. Do not implement the rule in Clap or
   only in `mcp/shell.rs`.
3. Keep one article-owned query-input validator in
   `entities/article/filters.rs`; normal article validation and search-all
   preparation must call it rather than copying recognizers. A narrow
   `pub(crate)` re-export through `entities/article/mod.rs` is permitted if
   needed. Call it from `PreparedInput::new` after slot normalization and
   before anchor/variant-context construction or dispatch-plan creation. A
   focused unit table must prove both malformed inputs fail preparation, while
   the literal-colon corpus and valid gene tokens still produce the existing
   plan. Derived genes from a valid structured variant are not reclassified as
   user-supplied article `gene` text.
4. Extend the existing `article_usage_stderr` executable test with one local
   atomic counting HTTP fixture and point all five article candidate bases at
   it. Prove flagged and positional keyword, multiword/empty gene, and
   malicious-suffix inputs exit 2 with the exact clean line, no reflected
   control/shell text, and a zero counter. Add matching human and JSON
   search-all subprocess cases whose environment redirects every network base
   reachable from the gene fan-out plan to the same counter; both must fail
   before dispatch with zero requests across all legs, no result card, and the
   specified stdout/stderr/exit behavior. Keep an explicit test-owned list of
   the redirected bases adjacent to the plan assertion so adding a gene-fanout
   provider cannot silently weaken this proof. Add the direct article JSON
   assertions in the existing JSON error contract.
5. Extend the existing rmcp client contract, without changing the typed schema
   or mapper, to send `{"entity":"article","keyword":["gene:RB1"]}` as
   typed search, `biomcp search article -k gene:RB1` as raw MCP, and a
   multiword typed `gene`. Require `isError: true`, one text content item, exact
   common guidance containing both CLI/raw and typed correction forms, and zero
   requests at a counting article-provider fixture. A subsequent harmless call
   must still work, proving the server session was not terminated. Also pin a
   literal-quote typed value as accepted so transport syntax cannot be confused
   with quote bytes in the value. Send malformed keyword and gene forms through
   raw MCP `search all` too, under the same all-leg counting environment used by
   the CLI proof. Typed MCP has no `search all` entity, so typed convergence is
   deliberately the article-search branch and this ticket must not add one.
6. Use the existing article source/query construction tests and the dedicated
   `run-article-semanticscholar-source-search.sh` request file to prove a
   literal-colon case reaches Semantic Scholar with the same decoded `query`
   value. Make the fixture handler require that exact decoded query and record
   path plus decoded query, rather than merely counting a path. In the article
   and MCP specs, make the already shared article fixture log every candidate
   search route, snapshot or reset that owned log immediately before rejected
   calls, and assert its exact unchanged line count afterward; never infer zero
   work from an empty result or from a log carrying earlier page requests. Keep
   all fixture behavior deterministic, local, and serialized under the runner's
   existing article-page ownership.
7. Update the existing article keyword reference, article user guide, CLI
   reference, article/search-all command help, embedded `biomcp list article`
   and `biomcp list search-all` text, and their current docs assertions. State
   the runtime-quote distinction precisely and document that article/search-all
   `gene` accepts one symbol. Add executable CLI behavior to
   `spec/entity/article.md` and raw/typed MCP parity to `spec/surface/mcp.md`;
   specification pages use prepared binaries and fixtures and must not invoke
   Cargo.

## Scope and exclusions

In scope: normal article `keyword` (including `-k`, `-q`, `--query`, and the
positional alias) and article `gene` validation; the corresponding keyword and
gene slots plus pre-dispatch boundary of `search all`; raw MCP for both CLI
commands and typed MCP article search; fixed actionable diagnostics; local
zero-request and positive serialized-request proof; embedded/public docs; and
the two existing executable-contract pages. Article `get` sections and article
batch accept neither field and are not affected. Internal gene-, disease-, and
pathway-related article legs continue to use `ArticleSearchFilters`; this
ticket adds no earlier validation or new user grammar to those commands, and
their existing valid constructed values must remain valid.

Out of scope: accepting or translating field syntax; a general query language;
arbitrary unknown `name:value` text; PubMed boolean/bracket grammar beyond the
existing recognized forms; changing ranking, provider selection, or results;
validating gene identity online; tightening non-article gene fields; article
ID batch; variant-article batch semantics; generic entity batches; get/related,
trial, study, PGx, GWAS, diagnostic, variant, protein, disease, drug, pathway,
author, adverse-event, or discover grammar; changing MCP JSON Schema; rerunning
the 31-task study; or backporting to 0.8.x.

This is a validation tightening. A caller that intentionally needs a reserved
colon phrase can include literal quote bytes as keyword text; ordinary shell or
JSON delimiters alone are not such bytes. All other documented colon and
biomedical notation remains compatible. No successful field expression is
being removed because BioMCP never interpreted these expressions as filters.

## Acceptance

- Every malformed and compatibility example in **Accepted behavior** has a
  table-driven unit assertion at the owning article boundary.
- Flagged/aliased and positional article search, search-all, human CLI, JSON
  CLI, raw MCP, and typed MCP exhibit the specified error
  wording/channel/status. The typed tests use `keyword`'s array schema, and
  runtime quote bytes—not JSON or shell delimiters—control the literal escape.
- Counting local transports observe exactly zero requests for every rejected
  direct-article and search-all case, including every provider reachable from
  the gene fan-out plan. The deterministic Semantic Scholar request log
  observes the exact decoded literal-colon keyword for a positive case and the
  owned article-search request log is unchanged across rejected CLI and MCP
  calls.
- Ordinary article searches and the existing native author/journal rejection
  corpus keep their current behavior.
- Focused Rust unit/integration tests, `tests/rmcp_client_contract.rs`, affected
  Python/docs contracts, `make lint`, `make test`, and `make spec` pass. No
  AlphaGenome code or feature behavior changes, so `make full-feature-check`
  is not required.

## Dependencies and constraints

No unlanded dependency blocks implementation. The article validation seam from
`dd733c65`, typed-search-to-CLI mapping already on main, structured invalid
argument/exit-2 policy from ticket 0353, raw parsed-command authorization from
ticket 1017, and deterministic request-contract infrastructure are landed
inputs to preserve, not prerequisites to add.

`cargo package --locked --allow-dirty --list` reports exactly 1,300 paths at
this review, the enforced package ceiling. Add no file or packaged sidecar and
do not raise/exempt the ceiling. Keep implementation, tests, docs, and specs in
the existing files named above. `src/mcp/shell.rs` is exactly 2,136 lines at its
pinned inventory baseline and must not grow; this design requires no edit
there. Current likely production files are
`src/entities/article/filters.rs` (265 lines), its narrow re-export in
`src/entities/article/mod.rs` if needed, `src/cli/search_all/plan.rs` (645),
and the existing article/search-all help and list modules including
`src/cli/list/literature.rs` (210); keep each previously sub-1,000-line file
below 1,000 lines and keep net production `src/` growth at or below 120 lines
without a ratchet allowance. Keep the currently 899-line
`tests/json_error_contract.rs` below 1,000 lines.

## Review

- Ticket amendment/evidence pass (2026-09-04, HEAD `eddcac9e`): verified the
  shared article validator, direct CLI and typed/raw MCP routing, search-all
  fan-out timing, current native-field
  recognizer and literal corpus, error/exit contracts, batch exclusions,
  request-proof fixtures, landed dependencies, exact 1,300-path package
  inventory, and relevant source-size baselines. Corrected the unreproducible
  study claim, the ambiguous 65% arithmetic, the stale all-query/all-gene
  scope, the undecided accept-versus-reject behavior, and the missing lexical,
  transport, safety, provider-I/O, docs, and executable-contract requirements.
- Independent design review (2026-09-04, HEAD `eddcac9e`): **ACCEPT after
  revision.** The review made the reserved lexical boundary and literal-quote
  transport contract exact, added empty/Unicode-whitespace gene cases, pinned
  the typed keyword array shape and one-content-item tool error, required
  search-all to validate before plan/fan-out with an all-leg zero-request
  subprocess proof, made the positive request log assert the decoded query,
  named affected aliases/help/docs and non-article exclusions, and preserved
  the exact 1,300-path package ceiling, 2,136-line `shell.rs` baseline, and
  +120 net production-line ceiling. No implementation dependency or unresolved
  design choice remains.
- Independent code review (2026-09-04): **REJECT.** The shared article
  full-text fixture did not log all five candidate-search routes, the MCP spec
  used unreachable port 9 instead of the fixture-owned request log, and the MCP
  page was not declared as a serialized consumer of that mutable log. The
  direct, search-all, raw-MCP, and typed-MCP assertions also sampled cases and
  substrings instead of pinning the exact common gene/disease/drug diagnostics;
  the ASCII-whitespace boundary and a genuinely empty argv element were
  missing.
- Code-review remediation (2026-09-04): the owned fixture now logs decoded
  PubTator, Europe PMC, PubMed, Semantic Scholar, and LitSense2 candidate
  searches and exports the LitSense2 base. Article, author, and MCP pages are
  declared as one serialized fixture/log consumer group. Article and MCP specs
  reset and snapshot that log immediately before rejected calls and require the
  exact line count to remain unchanged. Live local counters replace the
  rejection tests' port-9 absence assumption. Exact table-driven assertions now
  cover gene:, disease:, and drug: through -k, -q, --query, the
  positional article form, search-all human and JSON output, raw MCP article
  and search-all, and typed MCP article search. MCP assertions require
  isError: true, exactly one text item, and exact text. The corpus now includes
  melanoma gene:RB1 and an actual empty --gene argv element. Typed empty
  gene remains correctly rejected earlier by its published minimum-length
  schema and is not misrepresented as a shared-validator case.
- Remediation mutation evidence (2026-09-04): the fixture lifecycle test
  requires the exact ordered five-route log, so deleting or misnaming any
  candidate-route hook fails it; the runner lifecycle test requires MCP in the
  serialized article-page invocation, so returning it to parallel execution
  fails. The exact diagnostic tables fail on wording drift, wrong field-specific
  guidance, a missing alias/transport, a non-text or additional MCP item, or
  isError: false; the live counters fail if any rejected call reaches a
  redirected provider. Focused owning-validator, article planner, search-all
  planner, eight-test subprocess, JSON contract, raw/typed MCP, four fixture
  lifecycle, and 39 parallel-isolation tests passed after remediation.
- Remediation verification (2026-09-04): cargo formatting and diff checks
  passed; make lint passed, including Clippy and the quality ratchet. The
  serialized spec-contract lane passed 140 article/author/MCP checks with 4
  intentional skips, followed by the remaining 6 and 10 checks. Package
  inventory remains exactly 1,300 paths, src/mcp/shell.rs remains unchanged at
  2,136 lines, tests/json_error_contract.rs is 946 lines, and net production
  src growth remains +79 lines.
- Test-first implementation (2026-09-04): the new owning-boundary table first
  failed because `gene:RB1` returned `Ok(())`. Added one article-owned validator
  and reused it from normal article validation and `PreparedInput::new` before
  variant parsing or fan-out. Focused unit, CLI/JSON subprocess, rmcp, list/docs,
  and strict Semantic Scholar decoded-query checks pass. Rejected subprocess and
  MCP cases observed zero requests at local counters; the literal-quote typed
  MCP case was accepted and reached the local provider. Direct `--gene` values
  are now stored trimmed, matching search-all and provider construction.
- Implementation discovery (2026-09-04): the typed literal-quote positive MCP
  proof needs `BIOMCP_TEST_UNPACED_ORIGIN` for its local Semantic Scholar fixture;
  this is fixture policy, not product behavior. The amended ticket was already a
  working-tree change and was preserved. No `src/mcp/shell.rs` edit or packaged
  path was needed.
- Implementer gates (2026-09-04): `make lint` passed, including Clippy and the
  quality ratchet; `make test` passed 3,126 Rust tests with 30 intentional
  skips, 890 Python contracts with 3 skips, and strict MkDocs; `make spec`
  passed every routine page (including 39 MCP checks, one skipped) plus 38
  parallel-isolation checks and 8 static checks. Package inventory remains
  exactly 1,300 paths, `src/mcp/shell.rs` remains 2,136 lines, and net
  production `src/` growth is +79 lines (test modules excluded), below +120.
- Final independent code re-review (2026-09-04): **ACCEPT with no findings**.
  Verified validation precedes direct backend planning and all search-all
  parsing/fan-out, every required exact diagnostic and lexical boundary, all
  five decoded request-log routes, serialized fixture ownership, unchanged-log
  rejection proofs, MCP envelope/session behavior, typed empty-value schema
  handling, documentation, and all package/size rails. Focused validator,
  planner, CLI/JSON, MCP, fixture lifecycle, and serialization tests passed.

## Completed 2026-09-04

Article search now rejects reserved `gene:`, `disease:`, and `drug:` filter
expressions when they are placed in plain-text query fields, with exact
field-specific guidance and no provider request. The same validator runs before
ordinary article backend planning and before any seven-leg `search all`
fan-out. Literal colons and literal quote bytes remain searchable, and direct
gene input is normalized to one trimmed, whitespace-free symbol.

Final primary-agent verification passed: `make lint`; `make test` (3,126 Rust
tests passed with 30 skipped, 892 Python tests passed with 3 skipped, and strict
documentation built); and `make spec` (all routine pages, including 140
serialized article/author/MCP checks with 4 skips, 39 parallel-isolation
contracts, and 8 static specs passed). The package inventory remained exactly
1,300 paths, `src/mcp/shell.rs` remained 2,136 lines, and net production growth
remained +79 lines.
