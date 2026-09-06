---
flow: build
priority: 6
deps: [1144]
---

# Recover the passage that connects a citing paper to a cited paper

## Goal

`biomcp article citation-evidence <citing-id> <cited-id>` returns bounded source
text for one directed citation pair. Semantic Scholar context wins by default.
When that edge has no nonblank context, BioMCP inspects open Europe PMC JATS,
resolves the cited reference by an exact stable identifier, and returns the
paragraphs whose unambiguous bibliographic markers link to that reference.

A 2026-09-04 research run found useful Semantic Scholar context on only 23 of
61 incoming edges. Captured provider verification found three linked
paragraphs in `PMC12923956` for DOI `10.1038/nature10725`, and one in
`PMC13200738` for DOI `10.1016/j.artmed.2020.101822`. These are deterministic
fallback examples, not a universal-coverage claim. The original observation is
preserved in git at `995fa87e` under
`sdlc/issues/feature-recover-citation-evidence-when-upstream-context-is-missing.md`.

The result retrieves evidence only. It does not summarize the passage, infer
how the cited work was used, or claim that the citation supports a conclusion.

## Directed edge lookup

Resolve the trimmed citing and cited inputs through the existing article-to-
Semantic-Scholar identity path. A missing input, unsupported identifier, or
missing seed remains the existing command error; no five-state result is
fabricated. Keep the caller spellings for display and command construction,
while matching graph rows only by the resolved Semantic Scholar `paperId`.
Comparison is ASCII-case-insensitive because the IDs are hexadecimal; title,
author, DOI, and provider position are never substitutes for a missing or
different `paperId`.

Each singleton batch response must contain exactly one nonnull paper with a
nonblank 40-character hexadecimal `paperId`. A null row, extra row, absent or
malformed ID, or an edge whose `citedPaper.paperId` is absent/malformed is a
provider-decode command error. Resolving both caller inputs to the same ID is
not rejected locally; it matches only an explicit provider self-reference.

The directed assertion is always looked up as:

```text
GET /graph/v1/paper/<resolved-citing-paper-id>/references
```

and an edge matches only when `citedPaper.paperId` equals the resolved cited
paper ID. Do not reverse to the cited paper's `citations` endpoint and do not
accept the opposite direction.

Use a fixed provider page size of 100, begin at offset zero, and follow at most
three pages (300 returned edge rows and three graph requests). Reuse ticket
1144's exact response validation: every page must return the requested unsigned
`offset`; absent/null `next` is exhausted; present `next` must be an unsigned
integer strictly greater than the returned offset. Missing, negative,
fractional, string, overflowing, mismatched, equal, or decreasing values fail
the command before page evidence is emitted. A page with more than the
requested 100 rows is likewise malformed, so the 300-row bound cannot depend
on provider compliance. Follow only the provider's `next`; never infer
`offset + returned`, refetch page zero, or request the advertised page after a
match has been found.

Resolve the two seeds with at most two existing Semantic Scholar batch
requests; PMCID inputs may additionally retain one existing Europe PMC bridge
lookup apiece. The graph phase has a ten-second deadline in addition to its
three-request cap. The complete command, including the optional JATS request
and blocking parse, has a twenty-second deadline. A deadline or the 300-edge
cap reached while the provider still advertises another page is a bounded
Semantic Scholar unavailable error, not proof that the edge is absent. An
exhausted search with no match constructs the existing
`BioMcpError::NotFound` with entity `directed citation`, ID
`<citing-id> -> <cited-id>`, and the suggestion below. Its exact rendered
message (including the blank line) is:

```text
directed citation '<citing-id> -> <cited-id>' not found.

Semantic Scholar exhausted the directed reference pages without finding this pair.
```

If one page contains duplicate matching edges, retain their contexts in edge
order and context-array order. Unicode-trim each context, discard blank values,
and deduplicate the remaining strings by exact bytes, first occurrence wins.
Stop after that matching page; a duplicate edge never triggers another page or
another full-text request. Provider duplicates on ordinary `citations` and
`references` pages remain untouched as required by 1144.

## Provider context and forced full text

Without `--fulltext`, one or more surviving Semantic Scholar contexts produce
`context_from_provider` and no Europe PMC full-text request. With no surviving
context, attempt JATS recovery once. `--fulltext` forces that same JATS path
even when provider contexts exist; the contexts remain present in JSON, but
the status and passages report the forced JATS attempt. A failed forced attempt
does not silently fall back to `context_from_provider`.

