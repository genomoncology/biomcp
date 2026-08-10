---
flow: build
priority: 10
deps: ["0951"]
---
# Label the genome build on variant detail JSON

This is a fresh, complete first slice. The prior exhausted queue row is
archived as superseded; its legacy state had no content identity, so editing it
could not safely create a new run. The archive carries that queue rationale.

This is slice one of the coordinate-label sequence:

- 0950: variant detail JSON;
- 0899: search, gene, and normalization JSON;
- 0900: human rendering;
- 0690: only then change the bare-coordinate preference.

## Done when

Every genomic coordinate emitted by `get variant` has an answering build in
the same JSON object:

- rsID detail;
- gene plus protein detail, such as BRAF V600E;
- unqualified transcript-HGVS detail; and
- an explicitly qualified coordinate.

For current MyVariant-backed routes, the label is the actual assembly queried,
including the provider's GRCh37 default where that is the answer. Coordinate,
rsID, HGVS, and other biomedical values remain byte-identical to main.

## Proof required

- Design authors a serialized JSON assertion that is red because the field is
  absent; it need not refer to a Rust field that does not exist yet.
- One real receipted response anchors each distinct production route. Synthetic
  fixtures may cover malformed/error edges and are labeled.
- RequestPlan and local executor proof bind the answering assembly to the
  response.
- Entity and CLI tests prove no coordinate can serialize without its build.
- A before/after fixture comparison proves values did not move.

## Construction-only test edits

Adding a required field may break exhaustive Rust literals. The code commit is
explicitly authorized to update any such constructor as a mechanical compile
fix, including test constructors not known when this ticket was written.
Values and assertions must remain unchanged. Do not derive `Default` for Gene
or other domain entities and do not invent empty biomedical identities.

Design commits may restate variant detail JSON tests, schemas/examples, and
captures that currently assert the unlabeled shape. Changing which build wins
is out of scope.

The src line ceiling may rise by at most 60 lines.
