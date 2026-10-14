# 2089 — Run search all's two GWAS queries at the same time

Status: OPEN.

Milestone: 0.9.3

## Outcome

With a gene and a disease, the GWAS section of `biomcp search all` and `biomcp search gwas -g GENE --trait TEXT` waits only as long as the slower of its two GWAS Catalog queries, so `search all --gene BRAF --disease melanoma` finishes its GWAS section inside the 12-second section limit, with unchanged output.

## Evidence

- A run-time investigation on the build host (2026-10-14) found that `search all --gene BRAF --disease melanoma --counts-only` spends 10 to 12 s waiting on two GWAS Catalog requests sent one after the other: the gene query took 6.0 to 7.8 s and the trait query 3.6 to 4.9 s over ten tries each; sent together, both finished in 6.5 to 6.8 s. Every other section finished within about 2 s, because `search all` already runs its sections at once.
- The 12-second limit (`SECTION_TIMEOUT` in `src/cli/search_all/dispatch.rs`) cut the GWAS section off in three of the 2069 baseline's warm runs, which all read exactly 12.02 s and printed "gwas search timed out after 12s". Those runs dropped the GWAS rows, so the baseline's warm search-all number is a timeout, not a speed.
- Ian ruled that program speed matters more than build speed.

- Starts from: `search_gwas_page` in `src/entities/variant/gwas.rs`, which awaits `GwasClient::search_associations` for the gene leg and then for the trait leg before `intersect_gwas_legs`; its callers in `src/cli/search_all/dispatch.rs` (the GWAS section) and `src/cli/gwas/dispatch.rs`; the `BIOMCP_GWAS_BASE` override in `src/sources/gwas.rs`, which points the client at a local server in tests; the local `TcpListener` server pattern used in `src/cli/health/tests/http.rs`.
- Keeps: the rows, their order, the pagination fields and `truncated_by_provider_budget`; the 50-row fetch for a combined query; GWAS's no-store requests; the error a failed leg returns today.
- Changes: claims `src/entities/variant/gwas.rs` `src/entities/variant/gwas/tests.rs` `CHANGELOG.md`.
  - Run the two legs with `tokio::try_join!` when both a gene and a trait are given, and keep one call when only one is. A failure in either leg fails the page as it does today.
  - One Fixes changelog bullet: search all and gwas search with a gene and a trait wait for one GWAS query, not two.
- Proof: routine suite. A new test in `src/entities/variant/gwas/tests.rs` starts a local server that answers each GWAS association request after 1.5 s, points `BIOMCP_GWAS_BASE` at it, and calls `search_gwas_page` with a gene and a trait: it finishes in under 2.5 s and returns the same rows as the sequential path on the same replies. It fails on current main (about 3 s). On the build host, the reviewer runs `for i in 1 2 3 4 5; do /usr/bin/time -f %e biomcp search all --gene BRAF --disease melanoma --counts-only --json | jq '.sections[]|select(.entity=="gwas")|.error'; done` and records a median under 8 s with no timeout error.
- Defers: caching GWAS replies (2091).
- Order: runs alongside the build work; it shares no file with 2070 to 2088. Lands before 2091, which edits the GWAS source. Shares `CHANGELOG.md` with every ticket; 2083 removes that conflict.
