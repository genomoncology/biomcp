---
base: 337ea040
head: bcb7aafb
---

Carried DDInter coverage onto the drug card and fixed the freshness
and synonym seams, from the drug-card issue.

The card now reports coverage: `Drug` gains an optional
`interaction_coverage_status` (additive in JSON, absent when unset,
pinned by a serializer test) that `apply_interaction_report` fills and
both the failure and section-off branches clear. The provenance note
matrix distinguishes not-covered ("DDInter does not cover this drug,
so the absence of rows reflects coverage, not a clean bill"),
covered-with-no-rows, and rows-present; the template renders a
`DDInter coverage:` line beside the freshness label, so an uncovered
drug never reads as "no matching rows" alone. Freshness derives from a
cached basis plus the clock: `load_index` captures the oldest bundle
mtime into the cache entry, and `basis_freshness` reads Fresh/Stale as
`now - basis >= 72h` — an externally replaced bundle can no longer flip
a loaded index's label, while a server that outlives the window still
reports stale (the review rejected the first freeze-at-load design for
exactly that reason, and the tests pin both directions with
`File::set_modified`). Synonym matching reaches the index through a
new `#[serde(skip)] ddinter_synonyms` field populated in the transform
fold (bounded at 32, shared dedupe with brands, no break at the
three-brand cap) and threaded into `DdinterIdentity`; the identity
test proves aspirin finds a row filed under acetylsalicylic acid and
that brands alone cannot. Corrupt-bundle errors name the source and
carry the parse detail instead of a generic API line.

Evidence: design REJECT twice (freeze-at-load inverted the aging
signal; the synonym seam did not exist; then an inventory
misstatement) and ACCEPT on the third pass; code review REJECT once
with two P0s (a Result/Option method mixup that broke the build, and
the synonym population edit silently missing so the seam shipped dead)
— both fixed and verified ACCEPT; yellow gate at bcb7aafb — lint,
test, and spec OK after five cycles that caught formatter shapes, a
clone-on-copy, a non-matching variant field, and a size-inventory
addition for the card tests.

Residuals: the 8 MB cap check against the real bundle and the
real-bundle run defer to the M5 leg (recorded); no test pins the
transform population seam itself (the identity contract is pinned
instead); the combination-product resolution symptom is mitigated
incidentally and verified honestly at M5.
