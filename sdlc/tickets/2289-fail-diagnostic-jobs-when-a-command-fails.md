---
priority: 2
---

# 2289 — Fail diagnostic jobs when a command fails

Status: OPEN.

Milestone: 0.9.4

## Outcome

Every BioMCP job script, smoke and diagnostic helper that runs product commands exits nonzero when any product command fails, and prints that command's captured error. A job exit of 0 always means every command it ran passed.

- Proof: A deliberately failing command in each fixed helper makes its job exit nonzero and print the error.

## Evidence

Diagnostic job 2236/1792505124008-69036 reported `get protein P15056 structures --json` failing with a panic while the job script exited 0. The release.yml wheel smoke captured the structured JSON error from `cache stats --json` and hid it, so the first rehearsal showed exit 1 with no output (2286 fixed that one step).

Inventory the scripts under `tools/` and `scripts/` and the workflow steps that run product commands and capture output. Fix each to propagate failure and show the captured error. Add no new framework.
