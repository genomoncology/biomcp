---
flow: build
priority: 7
deps: [0689, 0899]
---
# Name the genome build in rendered output

Slice three of three, split out of 0689 on 2026-08-10.

- 0689 — `get variant` detail routes, JSON. Landed.
- 0899 — gene and search surfaces, JSON. Landed.
- **0900 (this one)** — the rendered, non-JSON output.

## Why

0689 and 0899 label the coordinate in `--json`. A field that appears only in
JSON leaves the human-facing path unlabeled, and the human-facing path is
where a coordinate gets read by eye and pasted somewhere else. Someone
reading rendered output should not have to re-run the command with `--json`
to learn which map the number came from.

## Scope

Every rendered surface that displays a coordinate: variant detail, variant
search results, and gene output.

Out of scope: JSON (done in 0689 and 0899), and changing which build any
route resolves against.

## Done when

- [ ] Rendered output names the build wherever it shows a coordinate.
- [ ] The label is meaningful to a reader — the build named next to the
      coordinate it applies to, not a footnote or a bare code far from the
      value.
- [ ] Where the build is the provider default rather than a caller choice,
      the rendered output says so, in words a reader can act on.
- [ ] The JSON labels from 0689 and 0899 are unchanged. This ticket adds a
      rendered label and moves nothing else.
- [ ] Every assertion is backed by a receipted capture.
- [ ] `make lint`, `make test`, and `make spec` pass.

## Tests you are authorized to restate

    src/render/markdown/variant/tests.rs
    src/render/markdown/gene/tests/rendering.rs
    src/render/markdown/gene/tests/extended.rs
    src/render/markdown/evidence/tests.rs
    src/render/markdown/related/tests/gene_drug.rs
    src/render/markdown/root_tests.rs
    src/render/markdown/sections/tests.rs

Change only what the rendered label requires. Needing a file outside this
list means the ticket is wrong — say so in the design.

## Note on wording

The 2026-08-09 review required a "meaningful Markdown label" rather than an
inline code fragment. Decide the exact phrasing in the design and apply it
consistently across all three surfaces; do not let variant detail and search
results describe the same fact two different ways.

The src line ceiling may rise by at most 50 lines.
