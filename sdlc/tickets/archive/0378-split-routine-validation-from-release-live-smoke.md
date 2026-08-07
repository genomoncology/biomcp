---
flow: build
priority: 8
---
# Split routine validation from release live smoke

What is IN scope: - `Makefile` spec/check/release-smoke targets - `.march/validation-profiles.toml` - `tests/test_validation_profile_contract.py` - `spec/surface/test_parallel_isolation_contract.py` - `spec/README-timings.md` - Architecture/runbook docs that explicitly describe `spec-only`, `release-gate`, or release/live-smoke behavior - `tools/biomcp-ci` invocation only as needed to support the live-smoke lane

Completed under March on 2026-05-24, as March ticket 378. Imported as history when BioMCP
moved to the sdlc factory; it was never run by this factory.

Work products from the run — design, reviews, dev log, verify:

    /home/ian/workspace/planning/biomcp/artifacts/378-split-routine-validation-from-release-live-smoke
