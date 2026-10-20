---
priority: 2
---

# 2289 — Fail diagnostic jobs when a command fails

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every BioMCP job script, smoke and diagnostic helper that runs product commands exits nonzero when any product command fails, and prints that command's captured error. A job exit of 0 always means every command it ran passed.

Changes: `scripts/release-smoke.sh`, `tests/test_release_smoke_script.py`, `scripts/geneagent-demo.sh`, `scripts/genegpt-demo.sh`, `tests/test_demo_scripts.py`, `scripts/measure-mcp-tools.py`, `tests/test_measure_mcp_tools.py`, `tools/measure-0-9-3-baseline`, `tests/test_measurement_summary_contract.py`, `tools/check-png-artifact`, `CHANGELOG.md`.

- Proof: A deliberately failing command in each fixed helper makes its job exit nonzero and print the error.

## Evidence

Diagnostic job 2236/1792505124008-69036 reported `get protein P15056 structures --json` failing with a panic while the job script exited 0. The release.yml wheel smoke captured the structured JSON error from `cache stats --json` and hid it, so the first rehearsal showed exit 1 with no output (2286 fixed that one step).

Inventory the scripts under `tools/` and `scripts/` and the workflow steps that run product commands and capture output. Fix each to propagate failure and show the captured error. Add no new framework.

## Implementation boundary

Inventory found swallowed MCP helper exits, hidden captured errors in demos and measurement wrappers, and measurement jobs returning success after recorded command failures. Preserve all samples and summaries while returning failure for unexpected command failures or timeouts. Existing explicitly asserted negative cases stay valid. The release.yml shared helper belongs to 2288; report its stderr requirement instead of editing that file here. Reproduce with fake commands through real helper entry points, no providers.
