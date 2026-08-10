---
flow: build
priority: 9
deps: ["0899", "0950"]
---
# Name the genome build wherever human output shows a coordinate

This is slice three of the coordinate-label sequence. It consumes the typed
model from 0950 and 0899 and does not change JSON or coordinate selection.

## Rendering contract

- Variant detail uses: Genomic coordinate (GRCh37): <coordinate>.
- When provenance says the provider default selected the build, append
  provider default inside the same parenthesis.
- Mixed-build search tables have a separate Build column on every row.
- Gene output labels each coordinate individually; one heading may not imply a
  build for several differently built coordinates.
- Unknown/other assembly states render honestly rather than disappearing.
- The human `Requested variant` line renders a readable normalized identity
  phrase for gene/protein inputs; it never prints a serialized JSON object.

Use these phrases consistently across variant detail, variant search, gene
detail/search, normalization-related output, and related/evidence sections.

## Done when

Every human-visible genomic coordinate has its build next to it. JSON from
0950/0899 and all coordinate values remain unchanged. Default output growth is
limited to the label or one compact table column.

Renderer unit tests consume model fixtures. They do not each require a new
provider receipt; the model-producing tickets own real captures. Include
empty, mixed-build, provider-default, unknown, genomic, gene, and protein
requested-identity cases.

## Authorized test changes

Design commits may restate variant/gene/evidence/related/section Markdown
tests, snapshots, docs, and examples that currently show an unlabeled
coordinate. If a renderer test file not known today constructs an affected
model, it may be updated mechanically without changing unrelated assertions.

The src line ceiling may rise by at most 70 lines.
