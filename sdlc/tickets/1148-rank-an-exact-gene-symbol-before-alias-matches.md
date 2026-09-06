---
flow: build
priority: 8
---

# Rank an exact gene symbol before alias matches

## Goal

An exact canonical gene-symbol search returns that gene before alias-only
matches, and every surface builds its first detail command from that same row.
On 2026-09-04, `biomcp --json search gene ODC1 --limit 5` returned
`SLC25A21` before `ODC1` and advertised `biomcp get gene SLC25A21`.

The current owner, `entities::gene::search_page`, preserves the MyGene page
order, applies local filters, and truncates. `cli::gene::handle_search` then
hands that order independently to Markdown and JSON; raw and typed MCP invoke
the same CLI path. Ranking therefore belongs in `search_page`, before its
`SearchPage<GeneSearchResult>` crosses the entity boundary, not in a renderer
or next-command builder.

## Exact admission and bounded acquisition

Trim the query with Rust `str::trim`. A query is eligible for canonical-symbol
promotion only when the trimmed value is nonempty, no longer than the existing
256-byte limit, contains only ASCII letters, digits, `_`, or `-`, and contains
at least one ASCII letter. Eligibility is case-insensitive: `ODC1`, `odc1`, and
` OdC1 ` select the same comparison key. Do not rewrite the bytes used by the
existing MyGene/Lucene query construction.

For an eligible query, make one MyGene request with the current query and
provider filters, `from=0`, and `size=50` (the existing gene search ceiling).
Only when `total <= 50` is this a complete candidate set and promotion is
allowed. This common exact-symbol/alias route includes a canonical hit that was
beyond a caller's first requested page.

Validate the caller's original `limit`, `offset`, and `offset + limit` against
the current CLI/MyGene bounds before substituting that acquisition window, so
the alternate fetch cannot admit an input rejected today.

For `total > 50`, do not claim a global ordering BioMCP did not observe:

- at offset zero, apply the current local filters and return the requested
  slice of that first response without promotion or deduplication;
- at a positive offset, issue the current page request with its existing
  `fetch_limit`, `from=offset`, and filters, then preserve the current
  filter/truncate result unchanged.

Thus an eligible search makes exactly one request for a complete set or an
overflow first page, and at most two requests for a positive-offset overflow.
All MyGene requests remain at or below 50 rows and within the existing 10,000
result-window checks. Ineligible free text takes the current single-request
path byte-for-byte. No speculative page walking, exact-detail lookup, provider
boost, fallback source, or cache bypass is added.

## Complete-set rank, filter, dedupe, and page algorithm

For the complete `total <= 50` path, perform these operations exactly once in
`entities::gene::search_page` and in this order:

1. Apply the existing normalized gene-type, chromosome, and region predicates
   to raw MyGene hits. Pathway and GO remain provider predicates. A canonical
   row rejected by any requested filter is not admitted by ranking.
2. Transform retained hits with the existing `transform::gene::from_mygene_hit`.
3. Stable-partition rows whose nonblank `symbol.trim()` equals the trimmed
   query under ASCII case-insensitive comparison ahead of every other row.
   Preserve provider order within both partitions. Alias and name matches are
   never exact; no exact row means provider order is unchanged.
4. Deduplicate first-wins after promotion. Two rows duplicate when their
   nonblank trimmed Entrez IDs agree, or when at least one lacks an Entrez ID
   and their nonblank trimmed symbols agree ASCII-case-insensitively. Distinct
   nonblank Entrez IDs remain distinct even if symbols tie. Rows with blank or
   missing symbols and Entrez IDs are never equal and retain their positions.
5. Set `SearchPage.total` to the retained complete-set length, then slice the
   half-open range `[offset, min(offset + limit, len))`. An offset at or beyond
   the retained length returns an empty page. Use checked/saturating bounds;
   do not index with unchecked `offset + limit`.

Comparison trimming/case folding never changes serialized provider values.
A missing/blank symbol cannot be exact and cannot create a detail command.
`search_next_commands_gene` remains unchanged: for a nonempty page it uses the
first-row symbol only when nonblank, followed by `biomcp list gene`; an empty
page has no commands. This ticket does not search later rows for a command.

The fixture's complete raw order is an alias-only `SLC25A21`, exact `ODC1`, a
duplicate of that ODC1 Entrez identity, and `OAZ1`. After processing, the
canonical sequence is exactly:

```json
["ODC1", "SLC25A21", "OAZ1"]
```

