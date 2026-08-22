---
flow: build
priority: 8
---
# Separate authors from affiliations in article indexing

`get article <id> indexing` renders author names, their affiliations, and the affiliation identifiers as sibling bullets at the same level, so the output reads as one flat list in which only some entries are people:

```
- Ada First (ORCID: 0000-0002-1825-0097)
- Fixture University
- ROR: shared
```

A reader cannot tell how many authors a paper has, and anything parsing the list gets institutions and identifiers mixed in with people. The underlying data already distinguishes the three — an affiliation belongs to an author, an identifier belongs to an affiliation — so this is a rendering fault rather than a missing field.

## Read this before deciding the fault is already fixed

A previous attempt refused this ticket after reading the template source, seeing indented bullets, and concluding the nesting was already there. The indentation in the source does not reach the output. In `templates/article.md.j2` the author loop opens with `-%}`, and minijinja is built here with its defaults, so that marker strips the newline **and the leading indentation of the line that follows it**. The nested-looking source renders flat.

The block above is real output, captured by rendering the fixture through the shipped code path. Render the template before judging the shape; do not read indentation off the source.

## Why this is a build and not a quickfix

The suite is green today and will stay green until someone authors a proof. The one test covering this output asserts only that the affiliation text appears somewhere in the rendered string, which is equally true flat or nested. There is no failing assertion to reproduce, so the proof has to be written rather than observed.

## Done when

- An author, that author's affiliation, and that affiliation's identifiers are each distinguishable in the rendered output, and the nesting shows which belongs to which.
- The number of authors can be counted correctly from the output.
- An author with no affiliation, and an author with several, both render sensibly.
- A proof pins the **shape** of the rendered output, not merely the presence of the text — a substring check that passes flat is not a proof.
- The typed JSON form keeps the association between an author and their affiliations.

## Where this lives

`templates/article.md.j2`, in the `### Authors` block of the Article Indexing section. The whitespace-control markers on the loop tags are the fault; the surrounding Rust does not need to change to fix the shape.

`src/render/markdown/author.rs` is **not** involved — that renders the separate `get author` command, whose flat `## Affiliations` list is correct as it stands. An earlier version of this ticket pointed there by mistake.
