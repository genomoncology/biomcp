# Wheel glibc floor check fails every real wheel

Filed 2026-09-24 from the review of ticket 1245 (merge 2dceaae2). Blocks 0.9.1.

## Symptom

`scripts/check-wheel-glibc-floor.py` rejects a wheel whose highest glibc reference is 2.9. A test wheel referencing only 2.2.5, 2.3, 2.3.4, 2.4, 2.9, and 2.17 failed with "2.9 exceeds 2.28". The installed biomcp binary references 2.3 through 2.9, so every Linux wheel fails and nothing publishes. The check would also report 2.30 as "2.3" and pass it.

## Cause

`member_versions` (`:24`) turns `GLIBC_2_9` into the float 2.9, which is larger than 2.28.

## Fix

Compare `(major, minor)` integer tuples. Parse the floor argument the same way. Add tests: 2.3 and 2.9 pass, 2.28 passes, 2.29 and 2.30 fail.

## Other 1245 findings

- Should-fix: `docs/reference/release-process.md:14` sends older-Linux users to the GitHub tarball. The `build` job (`release.yml:70-76`) compiles tarballs on ubuntu-24.04 with glibc 2.39, so they will not run there. Build the Linux tarballs in the same 2_28 containers, or point the docs at `cargo install`.
- Should-fix: nothing pins the wheel platform tag (`release.yml:210`). Pass `--compatibility manylinux_2_28` to maturin and assert the file names end in `manylinux_2_28_x86_64` and `manylinux_2_28_aarch64`.
- Minor: the floor smoke in the 2_28 container (`release.yml:269-275`) runs only `--version`. Run one deep command there too.
- Minor: state the glibc floor in the user install docs, not only the maintainer runbook and changelog.
- Minor: `release.yml:207` pipes the rustup installer into `sh` unpinned. Pin it by checksum or use the image's toolchain.
- Minor: ticket 1245 still says "Code review: pending".

## Resolved

Ticket 1249. Versions compare as integer pairs (2.9 passes a 2.28
floor, 2.30 fails and prints as 2.30); the Linux tarballs build in
the 2_28 containers with a pinned rustup; the wheel tag is pinned and
asserted; the floor smoke runs an offline deep command; the install
docs state the floor; 1245's ticket review line is corrected. The
container legs' first live run stays at the 0.9.1 tag. See
`sdlc/records/1249-fix-the-wheel-glibc-floor-check-and-the-linux-tarball-floor.md`.
