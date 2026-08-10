---
flow: build
priority: 9
deps: ["0689"]
---
# Label genome builds on gene, search, and normalization JSON

This is slice two of the coordinate-label sequence. It follows 0689's detail
shape and precedes 0900's rendering.

## Model contract

A genomic coordinate and its build are one typed value:

- where a model has multiple coordinates, use coordinate objects containing
  coordinate, genome_build, and source/provenance; never parallel arrays;
- where a result has one coordinate, keep the build adjacent in the same
  result object;
- every emitted genomic coordinate has GRCh37, GRCh38, another explicit
  assembly identifier, or an explicit unknown state. No raw unlabeled
  coordinate remains.

Apply this to variant search rows, Gene, GeneSearchResult, and
VariantNormalizationServiceResult. Preserve coordinate values.

## Done when

- search variant --gene PTEN --json labels every coordinate-bearing row;
- gene detail/search JSON labels each genomic coordinate;
- normalization JSON labels each genomic description or explicitly marks its
  assembly unknown;
- provider-default builds are recorded as provenance rather than inferred by
  the renderer;
- schemas/examples and MCP serialization carry the same typed shape.

## Proof required

One real receipted anchor per distinct provider route, production RequestPlan
and decoder/orchestration proof, local CLI process tests, before/after
coordinate-value comparison, and synthetic unknown/error edges.

## Construction-only test edits

The code commit may update any exhaustive Rust constructor broken by the new
required field as a mechanical compile fix. Values/assertions remain
unchanged. Do not derive Default for biomedical domain entities. Design
commits may restate JSON/schema expectations for the new typed shape.

The src line ceiling may rise by at most 140 lines.
