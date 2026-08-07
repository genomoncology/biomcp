---
base: c78e0545a3431484a342820791b6c587eb9a1898
head: 7f8cb2853dff501fdcfa9e263e4208775dba3a16
---
After the CLI command payloads move into per-entity family modules, the next slice is relocating the runtime behavior: helper functions, dispatch handler bodies, and the execution seam (`run()`, `execute()`, `run_outcome_inner()`).

Imported from March ticket 184. The commit range was
recovered after the fact (merge subject matches the ticket name), not written by the factory at
landing time.

Artifacts: /home/ian/workspace/planning/biomcp/artifacts/184-relocate-cli-runtime-and-outcome-into-family-modules
