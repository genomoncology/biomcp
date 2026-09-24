---
base: 36ebebca
head: bfaaa621 (merged into main after 1244)
---

Built Linux wheels on the true manylinux 2_28 floor and added Linux
ARM64, from the PyPI installability issue.

The first design review rejected a tag-only change, and the record
shows why it was right: the v0.9.0 Linux binaries import GLIBC_2.39
symbols (the 1219 container failure proved it at runtime), so
`--manylinux 2_28` on an ubuntu-24.04 runner would either fail the
audit or ship a wheel tagged 2_28 that dies on RHEL 8 — the false floor
the retired staged pipeline guarded against with a readelf check. The
floor is now true by construction: both Linux wheels build inside the
official `quay.io/pypa/manylinux_2_28_{x86_64,aarch64}` containers
(rustup 1.93.1 and maturin 1.9.6 installed inside, `--locked`, native
targets), the aarch64 leg on GitHub's free `ubuntu-24.04-arm` runners
(availability assumption recorded: a public repository). A symbol scan
(`scripts/check-wheel-glibc-floor.py`, with unit tests) fails the build
when any ELF member of the wheel references a glibc version above 2.28
or when the scan filter matches nothing; the wheel-smoke job
additionally installs and runs the wheel inside the matching 2_28
container, proving the floor at runtime, beside the deep-path smoke on
the native runners (now five legs). The runner-side toolchain steps are
gated off the Linux legs where the container does the work. The
provenance contract pins the fifth artifact name; the runbook states
the five platforms, the floor (RHEL 8, Debian 10, Ubuntu 20.04 onward,
with a current pip), the sdist exclusion (a pip sdist install would
require a user-side Rust toolchain; the tarball and cargo serve the
pre-2.28 fallback), and the ARM-runner assumption. The changelog bullet
carries the ticket marker and the current-pip caveat.

Evidence: design REJECT then ACCEPT on revision; code review REJECT
once (missing changelog ticket marker; the ARM artifact not pinned;
both one-line fixes) and ACCEPT after, with the report-only items (the
scan's no-match guard, now implemented; the dead runner steps, now
gated; the pip caveat, now worded) folded in; local contract suites
green (52 then 41 tests); the package boundary count moved to 1,354
with the ticket-1244-era comment extended, verified by the real
boundary test; yellow gate at bfaaa621 — lint, test, and spec OK
(after two earlier cycles caught the formatter shapes, the canonical
health API name, the branch-side baselines, and a duplicate import).

Residuals: the container path itself gets its first live exercise at
the 0.9.1 tag (as with the whole release gate); musl and s390x stay out
by scope.