With `--limit 1`, offsets 0, 1, 2, and 3 return respectively `ODC1`,
`SLC25A21`, `OAZ1`, and `[]`; totals are three and adjacent pages neither
repeat nor omit a row. This proves the exact row is promoted even though it was
beyond the original first one-row page. A two-exact-row case with distinct
Entrez IDs pins stable tie order; separate cases pin same-Entrez removal,
blank identities, exact-row filter rejection, and no-exact order preservation.

## Exact production-surface acceptance

Extend the existing MyGene provider-contract fixture and gene spec in place.
Do not add a second fixture family.

- Exercise executable CLI Markdown and JSON for `ODC1`, lowercase/trimmed
  spellings, all four adjacent offsets above, and `--limit 2 --offset 1`.
  Assert complete ordered JSON `results[].symbol` arrays, `count`, pagination,
  and `_meta.next_commands`, not substring membership. Offset zero is exactly
  `["biomcp get gene ODC1", "biomcp list gene"]`; its Markdown table has ODC1
  as the first data row and the exact
  `Showing 1-1 of 3 results. Use --offset 1 for more.` footer.
- Exercise the raw MCP `biomcp` tool and typed MCP `search` tool for both
  Markdown and JSON at offset zero and one. JSON result arrays and commands are
  byte-equal to CLI. Markdown result tables are byte-equal to CLI; MCP adds the
  existing exact footer `## Next commands` whose first bullet is
  ``- `biomcp get gene ODC1` `` on offset zero and
  ``- `biomcp get gene SLC25A21` `` on offset one, followed by the list command.
- Parse every emitted detail command with the real CLI parser. A provider row
  whose symbol contains whitespace, quote, backslash, dollar, backtick,
  semicolon, and ampersand pins `NextCommand` shell quoting: it parses as one
  unchanged symbol argument and is rejected by existing gene-get validation
  before any provider request or shell side effect. It is never considered an
  exact match for a valid symbol-shaped query.
- Pin request-log lines: the complete eligible route is exactly one
  `size=50&from=0` request; the overflow offset-zero route is one such request;
  the positive-offset overflow route is that probe plus the current bounded
  page request; free text remains exactly one current request. Rendering and
  command construction add zero provider requests.
- Cover adversarial query/filter input through production paths: leading and
  trailing Unicode whitespace, mixed ASCII case, every Lucene metacharacter
  already escaped by `mygene_query_term`, alias-only hits, exact/alias ties,
  duplicate/missing identities, chromosome case and `chr` prefix, normalized
  gene type, overlapping/nonoverlapping region, pathway, GO, an exact row
  filtered out, totals 50 and 51, last legal MyGene window, and invalid
  limit/window rejection before provider work.
- Snapshot the typed `search` gene branch, tool inventory/count, CLI help, and
  JSON schema before and after: they are byte-identical. Typed MCP retains its
  current limit 25/offset 1000 caps; CLI retains limit 1-50 and the MyGene
  10,000-window rules.

The fixture/spec must compare the complete deterministic Markdown bodies and
JSON arrays for the named cases and use the request log for acquisition proof.
Unit tests pin the pure stable partition/deduplication matrix and the 50/51
branch. No assertion may rely only on `contains`, a live MyGene response, or a
renderer-only reordered fixture.

## Boundaries, size, and gates

This ticket changes only bounded ordering/deduplication in gene search's entity
layer plus its colocated tests and existing fixture/docs/spec assertions. It
does not change gene-get identity, MyGene payload types or provider relevance,
query matching/escaping, local filter meaning, CLI arguments/caps, JSON fields,
pagination fields, Markdown templates, next-command construction, search-all,
MCP schemas/tools, other entities, dependencies, or add files.

`src/entities/gene.rs` is exactly 3,859 lines on this base and may not exceed
3,859 after implementation; extract tests within existing test modules rather
than raising the ratchet. The source package inventory remains exactly 1,300.
Run focused Rust entity/CLI/MCP tests, the Python provider/surface contracts,
and the gene mustmatch spec, then `make lint`, `make test`, `make spec`, exact
package inventory, and `git diff --check`.

Ticket 1149 also changes ordering, but in diagnostic synonym resolution. It is
not a dependency and neither ticket may absorb or sequence the other's work.

## Review

The first draft did not specify how an exact hit outside the requested first
page could be observed, how filtering/deduplication interacted with paging, or
how production CLI and MCP surfaces proved the same first command. This
revision freezes those contracts.

Independent design re-review accepted the immutable ticket at `13fd8a3f`. The
reviewer confirmed the bounded complete-set acquisition and overflow fallback,
stable ranking/deduplication/pagination rules, production CLI/raw-MCP/typed-MCP
proofs, hostile-input handling, unchanged public contracts, exact source and
package ceilings, and ticket 1149's explicit non-dependency.
