---
flow: build
priority: 3
---
# Stop dropping complex tables out of article full text

## Done when

`biomcp get article 30311380 fulltext` yields the cell text of all six
tables rather than six omission notices. Merged-cell structure may be
reported rather than reproduced, but no content is discarded.

## The finding

Raised as `sdlc/issues/feature-full-text-tables-are-dropped.md`; that file is deleted when this
lands. The text below is the issue as filed.

article was fetched.

`biomcp get article 30311380 fulltext` renders the body and then, in
place of every table, writes:

    **Table 1:.** Summary of Gene-Specific Criteria for PTEN Variant Classification

    *[complex table omitted: 42×4, merged cells]*

Six tables in that one article, all omitted. Table 1 is the entire
criteria summary — the reason a reader opens this paper. Table 2 is
a phenotype scoring sheet whose eleven rows are a scoring function
someone is going to implement. Both had to be recovered from
elsewhere.

The omission notice is honest, which is worth keeping. But it is a
dead end: nothing in the output says how to get the table, and there
is no verb that will.

## Why this is hard, and why partial is still worth it

Merged cells are genuinely awkward — a `42×4` JATS table with
`rowspan` on the criterion column does not become a clean markdown
grid. That is presumably why the guard exists. But the current
behaviour trades all of the content for all of the fidelity, and the
content is what the reader wants.

Ranked, cheapest first:

1. **A raw row dump.** Emit the cell text row by row, marked as
   unstructured, with the merge information stated rather than
   applied. Ugly and complete beats absent. Even a reader who has to
   re-derive the layout has the numbers.
2. **A table verb.** `biomcp get article <id> table 1` returning one
   table, so the cost of an awkward render is paid only when asked
   for. Fits the existing sub-verb pattern (`fulltext`,
   `annotations`, `assets`) and keeps the default output small.
3. **`--json` structured cells.** Rows and cells with their spans,
   letting a caller lay it out themselves. Most work, most useful
   for anything programmatic.

(1) alone would have closed the gap in the case that raised this.

## Related

Supplementary files are a separate route to the same content and
they have their own problems — see
`article-asset-download-returns-the-ncbi-interstitial-page.md`. When
a table is omitted *and* the supplementary download returns a
placeholder, an article that is fully open access still yields
nothing usable.

Raised 2026-08-08 from PTEN GN003 research for varclassify2.
