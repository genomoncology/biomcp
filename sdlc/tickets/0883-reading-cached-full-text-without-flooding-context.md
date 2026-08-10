---
flow: build
priority: 6
---
# Read bounded ranges from cached article full text

## Command contract

Extend the existing fulltext section with:

    biomcp get article <id> fulltext --outline
    biomcp get article <id> fulltext --lines 210:340

--outline and --lines are mutually exclusive. A line range is inclusive,
one-based, ordered, and limited to at most 500 lines. Invalid or oversized
ranges fail before reading output. The same controls work in JSON.

## Done when

Default fulltext output remains constant-size and adds only byte size, total
line count, and section count beside the cached result. --outline returns
bounded heading records with stable ordinal and line ranges. --lines returns
only the requested range plus total lines and returned range, so a caller can
continue without reading the entire paper or using a separate filesystem
tool.

The command may fetch and cache full text when absent, as it does today. Once
cached, range requests must not refetch the provider.

## Boundaries

- Heading detection supports duplicate headings by ordinal and line range;
  no heading is used as an ambiguous lookup key.
- JSON and Markdown have the same range semantics.
- MCP output retains the existing local-path redaction policy.
- Default output does not grow with document length.
- Searching, summarizing, and semantic section selection are out of scope.

## Proof required

Use a local full-text fixture with duplicate headings and a section over 500
lines. Pin Clap validation, cache reuse, outline ranges, exact line slices,
oversized rejection, JSON/Markdown rendering, and constant-size default
output.

## Authorized test changes

Design commits may restate ArticleGetArgs parsing/help tests, full-text cache
fixtures, entity retrieval tests, renderer tests, docs, and schemas/examples
that describe the fulltext section.

The src line ceiling may rise by at most 220 lines.
