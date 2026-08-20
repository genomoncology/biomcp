---
flow: quickfix
priority: 8
---
# Separate authors from affiliations in article indexing

`get article <id> indexing` renders author names and their affiliations as sibling bullets with nothing distinguishing them, so the output reads as a list of authors where every second entry is an institution:

```
- Keith T Flaherty
- Massachusetts General Hospital Cancer Center, Boston, USA. kflaherty@partners.org
```

A reader cannot tell how many authors a paper has, and anything parsing the list gets institutions mixed in with people. The underlying data already distinguishes the two — the affiliation is attached to the author — so this is a rendering fault rather than a missing field.

## Done when

- An author and that author's affiliation are visually distinguishable in the rendered output.
- The number of authors can be counted correctly from the output.
- An author with no affiliation, and an author with several, both render sensibly.
- The typed JSON form keeps the association between an author and their affiliations.

## Where this lives

The sibling bullet is written in `src/render/markdown/author.rs`, which emits `- {affiliation}` at the same list level as the author name. Restatement is authorized in these files, for these tests by name, only to the extent they assert the current flat bullet shape:

- `src/render/markdown/author.rs` — `detail_markdown_keeps_provider_identity_and_uncertainty_visible`
- `src/render/markdown/article/tests.rs` — `article_markdown_renders_semantic_scholar_and_indexing_sections`

No other test file is authorized.
