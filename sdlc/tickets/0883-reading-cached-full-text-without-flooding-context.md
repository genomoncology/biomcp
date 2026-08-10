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
one-based, ordered, and limited to at most 500 lines and 65,536 returned UTF-8
bytes. Invalid or oversized ranges fail before reading output. When complete
lines would cross the byte limit, stop before the next line and return
`truncated: true` plus `next_line` naming that first unreturned line. A single
line larger than 65,536 bytes fails with a typed line-too-large error rather
than being split or emitted. The same controls work in JSON.

## Done when

Default fulltext output remains constant-size and adds only byte size, total
line count, and section count beside the cached result. --outline returns at
most 200 heading records with stable ordinal and line ranges. A heading title
is limited to 512 UTF-8 bytes; a longer title is shortened at a character
boundary and carries `title_truncated: true`. The outline reports `returned`,
exact `total`, and `has_more`. --lines returns only the requested range plus
total lines, returned range, returned bytes, `truncated`, and `next_line`, so a
caller can continue without reading the entire paper or using a separate
filesystem tool.

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

Use a local full-text fixture with duplicate headings, more than 200 headings,
a heading over 512 bytes, a section over 500 lines, one line over 65,536 bytes,
and a range whose next complete line crosses 65,536 bytes. Pin Clap validation,
cache reuse, outline ranges, exact line slices, byte-bound continuation,
oversized-line rejection, JSON/Markdown rendering, and constant-size default
output.

## Authorized test changes

Design commits may restate ArticleGetArgs parsing/help tests, full-text cache
fixtures, entity retrieval tests, renderer tests, docs, and schemas/examples
that describe the fulltext section.

The src line ceiling may rise by at most 220 lines.
