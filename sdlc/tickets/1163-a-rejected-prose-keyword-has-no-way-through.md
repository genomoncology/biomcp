---
flow: build
priority: 4
deps: []
---

# A rejected prose keyword has no way through

## Goal

A caller whose article keyword legitimately contains `gene:`, `disease:`, or
`drug:` as prose can recover from the rejection without changing the
provider-neutral query they intended. The current fixed diagnostic only points
to the structured filter, which expresses a different search.

Reconfirmed on `origin/main` at `cc8c9eca`: `review of drug: safety`,
`disease: mechanisms`, and `melanoma (gene:RB1)` are rejected before provider
work. The shared recognizer in `src/entities/article/filters.rs` intentionally
matches a case-insensitive reserved label when it begins the trimmed runtime
value or its first character follows Unicode whitespace or `(`. Record 1100
established that boundary and its zero-provider-work behavior.

The previous draft described the compatibility escape too broadly. There is no
quote parser and enclosing a whole multiword value does not protect a later
label. Runtime `"review of drug: safety"` still rejects `drug:`, while runtime
`review of "drug: safety"` is admitted because `drug:` is immediately preceded
by a literal `"` byte. BioMCP preserves both quote bytes as query text.

## Complexity and implementation route

Contract 1 + state/timing 0 + reach 1 + proof 2 + cost of error 1 = 5.
There is no minimum floor, so this is Level 2. The change spans several public
CLI/MCP/documentation cases but remains pure pre-work validation; the main risk
is user-visible guidance, and hostile/exact-byte compatibility needs focused
proof. Use GPT-5.6 Luna High for implementation and GPT-5.6 SOL Medium for
independent code review.

## Accepted behavior

Do not change `reserved_keyword_field`, its label set, case folding, trimming,
boundary predicate, or fixed field scan order (`gene:`, then `disease:`, then
`drug:`). Do not add balanced-quote, escape, or recursive parsing. A literal
double quote is merely a non-whitespace, non-`(` preceding byte. A closing quote
is optional to the recognizer and, if supplied for readable prose, is ordinary
provider-visible text.

Teach one exact rule: put a literal `"` byte immediately before **each**
reserved label intended as prose. These runtime values are admitted unchanged:

| case | exact runtime keyword |
| --- | --- |
| label at start | `"disease: mechanisms"` |
| label after whitespace | `review of "drug: safety"` |
| label after `(` | `melanoma ("gene:RB1")` |
| multiple labels | `"gene:RB1" and "drug: trametinib" in "disease: melanoma"` |

The whole-value `"review of drug: safety"`, parenthesized
`melanoma (gene:RB1)`, or any multiple-label value with one still-recognized
label remains rejected. Multiple recognized labels retain current deterministic
field precedence rather than textual-order precedence.

### Stable diagnostics and envelopes

Replace the three current messages with these exact, static strings (the
leading `Invalid argument: ` remains the existing `BioMcpError` display):

```text
keyword is provider-neutral and does not accept gene: filter syntax. Use --gene RB1 for CLI or raw MCP, or the typed MCP field, for example "gene":"RB1". To search literal gene: text, put a literal double-quote byte immediately before every reserved label: CLI/raw MCP -k '"gene: expression"'; typed MCP "keyword":["\"gene: expression\""].
keyword is provider-neutral and does not accept disease: filter syntax. Use --disease melanoma for CLI or raw MCP, or the typed MCP field, for example "disease":"melanoma". To search literal disease: text, put a literal double-quote byte immediately before every reserved label: CLI/raw MCP -k '"disease: mechanisms"'; typed MCP "keyword":["\"disease: mechanisms\""].
keyword is provider-neutral and does not accept drug: filter syntax. Use --drug vemurafenib for CLI or raw MCP, or the typed MCP field, for example "drug":"vemurafenib". To search literal drug: text, put a literal double-quote byte immediately before every reserved label: CLI/raw MCP -k '"drug: safety"'; typed MCP "keyword":["\"drug: safety\""].
```

