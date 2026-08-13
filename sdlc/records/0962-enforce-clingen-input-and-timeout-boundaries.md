---
base: 4693d157
head: c6af99b7
---

ClinGen CSpec now applies an explicit 10-second connect timeout and 30-second
whole-request timeout to manifest and exact-version requests, without retries.
Tests inject small deadlines and prove safe ClinGen-attributed failures for
header and body stalls while retaining redirect and URL-policy enforcement.

ERepo file and standard-input batches now share a 65,536-byte reader. It reads
only one sentinel byte beyond the limit and rejects oversized input with the
structured `input_too_large` error before JSON parsing or transport. Exact,
plus-one, whitespace, multibyte, short-read, and I/O cases are covered.

The complete lint, routine test, executable specification, and all-feature
gates passed as part of the three-ticket ClinGen batch. Across the batch,
production `src/` changed by +58 net lines against the combined +520 ceiling;
CLI-only tests were retained under `tests/unit/cli` rather than counted as
production source.