The full-text fallback is deliberately Europe PMC JATS only. It does not call
the broader full-text waterfall, PMC HTML, PDF extraction, or `--pdf`. Resolve
the citing PMCID from the citing seed's `PubMedCentral` external ID, or through
the existing exact PMID/DOI-to-PMCID bridge when necessary, then make at most
one `/<PMCID>/fullTextXML` request. Keep the existing eight-MiB source-body
limit, the one-million-node external-XML limit, DTD/entity protections, and
source-context sanitization. Missing PMCID, 404/204, transport failure,
oversize, malformed XML, a non-article root, or a document without a body maps
to `fulltext_unavailable`; upstream bodies, URLs, and parser details do not
enter the result.

## Exact JATS reference identity

Add one pure structural extractor beside the existing JATS reference logic in
`transform::article::jats::refs`, re-exported only through the article
transform facade. It receives the resolved cited paper's typed IDs and the raw
JATS document; it does not parse rendered Markdown.

Consider only `<ref>` descendants of a `<ref-list>`. Extract a `<pub-id>` only
when its Unicode-trimmed `pub-id-type` is ASCII-case-insensitively `doi`,
`pmid`, or `pmcid`. Extract an `<ext-link>` only when its Unicode-trimmed
`ext-link-type` is ASCII-case-insensitively `doi`; use its normalized element
text, not an untyped `xlink:href`. Missing/blank/unrecognized type attributes
are ignored. Match in this strict precedence, stopping at the first identifier
class with any matches:

1. DOI
2. PMID
3. PMCID

DOIs are Unicode-trimmed, ASCII-lowercased, and stripped once or repeatedly of
`doi:`, `https://doi.org/`, `http://doi.org/`, `https://dx.doi.org/`, or
`http://dx.doi.org/`; the remaining value must begin `10.` and contain `/`.
PMIDs are Unicode-trimmed, may have one case-insensitive `pmid:` prefix, and
must otherwise be ASCII digits; compare their decimal value so leading zeroes
do not differ. PMCIDs are Unicode-trimmed, ASCII-uppercased, may have one
`pmcid:` prefix, and must be `PMC` plus ASCII digits; compare the canonical
`PMC<decimal>` value. Do not strip arbitrary punctuation, percent-decode,
compare titles, or use untyped free text.

A lower-priority identifier match is rejected when that same `<ref>` contains
a conflicting valid higher-priority identifier. Zero matching references, or
more than one distinct `<ref>` matching at the winning precedence, produces
`reference_unresolved`. Duplicate copies of the same normalized identifier
inside one `<ref>` do not make it ambiguous. The selected `<ref>` must have one
nonblank `id`; XML ID and `rid` comparison is exact and case-sensitive after
Unicode trimming.

## Markers, passage scope, order, and bounds

An eligible marker is an `<xref ref-type="bibr">` in the article `<body>`.
Split `rid` on Unicode whitespace. It is unambiguous only when the distinct
nonblank token set contains exactly the selected reference ID. A grouped
marker naming the target plus another reference is ignored. Adjacent separate
single-target `<xref>` elements remain eligible; markup grouping alone does not
make them ambiguous. Missing `rid`, a different case, or a dangling ID never
matches.

Recover the closest ancestor `<p>` under `<body>`, excluding paragraphs inside
`ref-list`, `table-wrap`, `table`, `fig`, `caption`, `fn-group`,
`supplementary-material`, or `boxed-text`. Do not recover a list item, table
cell, caption, abstract, reference entry, or floating object. Render plain
inline text with the existing JATS whitespace collapse and inline sanitization;
do not retain XML/HTML markup.

Return at most three passages in document order. Multiple target markers in
the same paragraph yield one passage. Deduplicate by the paragraph's XML node
identity, first occurrence wins; never deduplicate different paragraphs merely
because their text agrees. Each passage is at most 1,200 Unicode scalar values.
For a longer normalized paragraph, locate the first scalar of the first
eligible marker in the normalized output, choose a 1,198-scalar slice whose
start is `clamp(marker_start - 599, 0, paragraph_len - 1198)`, and prefix
and/or suffix one Unicode ellipsis when that side was removed. The complete
passage is therefore at most 1,200 scalars, always retains the marker's first
scalar, and the response never exceeds 3,600 passage scalars. Tests pin marker
positions at both boundaries as well as the scalar-count boundaries.

