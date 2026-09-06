---
flow: build
priority: 5
deps: [1145]
---

# Return an opt-in rich page of papers for an exact author

## Goal

`biomcp author papers <exact-provider-id> --full` returns one bounded Semantic
Scholar author-paper page with enough source metadata for local search,
citation ranking, paper identity checks, and author pivots. The default command
remains the existing compact page.

The current endpoint already returns the requested rich fields in the same
page. A 2026-09-04 capture verified `abstract`, `publicationDate`, citation and
reference counts, open-access data, fields of study, publication types,
authors, and external identifiers. The original observation is preserved in
git at `995fa87e` under
`sdlc/issues/feature-add-a-full-record-mode-to-author-paper-pages.md`.

This ticket does not traverse citation edges or inspect JATS. Ticket 1145 owns
directed-edge traversal and JATS reference matching and lands first; 1143 must
retain its accepted graph behavior and prove the author request makes no graph,
paper-detail, Europe PMC, full-text, or JATS request.

## Request and page contract

Add `--full` only to `author papers`. The author ID remains the exact,
case-sensitive `semanticscholar:<ASCII-decimal-id>` grammar, `--limit` remains
1-100 with default 10, and `--offset` remains a nonnegative machine-sized
integer. Compact and rich calls make the same single logical request:

```text
GET /graph/v1/author/<percent-encoded-id>/papers?fields=<mode-fields>&offset=<offset>&limit=<limit>
```

The compact field list and compact output are byte-for-byte unchanged. The rich
field list is exactly:

```text
paperId,corpusId,externalIds,title,abstract,venue,year,publicationDate,citationCount,referenceCount,influentialCitationCount,isOpenAccess,openAccessPdf,fieldsOfStudy,publicationTypes,authors.authorId,authors.name
```

No batch enrichment or N+1 lookup is allowed. The operation has one monotonic
35-second response deadline around request admission, shared retry/cache work,
body read, decode, projection, and rendering. The shared HTTP layer may retry a
transient attempt under its existing policy, but it may neither start nor
complete work after that absolute deadline. Timeout is the existing sanitized
Semantic Scholar unavailable command error, exit 1.

Every successful page must contain an unsigned `offset` equal to the requested
offset, at most `limit` data rows, and either absent/null `next` or an unsigned
`next` strictly greater than that offset. Missing, negative, fractional,
string, overflowing, mismatched, equal, or decreasing pagination values, an
oversized page, malformed JSON, or a wrong-typed rich field fails the complete
command through the existing sanitized provider-error envelope. It emits no
partial page. An exhausted empty page is a successful available result.

Compact and rich projection share row admission: retain a row only when its
`paperId` and `title` Unicode-trim to nonblank strings. A paper ID is an opaque
provider identifier; BioMCP does not narrow admission to 40-hex because the
existing compact fixture and provider contract permit other nonblank IDs.
Optional rich fields do not affect admission. Retain admitted rows in provider
order, including
duplicate papers; do not sort, merge, or deduplicate either papers or authors.
Thus matching compact and rich responses contain the same admitted paper IDs
in the same order. Tests include invalid rows before and between duplicates so
this property is not fixture-position dependent.

## Frozen rich JSON

The top-level rich object keeps the existing `author`, `papers`, `pagination`,
and `_meta` keys and shapes. Every admitted rich `papers[]` object always has
exactly these keys:

```json
{
  "paper_id": "0123456789abcdef0123456789abcdef01234567",
  "corpus_id": 277710284,
  "pmid": "40215974",
  "pmcid": null,
  "doi": "10.1016/j.fixture.2024.01.001",
  "arxiv_id": null,
  "title": "A rich author paper fixture",
  "abstract": "Source abstract.",
  "journal": "Fixture Medicine",
  "year": 2024,
  "publication_date": "2024-01-31",
  "citation_count": 17,
  "reference_count": 23,
  "influential_citation_count": 2,
  "is_open_access": false,
  "open_access_pdf": {
    "url": "https://example.invalid/paper.pdf",
    "status": "HYBRID",
    "license": null
  },
  "fields_of_study": ["Medicine"],
  "publication_types": ["JournalArticle"],
  "authors": [{
    "identity": {
      "kind": "exact_provider",
      "id": "semanticscholar:2059910739"
    },
    "display_name": "First Author"
  }]
}
```

Only the allowlisted `PubMed`, `PubMedCentral`, `DOI`, and `ArXiv` external-ID
keys project into the four flattened nullable fields; unknown external IDs are
not serialized. Counts preserve zero, booleans preserve false, and absent/null
source scalars serialize as null rather than defaults. Present string scalars
are Unicode-trimmed and may become the empty string, which remains distinct
from null. An absent/null list is null while a present empty list is `[]`.
List order and duplicates are preserved, and list strings receive the same
trim-only normalization.