Each message is selected only from the recognized label; it never interpolates
or echoes the supplied keyword. For a rejected human article or search-all
command, stdout is empty, stderr is exactly
`Error: Invalid argument: <message>\n`, and exit status is 2. CLI `--json`
writes no stderr, exits 2, and writes the existing envelope with exactly
`error.code = "invalid_argument"`, `error.message = "Invalid argument:
<message>"`, and `_meta.not_found = false` (no source/recovery/limit members).
Raw and typed MCP return `isError: true` and exactly one text item equal to
`Error: Invalid argument: <message>` for either MCP output selector; errors do
not become successful Markdown/JSON cards or JSON-RPC parameter failures.

### Exact transport examples

Use explicit `--source semanticscholar` in every positive executable contract.
Its request plan sends the built free-text value as the decoded `query` query
parameter, so a strict local fixture can distinguish admission from a merely
non-failing command. The canonical runtime keyword is
`review of "drug: safety"`.

The native POSIX command is:

```sh
biomcp search article --source semanticscholar -k 'review of "drug: safety"' --limit 1
```

POSIX removes the surrounding single-quote delimiters. The resulting argv is:

```text
[biomcp, search, article, --source, semanticscholar, -k, review of "drug: safety", --limit, 1]
```

and `article_search_request` stores the shown keyword element as
`filters.keyword`, without shell delimiters and with both literal `"` bytes.

The raw MCP `biomcp` tool request is exactly:

```json
{"command":"biomcp search article --source semanticscholar -k 'review of \"drug: safety\"' --limit 1","json":false}
```

After JSON decoding, the command string is byte-for-byte the native command.
`shlex::split` produces the exact argv above and the shared runtime keyword is
identical. The `json:true` form changes only output selection.

The typed MCP `search` request is exactly:

```json
{"entity":"article","keyword":["review of \"drug: safety\""],"source":"semanticscholar","limit":1,"json":false}
```

The existing mapper produces
`[biomcp, search, article, --keyword, review of "drug: safety", --source,
semanticscholar, --limit, 1]`; Clap normalization produces the same runtime
keyword. The `json:true` form again changes only output selection. `keyword`
remains the published one-to-three string array; no schema change is permitted.

For each distinct input decoder—native CLI, raw MCP, and typed MCP—make one
successful call through the owned fixture and require exactly one
`GET /graph/v1/paper/search`. Its decoded `query` is exactly
`review of "drug: safety"`; it contains no structured gene, disease, or drug
filter, and the request has only Semantic Scholar's ordinary `query`, `fields`,
and `limit` parameters. One output selector per decoder is enough for this
propagation proof. Separately extend existing mapper/envelope assertions to
show that selecting JSON rather than Markdown changes the mapped argv only by
adding `--json`; every non-output argument and the resulting runtime keyword
remain identical. A fixture response supplies one stable Semantic Scholar row
so success cannot be inferred from an empty or degraded card.

### Safety and compatibility

Keep one owning unit table beside `reserved_keyword_field` and
`validate_query_inputs`. It proves all three exact static messages, no input
reflection, the four admitted quote-before-each-label rows above, whole-value
quoting, partial escaping, deterministic field precedence, and representative
start/whitespace/parenthesis and false-boundary rows from record 1100. Existing
author/affiliation/journal and gene-value regression tables remain intact; do
not duplicate their entire corpora in every surface layer.

Extend the existing human/JSON CLI and raw/typed MCP error-contract tables with
representative rows for the new fixed messages and unchanged envelopes rather
than multiplying every label, spelling, and output selector. Keep direct
article and search-all pre-provider rejection coverage. One representative
hostile rejected keyword must include quote, backslash, control, and shell
metacharacter bytes; prove the fixed message contains none of those bytes,
provider logs remain empty, managed state is absent, and a test-owned sentinel
is not created. Existing direct-argv raw-MCP shell-safety contracts continue to
own the broader metacharacter matrix.

## Ownership and documentation

