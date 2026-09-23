---
base: 833dd148
head: 6f3dffb3
---

Made the process survive a panic and closed the reachable slice panics,
from the issue filed 2026-09-23.

`[profile.release]` now sets `panic = "unwind"`, so the two recovery seams
work in production: the execute-thread join at `src/cli/outcome.rs:616`
maps a panicking worker to a "worker panicked" error result, and
`citation_evidence.rs:547` maps a join failure to `FulltextUnavailable`.
MCP tool dispatch rides the same seam (`src/mcp/shell.rs:360` →
`execute_mcp_cli` → `run_outcome_with_worker_stack`), so the server keeps
its sessions. Four reachable byte-slice panics on external strings are
closed: the ClinGen date slice (`clingen.rs:638` → `get(..10)?`), the
DrugCentral approval-date shape guard before caller slicing
(`transform/drug.rs:247-256`), the Europe PMC PMCID `split_at`
(`europepmc.rs:382-383`), and the benchmark excerpt truncation
(`benchmark/run/execute.rs:312-316`). Ten guarded sites were enumerated
safe to leave, in the ticket.

The durable enforcement against regressing to abort is the Python profile
pin in `tests/test_upstream_planning_analysis_docs.py:1181-1189`, updated
to `unwind`: cargo forces unwind on test targets, so no Rust test can
detect the profile flipping back. The new
`worker_panic_on_execute_thread_surfaces_as_an_error_result` test proves
process survival and runs in both profiles.

Evidence: yellow gate at 6f3dffb3 — `make lint` OK, `make test` OK,
`make spec` OK, focused `cargo nextest run --release -E 'test(worker_panic)'`
OK. Binary-size cost of unwinding, measured with `size` on yellow against
the rust-equivalent d2e180ec baseline (zero diff on src, Cargo.toml,
Cargo.lock): text 31,745,305 → 37,303,136 (+5,557,831), file 32,965,240 →
38,524,288 (+5,559,048, about +17 percent).

Origin note: `panic = "abort"` arrived in `09b789d2` ("Rust 080 (#124)",
2026-02-23) with no recorded rationale, and the 0.9 runbook documents
gate-host SIGABRT flakes under abort. Under unwind, the wheel smoke's exit
134 check goes dormant for panics (a panic now exits 101, which still
fails the smoke's success requirement for the swept commands).

Residuals: the panic test mirrors the seam (same thread name, stack size,
and mapping) rather than calling `run_outcome_with_worker_stack`, which
needs a full `Cli`; an edit breaking the real mapping at `:616` would not
fail it. The DrugCentral guard tightens 10-byte non-YYYY-MM-DD values from
passthrough to `None`, intended.
