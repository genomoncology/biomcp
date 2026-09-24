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

1. Set `manylinux: 2_28` on the Linux wheel builds (glibc 2.28 covers
   RHEL 8, Debian 10, and Ubuntu 20.04 onward).
2. Add `ubuntu-24.04-arm` with `aarch64-unknown-linux-gnu` to the
   `pypi-build` matrix.
3. Add a matching wheel-smoke leg to the release checks.
4. Record the compatibility floor honestly in the docs and the
   changelog; older-than-2.28 Linux users install from the tarball or
   cargo.

This stays hygiene-sized: matrix plus profile plus a smoke leg. The
musl and s390x questions stay out.

## Acceptance

- Release workflow produces `manylinux_2_28_x86_64` and
  `aarch64` wheels; the smoke matrix covers both; the floor is
  documented.

## Review

- Design review: pending
- Code review: pending
