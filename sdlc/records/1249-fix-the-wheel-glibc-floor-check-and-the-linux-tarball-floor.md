---
base: a9952477
head: 68f885c1
---

Fixed the wheel glibc floor check and made the Linux tarball floor
true, from the review follow-ups to ticket 1245.

The floor check compared glibc versions as decimals, so a binary
whose highest reference was 2.9 failed a 2.28 floor (every real Linux
wheel, so nothing would publish) and a 2.30 reference read as 2.3 and
passed. Versions are now integer pairs end to end: the marker regex
groups, the accumulated maximum from (0, 0), the floor argument
(fullmatch, malformed exits 2 with usage), and the messages, which
print 2.30 as 2.30. A new `--elf` mode scans a raw binary the same
way and fails closed on no markers. The regression table pins 2.3,
2.9, and 2.28 passing; 2.29 and 2.30 failing with their exact
numbers; floor-argument semantics; and the real marker set
(2.2.5 through 2.17) passing.

The Linux tarballs no longer build on ubuntu-24.04 with glibc 2.39:
both legs build inside the same `manylinux_2_28` containers as the
wheels (aarch64 native on the free ARM runner, cross-compilation
retired), the pre-tar binary passes the `--elf` floor scan — the same
bytes the tar carries — and the host toolchain steps are gated off
for Linux. The rustup installer inside every container is now pinned:
versioned `rustup-init` per container arch with its published sha256
sidecar verified before running, replacing the unpinned `curl | sh`.
The wheel legs pass `--compatibility manylinux_2_28` and assert the
produced filename ends the leg's platform tag before the scan runs.
The container floor smoke now runs one deep offline command,
`biomcp cache stats --json`, asserting the output is a JSON object —
not just the version banner. The install docs state the glibc 2.28
floor for both the tarballs and the wheels; the runbook states it
once and points older-Linux source builds at `cargo install`. Nine
parametrized mutations (container swap, ARM runner, rustup version,
checksum removal, compatibility flag, filename assertion, ELF scan,
smoke downgrade, restored pipe) each break the provenance contract.

Evidence: design REJECT once — the reviewer caught the smoke command
being network-bound (`variant normalize` proxies Mutalyzer) and a
wrong protoc rationale — folded and re-reviewed ACCEPT; code review
REJECT once on a one-character quoting bug in the container smoke
(`"{"*")` does not terminate a bash case pattern; every release would
have failed the smoke step) plus the missing guard pin, both fixed
and pinned; yellow gate at 68f885c1 — lint, test, and spec OK.

Residuals: the container legs get their first live exercise at the
0.9.1 tag (recorded in the release-prep checklist); the two rustup
sidecar digests re-verify at run time by the workflow itself.
