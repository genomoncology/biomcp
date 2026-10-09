# BioMCP 0.9.2 queue — final report

The queue is empty. Every ticket through 1305 is landed, reviewed, and
verified. No tag was pushed and nothing was posted on GitHub. The
pre-tag review can verify each item below.

## Tickets landed this cycle, with merge SHAs, review verdicts, and merged-tree CI

Each row names the merge commit on main, the code review verdict
covering the landed content (recorded on the ticket in house grammar),
and the merged-tree CI run that covered the exact landing tree. Rows
marked with a deviation note describe the honest record.

| Ticket | Merge | Review on landed content | Merged-tree CI |
| --- | --- | --- | --- |
| 2018 | 6b504219e | ACCEPT 2026-10-07 (re-review after fix fold) | green at 6b504219e (landcheck run; the branch's own CI was pool-cancelled three times — recorded on the ticket) |
| 2019 | ad39a9c5d | ACCEPT 2026-10-07 + follow-up ACCEPT | green at 50c19bb35 (the exact main tip; the first merge reached main via a grammar-fix push — deviation recorded on the ticket) |
| 2020 | d1c6d713f | ACCEPT 2026-10-08 (second-fix review) | green at 4cea37dae |
| 2021 | 44b33eb8a | ACCEPT 2026-10-08 (fix-round P1 fixed; third-review finding 12 re-review) | green at 208858393 |
| 2022 | 74c0fe87f | ACCEPT (chain: REJECT+fix, fold ACCEPT, third-review round ACCEPT) | green at d7a1323be |
| 2023 | 05eeda3f3 | ACCEPT 2026-10-07 + delta ACCEPT | green at 05eeda3f3 (records-only delta after) |
| 2024 | f6eca26b3 + eae20df65 (fix round) | ACCEPT 2026-10-07 + fix-round ACCEPT 2026-10-08 (Windows proof green in run 37714318974) | green at 60c0e090c; the fix content reached main inside the 2023 landing chain — deviation recorded on the ticket |
| 2029 | 8eec444db | ACCEPT + delta ACCEPT 2026-10-08 | green at 2d3815718 |
| 2030 | ae9e6c2e9 | ACCEPT 2026-10-08 | green at 9b23236f8 |
| 2031 | 548f0b854 | ACCEPT (chain: initial, fix-round, head review covering the final tip) | green at 50679cc02 (measured 1_407) |
| 2032 | d18d30e7f | ACCEPT 2026-10-08 | main's landing-chain CI covers the docs-union tree; the code tip's run 37798569759 green |
| 2034 | 6bf1a550a | ACCEPT 2026-10-09 | green at 6bf1a550a |
| 2016 | 27478cc1e + e99ed6cfb (fix round 4) | ACCEPT (chain: fix2, repin, fix3, fix4) | green at 04b7f6c79, and at e99ed6cfb |
| 2017 | d8afa297f | ACCEPT + delta ACCEPT | green at d8afa297f (landcheck; the merge reached main via the local-chain push — deviation recorded) |
| 1291 | e38bab076 | ACCEPT (chain: post-fix, wait-ratchet delta) | green at e38bab076 |
| 1305 | 413156e95 | ACCEPT 2026-10-09 (final round) | main's CI on the landing chain (docs-only merge on a green tree) |
| 2033/2035 | closed with dispositions | every finding fixed or recorded | their sub-tickets' rows above |

1300 landed earlier this cycle (f75292875, review ACCEPT with the
exhaustive-trace second-fix review, merged-tree green). 1295, 1296,
1297, 1298, 1299, 1301, 1302, 1303, 1304, 1306, 1290–1294 landed
before the second review and carry their recorded reviews.

## Changelog gate at a scratch v0.9.2 tag

`changelog Unreleased covers all 37 tickets merged since v0.9.1` —
exit 0, run at a local scratch tag on final main (cdeb4c0b5), tag
deleted after. The 37: 1287–1306 (all landed this cycle), 2010
(absorbed), 2016–2024, 2029–2035, and 1305 itself.

## Offline suite on final main

853 passed, 389 deselected, exit 0 — run in the isolated gate clone
on final main after sync-python-dev, under the Makefile lane's
TMPDIR convention.

## pm sweep --dry-run

No BioMCP worktrees remain (0 beyond the main checkout). No BioMCP
ticket or landcheck branches remain on origin (the 12 remaining
remote branches are the biodata team's, gh-pages, and pre-cycle
archive ticket branches untouched by this queue). Local
`~/workspace/worktrees/` holds only non-BioMCP directories. The
`yellow-gate-holder` file is clear; the yellow lock is free.

## Standing disclosures

- The deadline-test deadlock root cause (blocking receive in the
  fixture hold) is fixed by 2030; the interim gate-evidence rule is
  retired in the deadlock issue with the closure note. Four futex
  forensics captures are preserved on the gate host for the record.
- Known smaller items carried in records, not code: the NCI trial
  ambiguity refusal covers --source nci only (2032's amended Defers);
  the merge-pooling lane owns the shared-product-name and rescue-miss
  pins (2031's records); the search-enrichment path excludes the
  spelled-gene MyGene resolution (2034's boundary note).
- The diagnostic-synonym spec-contract failure stays filed as an open
  issue with its triage questions (GTR fixture drift; nothing gated
  catches it).
- 0.9.2 was not tagged. Nothing was posted on GitHub.

Main now: cdeb4c0b5. The queue is empty and milestone 0.9.2 is ready
for the pre-tag review.
