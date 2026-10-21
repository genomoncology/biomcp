---
flow: build
priority: 10
---
# Restore truthful release results and private artifacts

The 2026-08-13 post-backlog review found four release-facing contracts that
look correct in focused tests but fail at the real command boundary. Repair
them together before another release binary is accepted.

GWAS Catalog association rows can carry a nonzero p-value that underflows its
lossy `f64` field to `0.0`, alongside an exact mantissa and exponent. BioMCP
must preserve and render the exact value supplied by the provider. It must
never report an impossible zero for a nonzero association, and filtering and
ordering must use semantics consistent with the exact value. Compatibility is
worth preserving only where it does not publish a false value.

A partially failed `batch` is a completed structured report, not an ordinary
command error. JSON and Markdown reports go to stdout, the process exits
nonzero when any item failed, and stderr contains no `Error:` wrapper around
the report. MCP execution must retain its existing text-only boundary and must
not silently discard the partial report.

The explicit `discover ERBB1` acceptance case from 0959 must work without
depending on OLS4 returning one particular HGNC row in its current top search
window. A recognized gene alias resolves or recommends canonical EGFR ahead of
weak UMLS and ontology matches through a bounded typed identity lookup. Do not
special-case ERBB1 alone and do not solve this by requesting an unbounded
ontology window.

Article fulltext downloads are managed BioMCP state. Newly written files and
existing files repaired during normal managed-tree maintenance must have the
same private-file protection as other cache state: mode `0600` on Unix and the
repository's established equivalent on Windows. Atomic replacement must not
briefly expose a broader mode.

## Done when

- A recorded GWAS response containing `p_value: 0.0` plus a nonzero mantissa
  and exponent produces a truthful exact value in JSON and Markdown, and exact
  threshold cases prove filtering does not depend on the underflowed zero.
- A mixed-success batch emits its complete parseable report on stdout, exits
  nonzero, and leaves stderr empty in both JSON and Markdown modes.
- Deterministic fixtures in which OLS4 omits the HGNC result still put the
  canonical EGFR identity or recommendation ahead of weak `ERBB1` concepts,
  with bounded provider work and no public network access in routine tests.
- New and pre-existing fulltext downloads are proven private, including the
  temporary-file and atomic-replacement paths.
- The focused gates and a clean full-feature release build pass. That release
  binary identifies the exact clean commit rather than a `.dirty` source tree.

## Authorized test changes

Design may restate the GWAS parsing and filtering assertions in
`src/sources/gwas/tests/parsing.rs`, `src/entities/variant/gwas/tests.rs`, and
`src/cli/gwas/tests.rs`; the mixed settlement assertions in
`src/cli/system/dispatch.rs` and CLI-boundary assertions needed around
`src/cli/outcome.rs`; the ERBB1 fixture and ranking assertions in
`src/entities/discover.rs`; and download permission assertions in
`src/utils/download.rs` and the existing cache privacy tests. It may add a
focused process-level CLI test proving stdout, stderr, and exit status together.
