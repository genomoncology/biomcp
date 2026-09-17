# BioMCP 0.9.1 / 1.0 backlog proposal

Written 2026-09-17 after the 0.9.0 release, from the session's observed
issues, deferred backlog items, and known gaps. Split into a hotfix track
(0.9.1, ship fast) and a feature track (1.0).

## 0.9.1 hotfix track — blocking distribution issues

### P1: PyPI wheel binary stack-overflows on the trial search path

Filed as `sdlc/issues/2026-09-17-pypi-wheel-binary-stack-overflows-on-trial-search.md`.
The maturin-built binary hits a stack overflow that the cargo-built binary
does not. The primary Python install path (`pip install biomcp-cli` /
`uvx`) is broken for trial searches. Fix the stack margin and add a
wheel-install smoke check to CI.

**Estimated complexity:** Level 2 (one outcome-thread change, one CI job).

### P2: Docker image publication dropped from the release workflow

The 0.8.25 release published `ghcr.io/genomoncology/biomcp` images. The
0.9.0 slimmed workflow dropped the `docker-publish` job to get the release
out. Docker users cannot pull 0.9.0.

**Estimated complexity:** Level 1 (restore the docker-publish job from the
0.8.25 workflow).

### P3: No macOS or Linux ARM64 CI runner

The gencc store and provider capture code had platform-specific type
mismatches (`dev()`, `ino()`, `mode_t`) that only surfaced when the release
workflow built on macOS. The CI workflow only tests Linux x86_64 and
Windows x86_64. Add a macOS runner to CI so platform regressions are caught
before release, not during.

**Estimated complexity:** Level 1 (one CI job addition, one `cargo check`
per platform).

### P4: Release workflow docs reference the abandoned signing pipeline

`docs/reference/release-process.md` and the runbook describe the elaborate
stage/promote workflow with MCPB signing, notarization, and sealed
manifests. The actual release uses the simple build-and-publish workflow
restored from 0.8.25. The docs mislead the next operator.

**Estimated complexity:** Level 1 (rewrite the docs to match the actual
process).

## 1.0 feature track — deferred backlog and new capabilities

### F1: Author bare-name search (deferred ticket E)

`search drug imatinib` works with a bare positional; `search author "Louis
Williams"` demands `--query`. Add the same bare-name compatibility.

**Estimated complexity:** Level 1.

### F2: Sync per-file change detail (deferred ticket F)

The `--json` sync commands report `changed` but not which files. The
fingerprint already computes the delta; emit it.

**Estimated complexity:** Level 1.

### F3: ORCID claim counts in the header (deferred ticket G)

Show "10 of 200 claimed works" in the Markdown header and JSON pagination
the way trials show "N of M results."

**Estimated complexity:** Level 1.

### F4: MCP discover typed tool (deferred ticket H)

The seven-tool catalog lacks a discover tool. Adding an eighth changes the
frozen catalog that multiple tests pin.

**Estimated complexity:** Level 2.

### F5: Drop the biodata/biomcp-1.0 16 MiB stack stopgap

Ticket 1191 records this. When biodata merges their branch with main, the
`EXECUTE_STACK_BYTES` stopgap drops in favor of main's `Box::pin` fix.
Coordinate with the biodata team.

**Estimated complexity:** Level 1 (a deletion, once the merge lands).

### F6: Linux ARM64 PyPI wheel

0.8.25 and 0.9.0 both ship without a manylinux aarch64 wheel. GitHub now
offers ARM64 runners (`ubuntu-24.04-arm`). Add one `pypi-build` job on that
runner to produce a native aarch64 wheel.

**Estimated complexity:** Level 1 (one CI job).

### F7: MCPB bundle for Claude Desktop directory installation

The official MCP Registry expects a `server.json` with binary distribution
metadata. The elaborate workflow was supposed to produce signed MCPB
bundles but was abandoned. If Claude Desktop directory installation is a
goal, design a minimal MCPB path that does not require the full signing
pipeline.

**Estimated complexity:** Level 3 (new artifact type, registry metadata,
distribution contracts). Needs a design review before implementation.

### F8: Benchmark tokenizer blob-diff noise

Filed as `sdlc/issues/2026-09-16-benchmark-tokenizer-blob-diff-noise.md`.
The benchmark contract produces diff noise from tokenizer state blobs.

**Estimated complexity:** Level 1.

### F9: Git pack history size audit

Filed as `sdlc/issues/2026-09-16-git-pack-history-size-audit.md`. The repo
has accumulated large history blobs from the growth audit.

**Estimated complexity:** Level 1 (an audit and cleanup, not a code change).

## What I would not do

- **Do not restore the elaborate signing pipeline.** It was designed but
  never operationalized. The simple workflow publishes correctly to
  GitHub, PyPI, and Homebrew. If code signing becomes a requirement
  (Apple notarization for macOS Gatekeeper, Windows Authenticode), design
  it as an additive step on the simple workflow, not a replacement.
- **Do not add a Linux ARM64 binary to the GitHub Release without a
  matching PyPI wheel.** Users who install from PyPI expect the same
  platform coverage as the tarballs.
- **Do not chase the S2 directed-traversal gap.** It is a provider data
  availability issue, documented in the backlog.

## Priority order

0.9.1 (ship within days):
1. P1 — PyPI stack overflow (blocking the primary install path)
2. P2 — Docker image (distribution gap)
3. P3 — macOS CI (prevents the class of P1-style surprises)
4. P4 — Release docs (operator clarity)

1.0 (ship when ready):
1. F5 — Stack stopgap cleanup (coordinate with biodata)
2. F6 — Linux ARM64 PyPI wheel (distribution completeness)
3. F1-F4 — Deferred quality-of-life tickets
4. F7 — MCPB bundle (if Claude Desktop directory is a goal)
5. F8-F9 — Housekeeping
