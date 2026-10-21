---
flow: build
priority: 8
deps: ["0974"]
---
# Close post-backlog interface and local-gate drift

The 2026-08-13 post-backlog review found a small set of shipped interfaces that
disagree with their implementation or apply their safety boundary only
partially. Close them in one focused cleanup while the affected command,
transport, and gate code is still loaded.

All-region Markdown drug search must expose the same region-selecting
continuation already present in JSON. A caller must be able to reach a later EU,
US, or WHO page without guessing that offsets are per region or unnecessarily
paging the other regions.

Generated GWAS help must not advertise the unsupported `--region` flag. Help,
`list gwas`, and Clap's accepted arguments must agree, and a focused contract
must prevent command examples from naming flags that the command rejects.

The `serve-http` allowed-Host policy applies to the whole HTTP router, including
`/`, `/health`, and `/readyz`, not only `/mcp`. The explicit
`--unsafe-allow-any-host` escape hatch continues to disable the check for every
route, and non-loopback binding rules remain unchanged.

`make lint` is a canonical gate and must not crash with a Python traceback when
its pinned ShellCheck or actionlint executable is absent. On a supported host,
the ordinary target should use the repository's checksum-verified pinned tools
without requiring the developer to construct a private `PATH`. On an
unsupported host it fails clearly with exact installation instructions; it
does not silently report success after skipping a promised gate.

Pre-commit installation remains opt-in. Extend the installer with a read-only
check mode that distinguishes the tracked two-line handoff from a missing or
stale local hook and prints the exact installation command. Do not overwrite a
developer's local hook merely because another gate ran.

## Done when

- Markdown and JSON both give the exact next command for each drug region with
  more results, and that command selects only that region.
- GWAS help contains only accepted flags, with a test that would catch the
  removed `--region` example returning.
- Every HTTP route rejects a disallowed Host and accepts an allowed Host; the
  unsafe override is covered explicitly.
- `make lint` uses the pinned tools on supported clean machines and missing-tool
  failures are concise and actionable rather than tracebacks or silent skips.
- The pre-commit installer can report current, missing, and stale hook states
  without changing them, while its existing install mode still writes the
  tracked handoff.

## Authorized test changes

Design may restate drug-search rendering assertions in
`src/cli/drug/tests.rs`, `tests/unit/cli/drug_json.rs`, and
`src/render/markdown/drug/tests.rs`; GWAS help assertions in
`src/cli/gwas/tests.rs`; HTTP surface assertions in `src/mcp/shell.rs`,
`tests/test_mcp_http_surface.py`, and `tests/test_streamable_http_demo.py`;
lint-tool assertions in `tests/test_shell_workflow_lint_contract.py`; and hook
installation assertions in `tests/test_pre_commit_reject_march_artifacts.py`.