An `openAccessPdf` object remains an object even when all three members are
null; an absent/null object is null. Preserve every supplied author array
entry. A valid nonblank ASCII-decimal `authorId` becomes an exact-provider
identity; otherwise its identity is null. A missing/null author name is null,
and a present name is trim-only. Rich author rows do not add affiliations,
ORCID, inferred identity, or per-author requests.

The rich page retains the existing pagination object and metadata ordering.
`_meta.source_status` is exactly one available Semantic Scholar row on success.
Evidence URLs appear once per admitted paper in row order, including
duplicates. Construct each from the fixed
`https://www.semanticscholar.org/paper/` base by percent-encoding the normalized
`paperId` as exactly one UTF-8 path segment. Use `url::Url` path-segment
serialization, or an equivalent explicit encoder for its WHATWG special-URL
path-segment set: C0 controls and space, `"`, `#`, `<`, `>`, `?`, backtick,
`{`, `}`, `/`, `%`, and backslash are uppercase `%HH`; other ASCII remains
literal, including `$`, `&`, `;`, `+`, `,`, `:`, `=`, and `@`. Handle `.` and
`..` as encoded data rather than allowing URL dot-segment normalization. Do not
use interpolation or `join`. Article follow-ups appear
in the same row order using the existing PMID, DOI, arXiv, then valid 40-hex
paper-ID preference and `NextCommand` quoting. An opaque non-40-hex paper ID
therefore remains an admitted row with an encoded evidence URL but is never an
article follow-up fallback. The final continuation, when present, is:

```text
biomcp author papers <exact-provider-id> --full --limit <limit> --offset <next>
```

It follows all paper commands. Compact continuation text remains unchanged and
does not acquire `--full`.

## Markdown and public surfaces

Compact Markdown is byte-for-byte unchanged. Rich Markdown keeps the existing
paper-page heading and provider order, but each paper block displays every
field in the frozen rich object: missing scalar/object/list values render
`unknown`, present empty lists render `none`, and false/zero render literally.
Authors render in byline order as `<display name or unknown>
(<provider-qualified ID or unknown>)`; open-access PDF renders URL, status, and
license without turning a provider URL into an executable command.

The rich renderer uses this exact field order and labels for every paper
(angle-bracketed terms are substitutions). It emits `### Authors` even for a
missing or empty byline, with one `- unknown` or `- none` row respectively, and
ends with the provider continuation only when `pagination.next` is nonnull:

```text
# Papers for <safe code span author ID>

## <safe inline title>

- Paper ID: <safe code span value>
- Corpus ID: <value>
- PMID: <safe code span value>
- PMCID: <safe code span value>
- DOI: <safe code span value>
- arXiv ID: <safe code span value>
- Journal: <safe inline value>
- Year: <value>
- Publication date: <safe code span value>
- Citations: <value>
- References: <value>
- Influential citations: <value>
- Open access: <true, false, or unknown>
- Open-access PDF URL: <safe code span value>
- Open-access PDF status: <safe code span value>
- Open-access PDF license: <safe code span value>
- Fields of study: <safe comma-joined inline values>
- Publication types: <safe comma-joined inline values>

### Authors

1. <safe inline display name> (<safe code span provider-qualified ID>)

### Abstract

<safe inline abstract or unknown>

Next: <safe code span complete continuation command>
```

The ID/PMID/PMCID/DOI/arXiv/PDF/date slots use `unknown` without code markup
when null and an empty code span when present-empty. Numeric slots use
`unknown` when null. Missing lists use `unknown`; present empty lists use
`none`. A null PDF makes each of its three displayed slots `unknown`, while a
present all-null object has the same human projection but remains distinct in
JSON. A missing byline and an empty byline remain distinct in JSON even though
their Markdown sentinel rows differ as specified above. Separate paper blocks
have exactly one blank line between them, and output ends in one newline.

Use the repository's safe inline/code-span renderers for every provider value.
Titles, abstracts, venue, dates, identifiers, PDF values, field/type names, and
author names containing pipes, angle brackets, backticks, newlines, quotes,
backslashes, dollar signs, semicolons, and ampersands cannot create headings,
tables, links, raw HTML, code fences, or shell commands. JSON preserves the
normalized plain values. Complete Markdown and complete JSON fixtures must
agree field-for-field; no `Debug` representation is public.

CLI help, `biomcp list author`, `docs/user-guide/author.md`, and the CLI
reference describe the compact default, rich opt-in, one-page/100-row bound,
and source-exact limitation. `spec/entity/author.md` executes both modes.