Each passage carries this exact locator:

```json
{
  "pmcid": "PMC12923956",
  "ref_id": "bib7",
  "section_path": ["Results"],
  "paragraph": 12,
  "marker": "7"
}
```

`section_path` is the outer-to-inner sequence of nonblank ancestor `<sec>`
titles after the same text normalization. `paragraph` is the one-based ordinal
among all eligible body paragraphs, including eligible paragraphs without a
citation. `marker` is the normalized inline text of the first eligible target
marker and may be empty. These locators and the canonical Europe PMC
`/<PMCID>/fullTextXML` evidence URL allow source inspection without claiming a
stable browser line number.

If the reference resolves but no eligible unambiguous marker reaches an
eligible paragraph, return `citation_marker_unlinked`. Grouped-only markers,
markers outside the allowed scope, and markers whose paragraph becomes blank
all take that state.

## Frozen JSON and messages

The successful CLI JSON object always contains `citing`, `cited`, `status`,
`message`, `source`, `provider_contexts`, `passages`, `fulltext_locator`, and
`_meta`. `citing` and `cited` are the existing resolved related-paper objects.
`provider_contexts` and `passages` are arrays in every state, never null.
`source` and `fulltext_locator` are present and nullable. Every passage has
exactly `text`, `locator`, and `evidence_url`. `_meta` always has exactly
`source_status`, `evidence_urls`, and `next_commands`; this terminal command's
`next_commands` is always empty.

The status is a closed five-value enum with these exact projections:

| `status` | exact `message` | `source` | passages / full-text locator |
| --- | --- | --- | --- |
| `context_from_provider` | `Semantic Scholar supplied citation context for this directed edge.` | `semantic_scholar` | empty / null |
| `context_from_fulltext` | `Open-access JATS linked the cited reference to the returned passage.` | `europe_pmc_jats` | nonempty / nonnull |
| `fulltext_unavailable` | `Structured open full text was unavailable for the citing paper.` | null | empty / null |
| `reference_unresolved` | `Structured full text was available, but the cited reference could not be resolved exactly.` | `europe_pmc_jats` | empty / nonnull |
| `citation_marker_unlinked` | `The cited reference was resolved, but no unambiguous in-text citation marker linked to it.` | `europe_pmc_jats` | empty / nonnull |

`provider_contexts` contains the normalized Semantic Scholar strings even in a
forced-fulltext state. `fulltext_locator`, when nonnull, is exactly
`{"pmcid":...,"evidence_url":...}`. `source_status` is always ordered
Semantic Scholar then Europe PMC JATS: Semantic Scholar is `available` after a
validated matching edge; Europe PMC JATS is respectively `not_requested`,
`available`, or `unavailable`. `evidence_urls` contains the Semantic Scholar
citing-paper URL first and cited-paper URL second; JATS-attempt states append
the Europe PMC XML URL only when a response body was successfully parsed.

The metadata object uses these exact object shapes and canonical URLs (shown
for the provider-context path):

```json
{
  "source_status": [
    {"source": "semantic_scholar", "status": "available"},
    {"source": "europe_pmc_jats", "status": "not_requested"}
  ],
  "evidence_urls": [
    {"source": "semantic_scholar", "url": "https://www.semanticscholar.org/paper/<citing-paper-id>"},
    {"source": "semantic_scholar", "url": "https://www.semanticscholar.org/paper/<cited-paper-id>"}
  ],
  "next_commands": []
}
```

Any response body that successfully passes XML parsing, including a parsed but
unusable non-article or bodyless document, appends
`{"source":"europe_pmc_jats","url":"https://www.ebi.ac.uk/europepmc/webservices/rest/<PMCID>/fullTextXML"}`;
a missing body, transport/HTTP failure, oversize body, or XML parse failure
does not. The Europe PMC status remains `unavailable` whenever the outcome is
`fulltext_unavailable`. No metadata member or per-state object member is
omitted.

