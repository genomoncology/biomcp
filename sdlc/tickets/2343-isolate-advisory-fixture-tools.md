# 2343 — Isolate advisory fixture tools

Status: COMPLETE.

Milestone: 0.9.4

Reviews: revision f2f64645bac316a87c4ff77288875020611c4899, accept

## Outcome

Synthetic advisory-gate tests always invoke their own fake toolchain, including when the caller inherits BIOMCP_TOOL_DIR pointing at an installed real cargo-deny. Isolate the fixture environment at its shared construction point. Preserve the production bootstrap, license and advisory policy.

- Changes: `tests/test_lint.py`, `tests/test_dependency_advisory_gate.py`, `sdlc/tickets/2343-isolate-advisory-fixture-tools.md`.
- Proof: reproduce current-main failures under an inherited real or canary tool cache, then pass the entire affected module in that same hostile environment. Assertions still prove the expected fake tool executions and advisory/license outcomes. Fresh review checks both direct subprocess paths and test isolation.

## Evidence

- Published code 57b425ab6b2b4f620f541a9b79064f4779011e0f changes only the two test modules and this ticket relative to qualified main 0412ebeeb1a09341181b50d5a4bce20d8bb40cdb. The shared fixture owns BIOMCP_TOOL_DIR; the added outside-in regression invokes Make with an inherited valid-version hostile scanner and requires the existing fake license, advisory and OSV behavior. Its explicit always advisory scope preserves policy rather than relying on landing's legitimate unchanged-dependency skip.
- Gator j3753 reproduced exactly fourteen failures on unchanged qualified source under the hostile cache. After the shared setting, the existing 76 cases passed; its new regression initially had the wrong landing-scope premise. Final j3764 passed all 77 cases across both complete modules plus Ruff under verified kernel offline isolation. The final two-file payload has SHA256 edac9c7465924768d346013cc5bc49dfff079199897a2df5f6bb70dc92b6bff9. Coordinator independently read the actual terminal receipt and raw result. Earlier j3701 failed only scanner-download preparation with curl exit 28 before any tests ran; it contributes no test result.
- Exact command/source/patch bindings, replay scripts, XML and raw logs remain on Yellow in data/biomcp/ticket-evidence/2343/2343-advisory-fixture-proof.tar.gz, SHA256 cb04014e6a80efc4225d7bb762558c6ec94b0e0aebeedf219e6c107565370be6. PM reply to m3411 permits reuse of passed qualification stages only if the repaired published source differs from 0412 solely in tests, fixtures and SDLC. The entire affected module and every unreached Linux stage must pass on that source. A fresh review and the declared landing gate remain required before publication.

- Coordinator restatement: keep the synthetic gate independent of installed host tools by owning its complete tool-selection environment. No production scanner behavior or policy changes.
- Bounded diagnosis locates the common environment construction in tests/test_lint.py::_advisory_fixture, used by the failing Make-based cases. Set its tool-directory selector to the fixture-owned repo tools, preserving its existing fake uv and scanner assertions. Do not change the qualification driver or production lint to accommodate fake metadata.
- Fresh Linux qualification 2330/j3192 on published main 0412ebeeb1a09341181b50d5a4bce20d8bb40cdb failed fourteen cases in this module. The approved fresh-clone recipe exports its actual BIOMCP_TOOL_DIR; fixture construction merges os.environ but does not replace that variable, so bin/lint prepends the real cache ahead of fake tools. Real cargo-deny then rejects intentionally synthetic cargo metadata missing package name, and expected fake invocation logs are absent. Both direct subprocess paths share this cause.
- The same job passed 4294 Rust cases, selected no explicit long Rust cases, passed the packaged-crate cold offline compile, and passed 1982 Python cases with twelve skips. Strict documentation, optimized full-feature checks and release specifications were not reached. Source guards and clean-tree checks passed. Yellow retains raw job.log SHA256 d2c439aa6cc0da29a691f2cee23c1acd9d7963ad8ec20891c4e24ba4a3e6868d under release-evidence/0.9.4/qualification-2330/j3192. Reuse or resumption requires explicit source/input bindings and PM disposition; this failed run is not a qualification pass.
