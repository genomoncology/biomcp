---
flow: build
priority: 29
---

# Bind Markdown test assertions to their rows and blocks

Four Markdown rendering test suites assert values globally over the whole
rendered document, so a value can move to the wrong row, section, or region
and the test still passes. This is the same defect family as the landed 1046
work (tests that assert presence where they mean shape), applied per block:
the fix pattern in all four is to scope each assertion to the row or block
that owns the value.

The four surfaces, each verified against the code on 2026-08-23:

- Article search: ranked-result tests check identifier order but search
  titles, sources, and ranking rationale globally, so a result can show the
  right rank with another paper's metadata.
- Disease: evidence-link and clinical-feature tests search the whole document
  for cells independently, so sources can move between gene, phenotype, and
  model rows and clinical-feature columns can be reordered or split.
- Trial: central contact, eligibility, and site-contact values are searched
  globally in both the unit test and the executable spec's `mustmatch`, so a
  site email can move outside its location row and central and eligibility
  details can move to another section.
- Drug: despite its name, the regional rendering test searches the complete
  document, so US BLA/OpenFDA facts and EU EMA facts can be moved, merged, or
  flattened across regional headings.

## Done when

- Each of the four tests asserts complete rows or blocks within their owning
  section — a value that belongs to one row or region asserted to appear
  there, and, where the issue names it, asserted **not** to appear in the
  other regions it could be confused with.
- The trial executable spec's contact assertions are scoped within section
  boundaries the same way as the unit test.
- The ordering assertions that exist today (article rank order) are retained
  on top of, not instead of, the row-level binding.
- A reviewer judging restated assertions should read this ticket as the
  authorization: the old global `contains` checks are being replaced by
  section-scoped row and block assertions, and the guarantee each old check
  gestured at is preserved by the new one.

Filed from `sdlc/issues/2026-08-23-bind-article-search-markdown-rows.md`,
`sdlc/issues/2026-08-23-bind-disease-markdown-test-rows.md`,
`sdlc/issues/2026-08-23-bind-trial-markdown-contact-blocks.md`, and
`sdlc/issues/2026-08-23-keep-drug-markdown-regions-isolated.md`.
