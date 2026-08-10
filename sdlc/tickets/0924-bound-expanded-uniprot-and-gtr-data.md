---
flow: build
priority: 8
deps: ["0915"]
---
# Bound expanded UniProt and GTR data

UniProt caps compressed response bytes and then gzip-decompresses with an
unbounded `read_to_end`; a tiny high-compression body can force a large
allocation. GTR streams a gzip file into an in-memory record map without an
explicit expanded-byte or record budget.

The review measured the 2026-08-09 GTR files at about 13.1 MiB compressed,
154,203,764 bytes expanded with 224,140 test-version rows, and 518,695
condition-gene rows. The limits below leave substantial headroom without
allowing an unbounded input.

## Resource contract

- UniProt permits at most 32 MiB of expanded bytes.
- GTR `test_version.gz` permits at most 512 MiB expanded and 1,000,000 data
  rows.
- GTR `test_condition_gene.txt` permits at most 1,000,000 data rows within its
  existing wire-byte cap.
- Every byte check reads at most limit plus one. Every row check stops at the
  first excess row. Errors are typed, source-attributed, and do not retain the
  oversized buffer.

These are hard safety limits, not output limits. A future upstream dataset that
legitimately exceeds one requires a measured ticket to raise it; the runtime
must not silently clamp or partially index the file.

## Done when

The decoder/resource-budget seams accept test-only limit values while the
production call sites always pass the constants above. Routine tests use small
limits and small local high-compression fixtures to cover exact limit and limit
plus one; they must not construct 32 MiB, 512 MiB, or million-row fixtures. A
static/constant contract pins the production values so a small test limit
cannot leak into production. An instrumented reader proves rejection after at
most configured limit plus one expanded byte or row, rather than after
consuming the complete payload. Existing recorded UniProt and GTR fixtures
parse byte-for-byte as before. A rejected GTR refresh leaves the last complete
local bundle usable and never promotes a partial index.

## Authorized test changes

Design commits may restate and extend decoder, refresh, and body-limit tests in
`src/sources/uniprot.rs`, `src/sources/gtr.rs`, and the relevant quality-ratchet
body-limit fixture. Existing wire caps, atomic GTR bundle promotion, and normal
decoder assertions remain.

The src line ceiling may rise by at most 150 lines.
