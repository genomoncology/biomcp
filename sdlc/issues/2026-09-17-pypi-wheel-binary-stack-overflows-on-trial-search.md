# PyPI wheel binary stack-overflows on the trial search path

Filed 2026-09-17 after the 0.9.0 release. Root cause confirmed 2026-09-22 and
fixed by ticket 1225 in main commit `de21f89e`.

## Symptom

`uvx --from biomcp-cli==0.9.0 biomcp search trial --criteria "anti-PD-1
therapy" --limit 3` aborts with:

```
thread 'biomcp-cli-execute' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

The same command works correctly on the GitHub Release binary (built with
`cargo build --release`) and on the locally-built release binary on the gate
host. Only the maturin-built PyPI wheel binary hits the overflow.

## Reproduction

```
uvx --from biomcp-cli==0.9.0 biomcp search trial --criteria "anti-PD-1 therapy" --limit 3
```

Non-trial commands (`get gene`, `search article`, `discover --search`) work
from the same wheel, so the overflow is specific to the trial search
execution path.

## Root cause (confirmed, with measurement)

The shipped wheel was a debug-profile build. The release workflow called
`PyO3/maturin-action@v1` with no `args`, and maturin's default profile is
debug, so the binary inside `biomcp_cli-0.9.0-py3-none-manylinux_2_39_x86_64.whl`
was unoptimized: 97,338,024 bytes with a 68,281,008-byte `.text`, against the
release tarball's 32,526,272 bytes and 24,055,888.

The execute thread's fixed 8 MiB stack (`EXECUTE_STACK_BYTES`,
`src/cli/outcome.rs:599`) is the design ticket 1191 chose, with the future
`Box::pin`'d at `:647`. Release-profile frames fit that margin; debug-profile
frames are several times taller, and the deep trial search, drug trials, and
drug interactions paths exceed 8 MiB only in that profile.

Same-SHA proof on the gate host: a release wheel's `.text` measured
24,429,584 bytes, byte-identical to the cargo release binary at the same
commit, and ran the four reported commands cleanly, while a dev wheel at the
same SHA measured 61,894,848 and aborted all four with exit 134. The defect
was the build profile of the shipped artifact, not the stack constant.

## Why it escaped

Every release check exercised a release-profile artifact: the tarballs
(`cargo build --release`), the container image (assembled from the published
tarballs), and the Homebrew formula (installed from the tarballs). The wheel
was the only dev-profile artifact, it had no install-and-run smoke, and
nothing asserted its profile. The stack margin was therefore only ever proven
against binaries that were not the one PyPI served.

## Fix (ticket 1225, main `de21f89e`)

- `args: --release --locked` on the maturin step, so every wheel matches the
  release profile the tarballs use; `--locked` matches CI's lockfile
  discipline.
- A `wheel-smoke` job between `pypi-build` and `pypi-publish` that installs
  the built x86_64 wheel into a venv outside the repo tree and runs the four
  reported commands, failing on SIGABRT, on the overflow string, or on any
  exit at or above 128. `pypi-publish` needs it.
- `EXECUTE_STACK_BYTES` stays at 8 MiB per ticket 1191's ruling against
  raising it to paper over a margin. The escalation rule if a release-profile
  wheel ever overflows again is recorded in ticket 1225: gdb watermark on the
  gate host, a 16 MiB cap, stop and report.

## Other locations of the same class

- Fixed-size stacks in production code: exactly one, the execute thread at
  `src/cli/outcome.rs:603`. All CLI and MCP command execution shares it, so
  the margin analysis covers every command surface.
- Test-only stacks: `src/cli/discover.rs:333` (16 MiB, `#[cfg(test)]`) and
  `src/cli/tests/facade/cache.rs:144` (8 MiB). Neither ships.
- Other artifact paths cannot ship a dev-profile binary: the tarballs are
  release builds, the container image and Homebrew consume the published
  tarballs, and `maturin` appears only in the now-fixed release workflow step.
- Downstream private integration branches carry a 16 MiB stopgap in their
  copies of `outcome.rs` because they stack deeper paths on the same seam;
  main deliberately does not (ticket 1191).
- A locally built debug binary (`cargo build` without `--release`) can still
  overflow on the deepest live-data paths; that is inherent to the profile
  and does not affect any published artifact.

## Priority

Resolved on main; the issue stays open until 0.9.1 ships the fixed wheel.
