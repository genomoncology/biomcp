# PyPI wheel binary stack-overflows on the trial search path

Filed 2026-09-17 after the 0.9.0 release.

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

## Root cause hypothesis

Ticket 1191 fixed the execute-thread stack margin with `Box::pin` at
`src/cli/outcome.rs:647`, bounding the recursive future at 8 MiB. The
maturin-built binary may have a different default thread stack size or
optimization profile that reduces the effective margin. The biodata 1.0
branch carries a 16 MiB `EXECUTE_STACK_BYTES` stopgap (ticket 1191 records
this) which has not landed on main.

## Fix direction

1. Reproduce locally: `maturin build --release && uv run --no-sync biomcp
   search trial --criteria "anti-PD-1 therapy"` to confirm the wheel binary
   hits the same overflow.
2. Raise the execute-thread stack to the same 16 MiB the biodata branch
   uses, or increase the Box::pin margin, whichever is smaller.
3. Add a CI job that installs the built wheel and runs the trial search
   path as a smoke check, so wheel-specific binary regressions are caught
   before publish.

## Priority

Blocking for 0.9.1 — the PyPI install path is the primary distribution
channel for Python users.
