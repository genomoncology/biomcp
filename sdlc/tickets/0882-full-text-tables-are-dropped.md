---
flow: build
priority: 5
deps: ["0951"]
---
# Preserve complex table cells in saved article full text

## Done when

The saved Markdown produced for PMID 30311380 contains the cell text from all
six complex JATS tables instead of omission notices. Merged-cell structure may
be represented as row/column span annotations rather than a perfect visual
grid, but no cell text is discarded.

The CLI may continue returning only the saved path and bounded metadata. This
ticket changes the saved file, not the default stdout context size.

## Simplest acceptable rendering

Render a clearly labeled raw row sequence:

- retain caption and table identifier;
- emit cells in source row order;
- mark rowspan/colspan when present;
- preserve nested text and links as plain content;
- keep an explicit warning when visual reconstruction is lossy.

Do not build a general table-layout engine when a complete row dump satisfies
the behavior.

## Proof required

- A real receipted JATS capture for PMID 30311380 passes through the production
  parser.
- Parser tests prove every source cell survives with span metadata.
- A saved-file assertion proves the six tables contain expected sentinel cells
  and no complex-table-omitted marker.
- Small ordinary tables render unchanged.
- Malformed tables fail locally or render an honest bounded warning without
  dropping the surrounding article.

## Authorized test changes

Design commits may restate the JATS parser fixtures, article full-text saved
file assertions, and rendering tests. Mechanical construction fixes may land
with implementation while unrelated article output remains unchanged.

The src line ceiling may rise by at most 250 lines.
