---
flow: quickfix
priority: 3
hold: draft for review; do not promote until Ian releases this
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
