# PyPI wheels miss older Linux and Linux ARM64

Filed 2026-09-23 from an independent review of `v0.9.0..f2549676`.

## Symptom

PyPI serves only a `manylinux_2_39_x86_64` Linux wheel and no source package. `pip install biomcp-cli` fails on Ubuntu 22.04, Debian 12, and RHEL 8 and 9. Linux ARM64 has no wheel at all.

## Fix

The code is pure Rust with rustls and checked-in generated protobuf code, and the ARM64 tarball already builds (`release.yml:37`).

- Set `manylinux: 2_28` on the Linux wheel builds.
- Add `ubuntu-24.04-arm` with `aarch64-unknown-linux-gnu` to the `pypi-build` matrix.
- Add a matching wheel-smoke leg.
