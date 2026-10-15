---
base: 4d05cfd8
head: 064cac91
---

Carried the stale-cache note into every search body that can have
one, from the 2026-09-28 review file.

Article search and search-all now show the age sentence in
`_meta.notes`, proven end to end against a real fixture server
(stale carries it, fresh does not). GWAS cannot: its client never
persists responses (`CacheMode::NoStore`, recorded in code after
earlier cache decode failures), so no stale serve exists to
describe — the exclusion is recorded in the get-JSON notes issue
with the code citation, and the original GWAS test was
removed rather than forced green. The ClinGen prefetch — the audit
found it is the only production fetch running on a bare spawn —
inherits the command's note collector through a handle taken
before the spawn, so a stale ClinGen result reaches the output;
a unit test proves the seam.

Two honesty faults fixed with it: the DDInter comment now credits
the upstream header it echoes, and the plain MyDisease label branch
is gone because no producer can write that source string — the
reason is recorded where the branch stood.

Evidence: review REJECT once (a struct literal that could not
compile, non-hermetic tests) folded; six gate cycles to green —
the compile literal, a string-typed fixture field, the GWAS NoStore
discovery, rustfmt, a collapsible if and a held-for-drop field
(with its dead-code exception), and the package boundary count for
the new sidecar — green at 064cac91, all four phases, zero failed
lines.