Raw MCP `biomcp` executes rich Markdown and `--json` through the same CLI path.
Its text and parsed JSON are byte-identical to direct CLI output, its request
log is identical, and provider command errors are MCP errors rather than
successful empty pages. This ticket adds no typed tool: typed `get author`
remains detail-only, typed search/get schemas remain byte-for-byte unchanged,
and the seven tool names, catalog description, measured byte/token ceilings,
and raw-tool description remain within their current ratchets. Catalog tests
explicitly prove that no `author_papers` typed tool or `full` author-get field
appears.

## Acceptance

1. Source-plan tests pin the exact compact/rich field lists, encoded path,
   caller offset/limit, API-key behavior, cache mode, and construction without
   I/O. Loopback request logs prove each CLI and raw-MCP rich page uses only the
   one author-papers endpoint and zero graph/detail/batch/Europe-PMC requests.
2. Wire/projection tests cover every rich field, null versus false/zero/empty
   string/empty list, unknown external IDs, an all-null PDF object, complete
   mixed-validity bylines, invalid row admission, duplicate retention, and
   byte-equal paper identity/order between compact and rich views. A hostile
   admitted `paperId` whose normalized value is `A/?#% \n雪` is preserved
   exactly in JSON, produces the exact evidence URL
   `https://www.semanticscholar.org/paper/A%2F%3F%23%25%20%0A%E9%9B%AA`,
   and produces no `get article` command. A second opaque ID `$&;+, :=@`
   produces exactly
   `https://www.semanticscholar.org/paper/$&;+,%20:=@`; `.` and `..` produce
   `https://www.semanticscholar.org/paper/%2E` and
   `https://www.semanticscholar.org/paper/%2E%2E`, respectively, rather than
   traversing the base path. A 40-hex control still produces both its unchanged
   evidence URL and follow-up.
3. Apply an exhaustive pagination matrix to compact and rich pages: valid
   terminal/continuing/empty pages, missing and mismatched offsets, every
   malformed `next` shape, and `limit + 1` rows. Tests prove exactly one
   provider page is requested and a returned `next` is never prefetched.
4. Paused-time tests hold the real request future across the exact 35-second
   boundary and prove one bounded unavailable result, no late output or second
   page, and no task surviving the response. Companion transport, HTTP,
   rate-limit, oversize-body, and decode cases preserve sanitized errors and do
   not leak provider bodies, URLs, credentials, or hostile sentinels.
5. Execute compact and rich CLI Markdown/JSON plus raw MCP against the captured
   fixture. Compare complete outputs, continuation commands, metadata, hostile
   content containment, error envelopes, and request logs. Parse every emitted
   command with the real CLI parser and recover the original arguments without
   executing injected shell text. Separate compact/rich CLI and raw-MCP hostile
   fixtures pin the JSON ID and encoded evidence URL above and the exact
   Markdown line ``- Paper ID: `A/?#% 雪` `` (the embedded control newline is
   coalesced with the adjacent space); neither surface
   contains an injected link/heading nor a hostile-ID article command.
6. Pin typed-MCP non-expansion and rerun the accepted ticket-1145 graph/JATS
   fixture family: citation/reference commands, evidence states, pagination,
   directed matching, and request bounds do not change. The author fixture
   itself records no graph or JATS traffic.
7. Run focused Rust source/entity/renderer/CLI tests, Python CLI/MCP/docs
   contracts, and `spec/entity/author.md`, then `make lint`, `make test`, and
   `make spec`; finish with `git diff --check` and the locked/offline package
   list at exactly 1,300 paths.

## Ownership and exclusions

`sources/semantic_scholar.rs` owns the two field lists, request planning, wire
types, page validation, and deadline-aware request. `entities/author/papers.rs`
owns row admission, normalization, public compact/rich types, metadata, and
continuations. `render/markdown/author.rs` renders those types; CLI and raw MCP
only select the mode and serialization.

Add no dependency or packaged path. Keep `src/sources/semantic_scholar.rs`
below 1,000 lines by moving its existing in-file tests if necessary only
through a package-neutral rename/removal, not by raising a ratchet. Do not grow
`src/mcp/shell.rs` or change its authorized baseline. Do not raise any existing
over-threshold source allowance; update a baseline only for measured movement
caused by a package-neutral extraction. The package stays exactly 1,300 paths.

This ticket does not export a corpus, create a local index, infer metadata,
resolve authors across providers, establish ORCID identity, add affiliations,
fetch paper detail/full text, traverse citation/reference/recommendation edges,
parse JATS/HTML/PDF, change article search, add a typed MCP tool, or alter
ticket 1145's directed-citation evidence contract.

## Review

The initial ticket established the opt-in outcome but did not freeze the rich
schema, row/page validation, failure and nullability semantics, resource
bounds, renderer/MCP behavior, ownership, or compatibility with 1145. This
revision resolves those findings and awaits independent design re-review.
