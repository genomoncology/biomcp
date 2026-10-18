---
priority: 1
---

# 2207 — Repair disabled article source qualification

Status: OPEN.

Milestone: 0.9.3

## Outcome

Disabled article sources remain explicitly skipped, silent and complete in the shipped CLI. Diagnose the current Linux release-qualification failure without hiding genuine provider failures or weakening the disabled-source assertion. Correct the smallest production defect or fixture omission that the actual failure establishes.

- Evidence: full nonpublishing rehearsal 39131401809 on current main `7907ab4b0871922270718db6065a1be7c291431c`, qualification job 117687973512, `make test-rust`, failed `disabled_article_sources_are_skipped_without_warnings_or_incompleteness` at `tests/disable_sources_cli.rs:1014`. Stderr: `WARN biomcp_cli::cli::article::dispatch: Exact article keyword entity lookup unavailable`. Summary: 4281 passed, 1 failed, 34 skipped; 91 tests not run after failure. Current main passed its full native check; this is a release-blocking Linux qualification failure, not an accepted flaky rerun.
- Changes: `tests/disable_sources_cli.rs`, `src/cli/article/dispatch.rs`, `CHANGELOG.md`. Restate the outcome, inspect current main, record the exact focused reproduction and claim any additional minimal file before production edits. Start with the existing real CLI test and fixture; add no general fixture framework.
- Proof: reproduce the named case through a build job, retain the exact warning and output, distinguish missing fixture behavior from source-disable behavior, and make the focused regression pass without dropping assertions. Check the adjacent real provider-failure case still warns. Use occasional Linux build-job reproduction when the host-specific condition requires it; native development gates still run through the normal builder. Obtain fresh review, one ready source push, shared landing gate and post-merge main check. Final full nonpublishing rehearsal must pass on the replacement candidate.

## Progress

- 2026-10-18 started