Invalid input, unresolved seeds, directed-edge not-found, malformed Semantic
Scholar pagination, Semantic Scholar transport/decode failure, and graph
budget/deadline exhaustion remain command errors with the existing JSON error
envelope and exit 1. The five states are successful, exit-0 evidence outcomes.
Full-text transport/decode details are bounded into the exact public
`fulltext_unavailable` message. JSON and Markdown must not contain injected
provider error sentinels.

## Markdown and graph command placement

Add a dedicated citation-evidence renderer with this exact section order and
punctuation (angle-bracketed values denote substitution, not literal output):

```text
# Citation evidence

Citing: <code-span(resolved citing label)>
Cited: <code-span(resolved cited label)>
Status: <exact status message>

## Provider contexts

1. <code-span(first context)>

## Passages

### Passage 1

<code-span(passage text)>

Locator: PMCID <code-span(pmcid)>; reference <code-span(ref_id)>; section <code-span(section path joined by " > ")>; paragraph <decimal>; marker <code-span(marker)>
Evidence: <code-span(evidence_url)>

Full text: <code-span(fulltext_locator.evidence_url)>
```

The provider-context heading/block is omitted when its array is empty and the
passage heading/block is omitted when its array is empty. `Full text:` is
omitted when the locator is null and otherwise follows the final present
context/passage block (or `Status:` when both arrays are empty). Repeat the
passage sub-block with consecutive one-based headings and exactly one blank
line between blocks. A resolved label is the existing
`article_related_label`: PMID, then DOI, then arXiv ID, then Semantic Scholar
paper ID, with title only if none exists. A blank section path or marker
renders the code span for `-`. Output ends in one newline. A shared new
evidence/edge code-span wrapper uses a delimiter one backtick longer than the
longest run in the value and the standard padding needed for leading/trailing
backticks or spaces; it does not alter 1144's existing root-continuation
rendering. Thus hostile provider/JATS text cannot add tables, headings, links,
HTML, or commands.

Tests pin the complete Markdown output byte-for-byte for every state and
separately prove its values agree with the JSON projection. No `Debug`
rendering is user-visible.

One shared helper builds:

```text
biomcp article citation-evidence <citing-id> <cited-id>
```

with `NextCommand`, preserving trimmed caller IDs and argument order. The same
string feeds ordinary graph discovery and tests; renderers never concatenate
it. On `article citations`, the edge paper is the citing ID and the caller
anchor is the cited ID. On `article references`, the caller anchor is citing
and the edge paper is cited. The edge ID uses the existing related-paper
executable preference: PMID, then DOI, then arXiv ID, then Semantic Scholar
paper ID. If either argument is unavailable, emit no command.

Ticket 1144's root graph `_meta.next_commands` remains exactly the pagination
continuation array and never receives evidence commands. Instead, a graph edge
whose contexts contain no nonblank value gains an optional edge-local
`_meta.next_commands` containing exactly the evidence command; it is absent on
an edge with useful provider context. In Markdown, only that blank edge's
existing `-` Context cell changes to `Try: <safe variable-length code span>`.
The graph heading, rows and cells with nonblank context, edge order and
duplicates, pagination sentences, empty-page row, and `Next:` footer remain
byte-for-byte as landed by 1144. Update 1144's exact graph matrix only for this
single intentional blank-context-cell delta and assert its root continuation
arrays and footer text are unchanged.

## Production-path acceptance

Extend the existing captured article graph/full-text fixture family and
`spec/entity/article.md`; do not create a second fixture family or a tracked
file.

1. Pure graph tests pin the references direction, page size 100, offsets from
   validated `next`, one/two/three-page request bounds, first-page early stop,
   duplicate matching edges and contexts, exact paper-ID comparison, exhausted
   not-found, cap exhaustion, and graph/whole-command deadline errors. The full
   malformed offset/next matrix from 1144 is applied to this traversal.
2. Pure JATS tests cover both verified documents plus DOI/PMID/PMCID precedence,
   every normalization rule, conflicting higher-priority IDs, zero/multiple
   refs, duplicate IDs in one ref, missing/blank IDs, case-sensitive `rid`,
   grouped and adjacent markers, dangling markers, excluded scopes, nested
   section paths, paragraph ordinals, document-order retention, paragraph
   dedupe, equal-text non-dedupe, exact 1,199/1,200/1,201-scalar boundaries,
   three/four-passage truncation, malformed/oversized/node-limit XML, and blank
   recovered text.