The diagnostic belongs only in
`src/entities/article/filters.rs::validate_query_inputs`; direct article,
search-all, raw MCP, and typed MCP already converge there. Extend its existing
unit sidecar plus `src/cli/article/tests/filters.rs`,
`src/cli/search_all/tests/plan.rs`, `tests/article_usage_stderr.rs`,
`tests/json_error_contract.rs`, and `tests/rmcp_client_contract.rs`. Extend the
prepared-fixture contracts in `spec/entity/article.md` and
`spec/surface/mcp.md`; do not add a parallel recognizer or fixture-only bypass.

Correct the overbroad “quotes around the value/text” guidance, naming the
immediate-before-each-label rule and surface-specific encodings, in:

- `docs/reference/article-keyword-search.md`;
- `docs/how-to/search-all-workflow.md`;
- `docs/user-guide/cli-reference.md` and `docs/user-guide/article.md`;
- article after-help in `src/cli/commands.rs`; and
- embedded `biomcp list article` / `list search-all` text in
  `src/cli/list/literature.rs` and `src/cli/list/helpers.rs`.

Update existing exact help/list/docs assertions with those replacements. Do not
change MCP tool names, descriptions, input JSON Schemas, catalog entries, or
inventory. `src/mcp/shell.rs` and `src/mcp/catalog.rs` must be byte-identical to
the implementation base; typed behavior is proved through the public tool and
existing mapper tests.

## Boundary

This is diagnostic and guidance remediation for the existing article/search-all
keyword rule. It does not accept or translate field syntax, strip quote bytes,
change provider selection/ranking/results, add a query language, alter the
author/affiliation/journal or gene-value recognizers, touch non-article fields,
or change any MCP schema. Article get/batch/graph/fulltext and variant-article
paths are unaffected.

No unlanded dependency is required; `deps` is deliberately empty. Record 1100,
the Semantic Scholar selectable source, typed-search mapping, raw MCP allowlist,
and fixture runners are landed behavior to preserve, not dependencies.

## Size and verification

`cargo package --list --allow-dirty --locked --offline` reports exactly 1,300
paths at the design base. Add no files, raise no ceiling, and retain exactly
1,300 package paths. Keep production changes to replacement wording in the
existing owners: `src/entities/article/filters.rs` is 330 lines,
`src/cli/commands.rs` 693, `src/cli/list/literature.rs` 211, and
`src/cli/list/helpers.rs` 156. All remain under the enforced 700-line Rust-file
cap and net production `src/` line growth is zero. Tests/docs/specs grow only in
their existing files; split a test sidecar rather than raising a ratchet if an
existing test file approaches its enforced cap.

Implement test-first. Run the focused filter, article/search-all CLI, JSON,
rmcp client, help/list/docs, Semantic Scholar request-plan, and affected
mustmatch contracts, then `make lint`, `make test`, and `make spec`. Run the MCP
schema/catalog inventory tests on final HEAD and verify a base-to-HEAD byte diff
for `src/mcp/shell.rs` and `src/mcp/catalog.rs`. Re-run the offline package list
and require exactly 1,300 paths. No AlphaGenome behavior or feature graph is
touched, so `make full-feature-check` is not required.

## Done, observably

- Every fixed rejection teaches both the structured correction and the exact
  per-label literal rule without reflecting input, and all surfaces retain
  their frozen error envelopes and zero provider work.
- The owning unit table pins start, interior, parenthesized, multiple,
  partially escaped, precedence, and representative record 1100 boundaries.
- Native POSIX, raw MCP, and typed MCP decoder calls deliver identical runtime
  quote bytes to exactly one explicit Semantic Scholar request; JSON selection
  adds only `--json`, leaving every non-output argument and runtime keyword
  unchanged, and representative hostile text remains inert and performs no
  work when rejected.
- Named shipped guidance is accurate, MCP schemas/catalog are byte-unchanged,
  all source/package ratchets hold, and the standard gates pass.

## Review

Accepted after fresh independent SOL design review. The review corrected the
base and size inventory, added the Level 2 route, reduced duplicated proof, and
confirmed that JSON selection adds only `--json` while every query-bearing
argument and the runtime keyword remain unchanged.
