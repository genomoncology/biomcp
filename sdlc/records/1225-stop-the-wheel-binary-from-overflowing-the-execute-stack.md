---
base: 22b7808e
head: de21f89e
---

Fixed the 0.9.0 wheel stack overflow by building the wheel in the release
profile and smoking it before PyPI.

Root cause: `.github/workflows/release.yml` called `PyO3/maturin-action@v1`
without `args`, and the action has no default, so the published wheels were
dev-profile builds. The 0.9.0 manylinux wheel binary was 97,338,024 bytes with
a 68,281,008-byte `.text`; the release tarball binary is 32,526,272 bytes with
a 24,055,888-byte `.text`. Unoptimized frames overflowed the fixed 8 MiB
execute stack on the trial search, drug trials, and drug interactions paths.

The `pypi-build` job now passes `args: --release --locked`, its Linux leg is
pinned to `ubuntu-24.04` to match the smoke runner, and a new `wheel-smoke` job
between `pypi-build` and `pypi-publish` installs the built x86_64 wheel into a
venv outside the repo tree and runs the four reported commands, failing on
SIGABRT, on the overflow message, or on any exit at or above 128.
`pypi-publish` needs it. `EXECUTE_STACK_BYTES` stays at 8 MiB per ticket 1191;
no stack constant changed.

Measured on the gate host at e46c22ba: the release wheel's `.text` is
24,429,584 bytes, byte-identical to the same-SHA cargo release binary and
within 1.6% of the tarball baseline, against 61,894,848 bytes for a dev wheel
built at the same SHA. On the release wheel, `search trial --condition
diabetes --limit 1`, `search trial --criteria "anti-PD-1 therapy" --limit 3`,
and `drug trials imatinib` exited 0, and `drug interactions apixaban` exited 1
with a clean DDInter-unavailable error because the gate host has no local
DDInter bundle. On the dev wheel, all four exited 134 with `has overflowed its
stack`.

Gate on yellow at de8ffda8: `make lint` OK, `make test` OK (3756 Rust tests
passed, Python lane clean), `make spec` OK. After the two review-hardening
commits, `make lint` and the provenance tests (8/8) pass at de21f89e.
`tools/check-zero-coupling.py` passes; the earlier ticket wording that named
downstream branches was removed.

Residual: the `args` plumbing through the floating `maturin-action` tag is not
exercised until the next release, where a mis-plumbed argument would still fail
loudly because the dev-profile wheel aborts at 134 in the smoke. The
`drug interactions` leg reaches the deep path before the source check in source
order, but it passes cleanly without a local bundle, so its crash signal is
weaker than the other three. Panics at exit 101 are not flagged while
`panic = "abort"` holds. The smoke job has no `timeout-minutes`, matching the
workflow's existing convention.