3. Executable CLI Markdown and JSON cover all five statuses, default provider
   precedence, `--fulltext` success and failure despite useful provider
   context, the two verified JATS examples, exact messages/nullability/source
   status/provenance, directed not-found, and bounded request counts/order.
   Compare complete JSON objects and complete Markdown, not `contains` checks.
4. Raw MCP `biomcp` executes those same default and `--fulltext` cases in
   Markdown and JSON. Assert non-error tool results for all five states, error
   results for command failures, and byte-exact agreement with CLI text/JSON.
   There is no typed citation-evidence, citation, or reference tool: keep the
   typed search/get schemas, seven-tool count, catalog names, and inventory
   byte-for-byte and pin that exclusion in the existing catalog tests.
5. Run both ordinary `citations` and `references` through CLI and raw MCP in
   Markdown and JSON with one contextual and one contextless edge. Assert the
   exact edge-local command, its absence on the contextual edge, the unchanged
   root continuation array/footer, and all 1144 ordering, duplicate,
   pagination, malformed-page, and request-log contracts.
6. Use leading/trailing whitespace around caller IDs and otherwise valid
   DOI-shaped suffixes that independently contain quote, backslash, dollar,
   backtick, semicolon, and ampersand. Parse every emitted evidence
   command with the real CLI parser, recover the two original arguments,
   execute it against the fixture, and prove request logs contain only the
   expected encoded seed/graph/JATS requests. Assert no marker file or extra
   shell command is created. JATS passage/title/section/marker fixtures also
   contain pipes, backticks, angle brackets, control whitespace, and repeated
   text; JSON preserves normalized plain text while Markdown cannot inject a
   table row, code span, heading, or raw HTML.
7. Request logs prove provider context makes no JATS request by default;
   contextless and forced calls make exactly one; a first-page match never
   fetches `next`; not-found follows only advertised offsets; malformed pages
   emit no evidence; and construction itself performs no hidden request.
8. Update CLI help, `biomcp list article`, user/reference documentation, and
   the executable article spec with the exact five states, best-effort bounds,
   `--fulltext`, and the non-interpretive limitation. Run focused Rust,
   Python, MCP, and mustmatch checks, then `make lint`, `make test`, and
   `make spec`, followed by package inventory and `git diff --check`.

## Ownership, boundaries, and order

`entities/article/graph.rs` owns directed traversal, budgets, outcome assembly,
edge-local evidence discovery, and the shared evidence-command builder.
`transform/article/jats/refs.rs` owns pure reference/marker extraction beside
the existing reference parser. `sources/semantic_scholar.rs` owns the exact
request plan and wire decoding. `render/markdown/article.rs` only renders the
typed result. CLI and raw MCP delegate to those owners; neither reconstructs
states, passages, provenance, or commands.

Do not add a dependency or tracked file. Keep `src/cli/article/dispatch.rs` at
its current 696 lines or smaller and therefore below the absolute 700-line CLI
cap. Keep `src/entities/article/fulltext.rs` exactly 1,727 lines: this ticket
does not modify the broad full-text waterfall. Preserve ticket 1144's exact
1,385-line `src/entities/article/mod.rs` authorized baseline and exact
1,244-line `src/render/markdown/article/tests.rs` authorized baseline; place
new graph/render integration tests in the existing article CLI test modules
and new extractor cases in the existing JATS tests. Do not raise any other
over-threshold source baseline. The source package remains exactly 1,300 paths.

This ticket does not alter ordinary graph edge acquisition, pagination,
provider order/duplicates, context values, totals, continuation construction,
or 1144's provider-relative coverage semantics. It does not bypass paywalls,
parse HTML or PDF evidence, alter the general article full-text cache or
manifest, add a typed MCP tool, summarize passages, interpret citations,
perform fuzzy reference matching, or change the separate citation-sidecar
issue.

Dependency 1144 is real and landed: 1145 reuses its validated graph-page
contract and must retain its continuation behavior. Ticket 1143's opt-in rich
author papers are independent. With 1144 complete, 1145 is the higher-priority
next article task and should land before 1143.

## Review

The initial design established the five outcome names but left traversal,
JATS identity, bounds, nullability, ownership, MCP coverage, and its interaction
with 1144 open. This revision freezes those contracts and awaits independent
design re-review before implementation.
