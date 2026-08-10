---
flow: build
priority: 8
deps: [0898, 0689]
---
# Label the genome build on gene and search output

Slice two of three, split out of 0689 on 2026-08-10 after that ticket was
refused eight times for being three tickets in one coat.

- 0689 — `get variant` detail routes, JSON. Landed before this.
- **0899 (this one)** — gene and search surfaces, JSON.
- 0900 — rendered/Markdown output.

## Why

`search variant --gene PTEN --json` carries coordinates in its result rows
and **no build field at all** — not null, absent. Search is the more
dangerous surface of the two, because search is how people discover variants
they do not already have coordinates for. They copy a coordinate out of a
result row with nothing telling them which map it came from.

The 2026-08-09 design review read the code and named the exact serializers.
Do not rediscover them:

- `Gene` and `GeneSearchResult` serialize `genomic_coordinates` without a
  build.
- `VariantNormalizationServiceResult` serializes `genomic_descriptions`
  without a build.

## Scope

In scope, `--json` only: every coordinate emitted by `search variant`, by the
`Gene` and `GeneSearchResult` serializers, and by
`VariantNormalizationServiceResult`.

Out of scope: rendered/Markdown output (0900), the variant detail routes
(0689, already landed), and changing which build anything resolves against.

## Done when

- [ ] `search variant --gene PTEN --json` carries a build on every result row
      that has a coordinate.
- [ ] `Gene`, `GeneSearchResult` and `VariantNormalizationServiceResult`
      each carry the build alongside the coordinate they serialize.
- [ ] Where a route's build is fixed by the provider default rather than
      chosen by the caller, the output says so explicitly. No route silently
      omits the field; if a route genuinely cannot know the build, it says so
      rather than emitting nothing.
- [ ] Every assertion is backed by a receipted capture, not a synthesized
      fixture.
- [ ] Coordinate values are unchanged from `main`. Prove it with a
      before/after comparison on the PTEN search and one gene lookup.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Tests you are authorized to restate

    src/entities/variant/search/tests.rs
    src/render/markdown/gene/tests/extended.rs
    src/render/markdown/gene/tests/rendering.rs
    src/cli/tests/next_commands_json_property/gene_article.rs
    src/sources/disgenet/tests/mod.rs
    skills/examples/get-gene-BRAF.json
    skills/schemas/gene.json

Change only what the new label requires; every other assertion keeps checking
what it checks today. Needing a file outside this list means the ticket is
wrong — say so in the design instead of editing it.

## Dependencies

0898 makes these structs constructible without naming every field. 0689
establishes the labeling shape on the detail routes; this slice follows it
rather than inventing a second convention.

The src line ceiling may rise by at most 60 lines.
