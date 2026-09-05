---
flow: build
priority: 7
---

# Continue citation and reference pages without losing coverage

`biomcp article citations` and `biomcp article references` return only the
first requested page. They expose no offset, continuation, or completion
state. A researcher knowledge-base run therefore stopped at an arbitrary edge
count. A live `article citations 22237106 --limit 1` request returned one edge
without a way to request the provider's next page. The original observation
and provider evidence are preserved in git at commit `995fa87e` in
`sdlc/issues/feature-make-citation-pages-complete-and-continuable.md`.

Semantic Scholar's citation and reference endpoints accept an `offset` and
return `offset`, optional `next`, and `data`; they do not return a total. A
three-page live check at offsets 0, 1, and 2 returned distinct edges and next
offsets 1, 2, and 3.

## Required behavior

Both graph commands add `--offset <u64>`, defaulting to zero, while retaining
the existing `--limit` range of 1-100 and default of 10. The offset remains a
`u64` from Clap through the entity and source request plan, the provider
response, JSON, and continuation builder: do not narrow it through `usize`.
`0` and `u64::MAX` are valid CLI values; a negative value or an integer above
`u64::MAX` is rejected before any provider request. The request plan sends the
decimal value as Semantic Scholar's `offset` query parameter.

After the existing seed-ID resolution, each command makes exactly one request
to the selected graph endpoint for the requested offset and limit. It returns
that page's `data` in provider order, including any provider duplicates. It
does not fetch a following page, refetch an earlier page, or deduplicate
against state that is unavailable to this invocation. PMCID resolution may
retain its existing Europe PMC lookup; the graph anchor remains the resolved
Semantic Scholar paper ID.

The JSON result keeps the existing `article` and `edges` members and adds these
exact members:

```json
{
  "pagination": {
    "offset": 0,
    "limit": 10,
    "returned": 10,
    "next_offset": 10,
    "coverage_status": "continuable"
  },
  "_meta": {
    "next_commands": [
      "biomcp article citations 22663011 --limit 10 --offset 10"
    ]
  }
}
```

`coverage_status` is a closed two-value contract:

- `continuable`: the provider supplied a valid advancing `next`; `next_offset`
  is that value and `_meta.next_commands` contains exactly the corresponding
  graph continuation command.
- `exhausted`: the provider omitted `next` or returned it as JSON `null`;
  `next_offset` is JSON `null` and `_meta.next_commands` is an empty array.

`offset`, `limit`, `returned`, `next_offset`, and `coverage_status` are always
present. `returned` is the number of edge objects returned after successful
decoding. Neither the pagination object nor any other graph result member adds
`total`, an inferred total, `has_more`, or a locally inferred next offset.
Exhaustion is provider-relative: it means only that this response carried no
provider continuation.

The provider page is accepted only when `offset` is a JSON unsigned integer
equal to the requested offset. A missing or null offset, a negative,
fractional, string, overflowing value, or an offset that differs from the
request is a malformed provider response and fails the command without
emitting page data. `next` may be absent or null (both mean `exhausted`), or it
must be a JSON unsigned integer strictly greater than the returned provider
offset. A malformed, overflowing, equal, or decreasing `next` likewise fails
closed; BioMCP must not emit edges with an unusable continuation or substitute
`offset + returned`. An empty `data` array follows the same rule: an advancing
`next` is a valid empty but continuable page, while absent/null `next` is a
valid empty exhausted page.

One shared continuation builder accepts the caller's validated, trimmed
article ID, direction (`citations` or `references`), limit, and provider next
offset. It uses the repository's shell-safe `NextCommand` renderer and emits
this canonical argument order:

```
biomcp article <citations|references> <caller-id> --limit <limit> --offset <next>
```

It preserves the caller-facing ID (for example, a PMCID stays a PMCID) rather
than replacing it with the resolved Semantic Scholar paper ID. The same built
string feeds JSON `_meta.next_commands` and the Markdown `Next:` line, so the
two surfaces cannot drift. No command is built or printed for an exhausted
page or an invalid provider response.

Markdown retains the anchor heading, edge table, identifiers, title, all
intents, influence flag, and first context. It then prints page offset, page
size, returned count, the provider-relative `continuable` or `exhausted`
coverage status, and the statement that no exact total is available. A
continuable page ends with the shell-safe copyable `Next:` command line. An
exhausted page has no `Next:` line. An empty page retains the existing
`No related papers returned` table row and still prints the same pagination
footer, including a continuation for the valid empty-continuable case.

## Acceptance

- Captured citation and reference fixtures cover first, middle, terminal, and
  empty pages. They assert the exact provider query (`fields`, `limit`, and
  `offset`), one graph request after seed resolution, provider edge order, and
  preservation of edge identity, every intent, influence, and context data.
- Citation and reference tests each cover valid advancing `next`, absent
  `next`, explicit null `next`, and empty pages. The next-page fixture proves a
  different provider page is returned without automatic cross-page fetching,
  deduplication, or a repeat of page zero.
- A table-driven malformed-response test covers missing/null/mismatched
  `offset`; negative, fractional, string, and overflowing offsets; and
  malformed, overflowing, equal, and decreasing `next`. Every case fails
  without page output or a continuation command.
- Boundary tests prove offset zero and `18446744073709551615` reach the request
  plan unchanged, while `-1` and `18446744073709551616` fail before network
  access. A generated continuation containing `u64::MAX` is not required,
  because no larger valid advancing `next` exists.
- Exact JSON assertions cover the full `pagination` and `_meta` objects for a
  continuable page and an exhausted page, assert the deliberate absence of
  `total` and `has_more`, and cover valid empty-continuable and
  empty-exhausted pages.
- Exact Markdown assertions cover first, middle, terminal, empty-continuable,
  and empty-exhausted pages for both directions. They prove the same safe
  continuation string appears in JSON and Markdown and that an ID containing
  shell-significant characters cannot become executable syntax.
- CLI help and durable command references document `--offset`, provider-
  relative exhaustion, and the lack of a total for both graph commands.
- Raw MCP executes both commands in JSON and Markdown mode and preserves the
  same pagination and continuation contract. No typed MCP graph tool is added;
  the typed-tool inventory/schema tests explicitly keep citations and
  references out of the typed surface.
- Routine `make lint`, `make test`, and `make spec` pass, including executable
  graph specs for the JSON, Markdown, CLI-help, and raw-MCP surfaces.

Boundary: this ticket adds one-page-at-a-time offset continuation and honest
provider-relative coverage to the existing Semantic Scholar citation and
reference traversals. It does not add date-based refresh, merge graph
providers, calculate a total, fetch all pages, deduplicate across pages, or
recover missing citation passages. Ticket 1145 remains the downstream owner of
passage recovery and must preserve this ticket's edge and pagination contract.
Ticket 1143 changes exact-author paper richness and is not a dependency.
