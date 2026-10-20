---
priority: 2
---

# 2289 — Fail diagnostic jobs when a command fails

Status: OPEN.

Milestone: 0.9.4

Reviews: revision 916c52d, accept

Reviews: revision a79860c, accept

Reviews: revision 1d427b8, accept

Reviews: revision cd0af5e, accept

Reviews: revision d14fbd7, accept

## Outcome

Every BioMCP job script, smoke and diagnostic helper that runs product commands exits nonzero when any product command fails, and prints that command's captured error. A job exit of 0 always means every command it ran passed.

Changes: `scripts/release-smoke.sh`, `tests/test_release_smoke_script.py`, `scripts/geneagent-demo.sh`, `scripts/genegpt-demo.sh`, `tests/test_demo_scripts.py`, `scripts/measure-mcp-tools.py`, `tests/test_measure_mcp_tools.py`, `tools/measure-0-9-3-baseline`, `tests/test_measurement_summary_contract.py`, `tools/check-png-artifact`, `tests/test_png_artifact_contract.py`, `sdlc/records/2289-command-failure-propagation.md`, `CHANGELOG.md`.

- Proof: A deliberately failing command in each fixed helper makes its job exit nonzero and print the error.

## Evidence

Diagnostic job 2236/1792505124008-69036 reported `get protein P15056 structures --json` failing with a panic while the job script exited 0. The release.yml wheel smoke captured the structured JSON error from `cache stats --json` and hid it, so the first rehearsal showed exit 1 with no output (2286 fixed that one step).

Inventory the scripts under `tools/` and `scripts/` and the workflow steps that run product commands and capture output. Fix each to propagate failure and show the captured error. Add no new framework.

## Implementation boundary

Inventory found swallowed MCP helper exits, hidden captured errors in demos and measurement wrappers, and measurement jobs returning success after recorded command failures. Preserve all samples and summaries while returning failure for unexpected command failures or timeouts. Existing explicitly asserted negative cases stay valid. The release.yml shared helper belongs to 2288; report its stderr requirement instead of editing that file here. Reproduce with fake commands through real helper entry points, no providers.

## Reproduction on current main

Before product edits, focused M5 job 2289/1792506036351-91894 on e0d84c429 ran the five owning regression files with fake executables: release smoke, demo scripts, MCP measurement, measurement summary and PNG artifact contracts. Nine cases failed and fifteen passed. Failures demonstrate hidden command streams, ignored MCP helper failure, coverage status 100 returning 0, runtime exit 7 returning 0 and census exit 42 returning 0. The earlier attempt stopped on a regression-test quoting error before product checks; its evidence remains separate. No provider requests or production mutation.

Keep: experiments/2102-biomcp-2258-final-live-replay — retain failure-propagation/2289 raw preimage and corrected job evidence while release admission consumes this batch.

Landing alone: The batch classifier requires scripts/geneagent-demo.sh changes to land separately as a gate/toolchain dependency. Other accepted work lands through its own permitted gate.
