# Fix the wheel glibc floor check and the Linux tarball floor

From sdlc/issues/2026-09-24-wheel-glibc-floor-check-fails-every-real-wheel.md.
Blocks 0.9.1: every real Linux wheel currently fails the floor check
and nothing would publish.

## Problem

`scripts/check-wheel-glibc-floor.py` compares glibc versions as
decimals. `member_versions` (:24) turns `GLIBC_2_9` into the float
2.9, which compares above 2.28, so a binary whose highest reference is
glibc 2.9 fails a 2.28 floor. It also reads `2.30` as `2.3`, so a
2.30 reference would pass. The shipped binary references 2.3 through
2.9, so every Linux wheel fails.

Separately, the release `build` job compiles the Linux tarballs on
ubuntu-24.04 (glibc 2.39) while the runbook sends older-Linux users to
the tarball: the artifact will not run for them. Nothing pins the
wheel platform tag, the container floor smoke runs only `--version`,
the install docs never state the floor, and the rustup installer
piped into the wheel containers is unpinned.

## Design

1. Compare versions as integer pairs. `member_versions` returns
   `list[tuple[int, int]]` from the two regex groups; `wheel_floor`
   returns the max pair starting from `(0, 0)`; the floor argument is
   parsed by `re.fullmatch(r"(\d+)\.(\d+)", ...)` into a pair, and a
   malformed argument prints the usage and exits 2. Messages render
   pairs as `f"{major}.{minor}"` so 2.30 prints as 2.30.
2. Tests pin the regression table: 2.3 and 2.9 pass the 2.28 floor,
   2.28 passes, 2.29 and 2.30 fail with their exact numbers in the
   message, and a floor argument of "2.30" parses while "2,28"
   exits 2.
3. Build the two Linux tarball legs inside the same
   `quay.io/pypa/manylinux_2_28_{x86_64,aarch64}` containers used for
   wheels: the aarch64 leg moves to the free arm runner and builds
   native instead of cross-compiling. The container runs
   `cargo build --release --locked --target <triple>`, packages from
   `target/<triple>/release/biomcp`, and emits the same artifact names
   and `.sha256` sidecars. `check-wheel-glibc-floor` is not reused:
   the tarball is a raw ELF, so the same marker scan runs over the
   extracted binary via a small shared helper in the script (a
   `--elf` mode or a public function both callers use) — whichever is
   smaller, with the runbook floor assertion stated once. No protoc
   needed: prost is behind the optional `alphagenome` feature and the
   release build does not enable it.
4. Pin the wheel platform tag: maturin gains
   `--compatibility manylinux_2_28`, and the build step asserts the
   produced filename ends `manylinux_2_28_x86_64.whl` (resp.
   aarch64) before the scan runs.
5. The container floor smoke runs one deep offline command after
   `--version` (the pure-compute `variant normalize` call the venv
   smoke already uses), not just the banner.
6. State the glibc 2.28 floor in `docs/getting-started/installation.md`
   next to the pip and binary-download paths, and say the Linux
   tarball and wheels carry the same floor.
7. Pin rustup in the containers: download the versioned
   `rustup-init` for the container arch with its published `.sha256`
   sidecar, verify with `sha256sum -c`, then run it. No unpinned
   `curl | sh`.
8. Update ticket 1245's stale "Code review: pending" line to the
   recorded verdicts (the pending-check ticket later makes this
   mechanical).

## Acceptance

- The regression table in (2) passes; a synthetic wheel built from
  the real marker set (2.2.5, 2.3, 2.3.4, 2.4, 2.9, 2.17) passes.
- The workflow tests assert the tarball container legs, the pinned
  compatibility flag, the filename assertion, and the pinned rustup
  step.
- Yellow gate green at the head SHA.

## Review

- Design review: pending
- Code review: pending
