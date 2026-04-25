# Code Review Log

## Critique

- Read the full `main..HEAD` diff plus the live worktree delta in `src/entities/discover.rs`.
- Design completeness and forward traceability matched the final proof matrix for the
  three surface specs, active-corpus docs, and docs-contract test updates.
- Reverse traceability found one review-surface mismatch: `spec/surface/mcp.md`
  bypassed `tools/biomcp-ci` for the stdio help block even though the active-corpus
  docs now state every executable-spec bash block should go through that wrapper.
- The shipped surface corpus still exposed the known discover runtime gap until the
  worktree repair: symptom-first HPO-backed phrases such as `developmental delay`
  could still surface disease-heavy OLS hits without a phenotype-first follow-up.
- Edit-discipline audit stayed within scope: the main branch diff is the named
  docs/spec surface, and the review repairs stayed on the discover runtime path and
  the MCP surface-spec wrapper path only.

## Fixes Applied

1. Kept the minimal discover runtime repair that supplements disease-heavy OLS
   results with one HPO concept when the query is symptom-first and OLS misses the
   phenotype bridge, plus a bounded unit test for the `HP:0001263` follow-up path.
2. Switched `spec/surface/mcp.md` stdio-help coverage from direct `"$BIOMCP_BIN"`
   calls to `../../tools/biomcp-ci` so the spec matches the documented active-corpus
   wrapper contract.

## Residual Concerns

- None.

## Defect Register

| # | Category | Lintable | Description |
|---|----------|----------|-------------|
| 1 | validation-gap | no | `discover "developmental delay"` still needed the HPO-backed phenotype bridge in runtime code so symptom-first phrases surface `biomcp search phenotype "HP:0001263"` before broader disease search. |
| 2 | stale-doc | yes | `spec/surface/mcp.md` used `"$BIOMCP_BIN"` directly for stdio help even though the active-corpus docs now require executable-spec bash blocks to run through `tools/biomcp-ci`. |
