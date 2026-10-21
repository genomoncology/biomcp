---
base: 88f8e6ddb937cb5a07852524329a3c14dd01583f
head: ec0003de
---

The unsupported GWAS region filter is gone from Clap, discovery output, and
public documentation, so it cannot silently return an empty success. Gene and
trait searches now use the official bounded v2 association endpoint. A
combined search performs exactly one gene request and one trait request,
normalizes rsIDs, returns their intersection, and applies the p-value filter
afterward.

The implementation caps each leg at 50 decoded candidates, never traverses
additional provider pages, and preserves the existing association mapping and
deduplication. Focused tests cover overlapping and disjoint inputs, the
post-intersection p-value boundary, exact v2 request construction, fixture
decoding, help rejection, and existing variant-detail enrichment. A minimized
real response and capture receipt pin the provider contract.

The combined 0927/0928 implementation added 129 net `src` lines, below the
two tickets' combined 280-line ceiling.
