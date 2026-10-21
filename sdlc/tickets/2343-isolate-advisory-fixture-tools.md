# 2343 — Isolate advisory fixture tools

Status: OPEN.

Milestone: 0.9.4

## Outcome

Synthetic advisory-gate tests always invoke their own fake toolchain, including when the caller inherits BIOMCP_TOOL_DIR pointing at an installed real cargo-deny. Isolate the fixture environment at its shared construction point. Preserve the production bootstrap, license and advisory policy.

- Changes: `tests/test_lint.py`, `tests/test_dependency_advisory_gate.py`, `CHANGELOG.md`, `sdlc/tickets/2343-isolate-advisory-fixture-tools.md`.
- Proof: reproduce current-main failures under an inherited real or canary tool cache, then pass the entire affected module in that same hostile environment. Assertions still prove the expected fake tool executions and advisory/license outcomes. Fresh review checks both direct subprocess paths and test isolation.

## Evidence

- Coordinator restatement: keep the synthetic gate independent of installed host tools by owning its complete tool-selection environment. No production scanner behavior or policy changes.
- Bounded diagnosis locates the common environment construction in tests/test_lint.py::_advisory_fixture, used by the failing Make-based cases. Set its tool-directory selector to the fixture-owned repo tools, preserving its existing fake uv and scanner assertions. Do not change the qualification driver or production lint to accommodate fake metadata.
- Fresh Linux qualification 2330/j3192 on published main 0412ebeeb1a09341181b50d5a4bce20d8bb40cdb failed fourteen cases in this module. The approved fresh-clone recipe exports its actual BIOMCP_TOOL_DIR; fixture construction merges os.environ but does not replace that variable, so bin/lint prepends the real cache ahead of fake tools. Real cargo-deny then rejects intentionally synthetic cargo metadata missing package name, and expected fake invocation logs are absent. Both direct subprocess paths share this cause.
- The same job passed 4294 Rust cases, selected no explicit long Rust cases, passed the packaged-crate cold offline compile, and passed 1982 Python cases with twelve skips. Strict documentation, optimized full-feature checks and release specifications were not reached. Source guards and clean-tree checks passed. Yellow retains raw job.log SHA256 d2c439aa6cc0da29a691f2cee23c1acd9d7963ad8ec20891c4e24ba4a3e6868d under release-evidence/0.9.4/qualification-2330/j3192. Reuse or resumption requires explicit source/input bindings and PM disposition; this failed run is not a qualification pass.
