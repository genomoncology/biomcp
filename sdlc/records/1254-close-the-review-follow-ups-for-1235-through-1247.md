---
base: f8c1f223
head: d8475987
---

Batch 1 of the review follow-ups: the DDInter and GenCC items
(1254's items 1-4).

Synonyms now come from the chosen MyChem anchor hit only: the fold
gates the synonym push on the anchor index, so a combination
product's synonym list can no longer name a real interaction partner
and have the aggregation skip drop that row (the dipyridamole
regression test proves the row survives); the more-than-three-
DrugBank-synonym test proves "acetylsalicylic acid" reaches
`ddinter_synonyms` and the identity. The freshness tests drive
`cached_index_for_root` with real bundles and `File::set_modified`,
pinning both directions — the old vacuous test (the same fixed
timestamp twice) is gone. DDInter read and parse errors carry a
marker the render arm matches selectively, and the download-failure
message no longer embeds upstream body text (the 503 test asserts
neither the status nor the body appears); local missing-file reads
stay the generic unavailable line, recorded. The covered-zero-rows
wording is pinned. GenCC directory validation routes through the
errno taxonomy: a deliberate mismatch (wrong mode, wrong owner,
not-a-directory) is Invalid and prunable — the 0755 test proves a
real publish cycle prunes the mangled generation — and EACCES,
ESTALE, and EAGAIN deliberately retain (environmental, recorded in
the ticket and pinned by tests).

The batch also found the cause of the day's flapping package counts:
transient `uv-*.lock` files from concurrent uv invocations appear in
`cargo package --list` when untracked. They are gitignored now, so
the boundary count is deterministic (1,361 confirmed by a full
package run).

Evidence: code review ACCEPT with three report-only P2s (the unused
body parameter, dropped; the local-read generic line, recorded; the
floor attestation, confirmed by the 11-file diff-stat and a passing
ratchet); yellow gate at d8475987 — lint, test, and spec OK after
one cycle that exposed the uv-lock flap. Items 5-12 remain for
batches 2 and 3.
