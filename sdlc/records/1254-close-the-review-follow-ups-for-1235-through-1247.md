---
base: f8c1f223
head: fe90d348 (batch 3 merge; d8475987 was the batch-1 head)
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

## Batch 2 (2026-09-26)

Items 5-8 landed. The licensing guard warns at 300 days (a real
UserWarning, not a hidden print) and fails the gate past 365, with
the boundary pinned by synthetic entries; the oldest real review
date is 2026-03-20 (190 days), recorded in the ticket with its
2027-03 crossing flagged for Ian's review pass — nothing was bumped.
The three raw-ctgov duplicate issues are merged into the canonical
file with their unique sentences restored (including the two the
first merge dropped), the dangling references repointed, and the
1244 record corrected. The 1247 test names its observable
(settles and the previous generation survives), not an ordering the
product does not have, with the stress lane's filters following. The
health handshake gained a real end-to-end test through a test-only
address override — and the review's P1 closed it properly: the
override is gated exactly like the GenCC endpoint: it activates
only when debug assertions are on, or when the override base is an
exact loopback address AND the test signal variable is set (2026-09-27
correction: the earlier wording said "never in release builds" — a
release build does honor the override when both test variables point
at loopback; the protection is the loopback-plus-signal pair, which
an unconfigured attacker cannot satisfy, not the build mode alone);
both
sides now parse as URLs with a fallback, and every probe arm
including the keyed one routes through the gate. Code review REJECT
once on the ungated override, all eight findings fixed and
re-verified; yellow gate at 43e86ff6 — lint, test, and spec OK.
Batch 3 (items 10-12, the 1235/1236/1237 minors) remains.

## Batch 3 (2026-09-26)

Items 10-12 and the spec-fixtures triage landed; the ticket is
complete. The panic-recovery contract test now runs in release under
`make verify`; `panic_payload_message` has one definition and unit
tests; the trial-alias and lease locks recover through the shared
poison helper with the read guard scoped before its await; the
provider-network inventory counts code only and covers the two real
pool builders, with a fresh-process test proving the real shared,
ORCID, and CSPEC constructors parse a valid bundle exactly once; the
stdio fallback test drives two tool calls and asserts one warning;
the loader handles a real `ca-\xff.pem`; the unreachable arm degrades
honestly; the doubled Warnings heading is gone with one-heading
pinned; `dailymed_setid_url` has one home; the drug tests floor is
back to 954. The spec-contracts lane runs the provider-contract and
variant-identity fixtures the issue named, with the copied-workspace
guard every sibling runner carries — the review caught that the
first cut broke two lifecycle tests in scratch checkouts (bash 127)
and the correction is recorded in the issue file. Code review REJECT
once on exactly that, fixed; the issue's verification section now
tells the truth about which local failures were pre-existing.
Yellow gate at ffd0fcee — lint, test, spec, and stress all OK.
