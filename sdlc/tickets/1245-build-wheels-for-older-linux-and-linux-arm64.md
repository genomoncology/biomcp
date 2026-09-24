# Build wheels for older Linux and Linux ARM64

Split from ticket 1238. Source issue:
`sdlc/issues/2026-09-23-pypi-wheels-miss-older-linux-and-linux-arm64.md`.

## Problem

PyPI serves only a `manylinux_2_39_x86_64` wheel and no source
package, so `pip install biomcp-cli` fails on Ubuntu 22.04, Debian 12,
and RHEL 8 and 9, and Linux ARM64 has no wheel at all. The code is pure
Rust with rustls and checked-in generated protobuf code, and the ARM64
tarball already builds.

## Design

The first design review rejected a tag-only change: the v0.9.0 Linux
binaries import GLIBC_2.39 symbols (proven by the 1219 container
failure), so `--manylinux 2_28` on an ubuntu-24.04 runner would either
fail the audit or ship a wheel tagged 2_28 that dies on RHEL 8 — the
false floor the retired staged pipeline guarded against with a readelf
check. The floor must be true by construction:

1. Build the Linux wheels inside the official
   `quay.io/pypa/manylinux_2_28_{x86_64,aarch64}` containers: rustup
   1.93.1 and the pinned maturin 1.9.6 installed inside, `--locked`
   build against the native target, output to `target/wheels`. The
   x86_64 leg runs on `ubuntu-24.04`; the aarch64 leg runs on
   `ubuntu-24.04-arm` (availability assumption: this is a public
   repository, so GitHub's free ARM runners apply; the runbook records
   the assumption). No cross toolchain and no protoc — `build.rs` is
   empty and the generated protobuf code is checked in.
2. Enforce the floor mechanically in the same container step: after the
   build, scan every ELF file in the wheel for `GLIBC_x.y` symbol
   references and fail if any exceeds 2.28.
3. Prove the floor at runtime in `wheel-smoke`: the Linux legs install
   the wheel and run `biomcp --version` inside the matching 2_28
   container, in addition to the native-runner deep-path smoke.
4. `wheel-smoke` gains the `ubuntu-24.04-arm` leg (native runner,
   glibc 2.39, deep paths).
5. Docs: the runbook's four-wheel sentence becomes five platforms and
   gains the floor statement — manylinux_2_28 covers RHEL 8, Debian 10,
   and Ubuntu 20.04 onward; older Linux installs from the GitHub
   tarball or `cargo install`. Changelog entry under Unreleased.
6. The sdist stays out on purpose: a pip sdist install would require a
   user-side Rust toolchain, and the tarball and cargo already serve
   the pre-2.28 fallback. The musl and s390x questions stay out.

Sizing: this is a workflow change with two container steps, a symbol
scan, two matrix legs, a docs sentence, and a changelog line — no
Rust source changes.

## Acceptance

- Release workflow produces `manylinux_2_28_x86_64` and
  `aarch64` wheels; the smoke matrix covers both; the floor is
  documented.

## Review

- Design review: REJECT once (a tag-only floor would be false — the
  binaries import GLIBC_2.39); revised to manylinux containers with a
  mechanical symbol check, re-review pending
- Code review: pending
